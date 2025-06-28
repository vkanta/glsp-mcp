/**
 * Canvas-based diagram renderer
 * Renders diagram elements using HTML5 Canvas
 */

import { DiagramModel, ModelElement, Node, Edge, Bounds, Position } from '../model/diagram.js';
import { SelectionManager } from '../selection/selection-manager.js';
import { InteractionMode, InteractionModeManager } from '../interaction/interaction-mode.js';
import { WasmComponentRendererV2 } from '../diagrams/wasm-component-renderer-v2.js';
import { McpClient } from '../mcp/client.js';

export interface RenderOptions {
    backgroundColor?: string;
    nodeColor?: string;
    edgeColor?: string;
    textColor?: string;
    selectedColor?: string;
    gridEnabled?: boolean;
    gridColor?: string;
    scale?: number;
    offset?: Position;
}

export interface InteractionEvent {
    type: 'click' | 'hover' | 'drag-start' | 'drag-move' | 'drag-end' | 'canvas-click' | 'edge-start' | 'edge-end';
    position: Position;
    element?: ModelElement;
    sourceElement?: ModelElement; // For edge creation
    originalEvent: MouseEvent;
}

export type InteractionHandler = (event: InteractionEvent) => void;

export class CanvasRenderer {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private options: Required<RenderOptions>;
    private currentDiagram?: DiagramModel;
    private selectionManager: SelectionManager;
    private modeManager: InteractionModeManager;
    private interactionHandlers: InteractionHandler[] = [];
    private isDragging = false;
    private dragStart?: Position;
    private dragOffsets: Map<string, Position> = new Map();
    private hasDragged = false;
    private isPanning = false;
    private panStart?: Position;
    private selectionRect?: { start: Position; end: Position };
    private isSelectingRect = false;
    private edgeCreationSource?: ModelElement;
    private edgeCreationType?: string; // Will be used when creating edges
    private edgePreviewTarget?: Position;
    private minScale = 0.1;
    private maxScale = 5.0;
    private scrollBounds?: { minX: number; minY: number; maxX: number; maxY: number };
    private mcpClient?: McpClient;

    constructor(canvas: HTMLCanvasElement, options: RenderOptions = {}) {
        this.canvas = canvas;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
            throw new Error('Unable to get 2D context from canvas');
        }
        this.ctx = ctx;
        this.selectionManager = new SelectionManager();
        this.modeManager = new InteractionModeManager();

        this.options = {
            backgroundColor: '#ffffff',
            nodeColor: '#e1f5fe',
            edgeColor: '#666666',
            textColor: '#333333',
            selectedColor: '#2196f3',
            gridEnabled: true,
            gridColor: '#f0f0f0',
            scale: 1.0,
            offset: { x: 0, y: 0 },
            ...options
        };

        this.setupEventListeners();
        this.resizeCanvas();
        
        // Listen to selection changes
        this.selectionManager.addChangeHandler(() => {
            this.render();
        });
    }

    private setupEventListeners(): void {
        // Mouse events - Note: mousedown/up should be before click for proper event ordering
        this.canvas.addEventListener('mousedown', this.handleMouseDown.bind(this));
        this.canvas.addEventListener('mousemove', this.handleMouseMove.bind(this));
        this.canvas.addEventListener('mouseup', this.handleMouseUp.bind(this));
        this.canvas.addEventListener('click', this.handleClick.bind(this));
        this.canvas.addEventListener('wheel', this.handleWheel.bind(this));

        // Resize observer
        const resizeObserver = new ResizeObserver(() => {
            this.resizeCanvas();
            this.render();
        });
        resizeObserver.observe(this.canvas.parentElement || this.canvas);
    }

    private resizeCanvas(): void {
        const parent = this.canvas.parentElement || this.canvas;
        const rect = parent.getBoundingClientRect();
        
        // Set canvas size to fill parent
        this.canvas.width = rect.width * window.devicePixelRatio;
        this.canvas.height = rect.height * window.devicePixelRatio;
        this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
        
        // Set CSS size
        this.canvas.style.width = rect.width + 'px';
        this.canvas.style.height = rect.height + 'px';
        this.canvas.style.display = 'block';
    }

    addInteractionHandler(handler: InteractionHandler): void {
        this.interactionHandlers.push(handler);
    }

    removeInteractionHandler(handler: InteractionHandler): void {
        const index = this.interactionHandlers.indexOf(handler);
        if (index > -1) {
            this.interactionHandlers.splice(index, 1);
        }
    }

    private emit(event: InteractionEvent): void {
        this.interactionHandlers.forEach(handler => handler(event));
    }

    private getMousePosition(event: MouseEvent): Position {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: (event.clientX - rect.left - this.options.offset.x) / this.options.scale,
            y: (event.clientY - rect.top - this.options.offset.y) / this.options.scale
        };
    }

    private getElementAt(position: Position): ModelElement | undefined {
        if (!this.currentDiagram) return undefined;

        // Check nodes first (they should be on top)
        for (const element of Object.values(this.currentDiagram.elements)) {
            const elementType = element.type || element.element_type || '';
            if (elementType === 'graph') continue;
            
            if (element.bounds && this.isPointInBounds(position, element.bounds)) {
                return element;
            }
        }

        // Check edges if no node was found
        for (const element of Object.values(this.currentDiagram.elements)) {
            const elementType = element.type || element.element_type || '';
            
            // Check for various edge types
            const isEdge = elementType.includes('edge') || 
                          elementType === 'flow' || 
                          elementType === 'association' || 
                          elementType === 'dependency' ||
                          elementType === 'sequence-flow' ||
                          elementType === 'message-flow' ||
                          elementType === 'conditional-flow';
            
            if (isEdge && this.isPointOnEdge(position, element)) {
                return element;
            }
        }

        return undefined;
    }

    private isPointInBounds(point: Position, bounds: Bounds): boolean {
        return point.x >= bounds.x && 
               point.x <= bounds.x + bounds.width &&
               point.y >= bounds.y && 
               point.y <= bounds.y + bounds.height;
    }

    private isPointOnEdge(point: Position, edge: any): boolean {
        const tolerance = 8; // Pixels tolerance for edge selection (scaled)
        const scaledTolerance = tolerance / this.options.scale;
        
        // Get source and target elements
        const sourceId = edge.sourceId || edge.properties?.sourceId;
        const targetId = edge.targetId || edge.properties?.targetId;
        
        if (!sourceId || !targetId) return false;
        
        const sourceElement = this.currentDiagram?.elements[sourceId];
        const targetElement = this.currentDiagram?.elements[targetId];
        
        if (!sourceElement?.bounds || !targetElement?.bounds) return false;
        
        // Calculate source and target centers
        const sourceCenter = {
            x: sourceElement.bounds.x + sourceElement.bounds.width / 2,
            y: sourceElement.bounds.y + sourceElement.bounds.height / 2
        };
        
        const targetCenter = {
            x: targetElement.bounds.x + targetElement.bounds.width / 2,
            y: targetElement.bounds.y + targetElement.bounds.height / 2
        };
        
        // Create line segments to check
        const segments: Array<{start: Position, end: Position}> = [];
        
        if (edge.routingPoints && edge.routingPoints.length > 0) {
            // Edge has routing points
            let prevPoint = sourceCenter;
            
            for (const routingPoint of edge.routingPoints) {
                segments.push({ start: prevPoint, end: routingPoint });
                prevPoint = routingPoint;
            }
            
            // Final segment to target
            segments.push({ start: prevPoint, end: targetCenter });
        } else {
            // Direct line from source to target
            segments.push({ start: sourceCenter, end: targetCenter });
        }
        
        // Check if point is close to any segment
        for (const segment of segments) {
            if (this.distanceToLineSegment(point, segment.start, segment.end) <= scaledTolerance) {
                return true;
            }
        }
        
        return false;
    }

    private distanceToLineSegment(point: Position, lineStart: Position, lineEnd: Position): number {
        const A = point.x - lineStart.x;
        const B = point.y - lineStart.y;
        const C = lineEnd.x - lineStart.x;
        const D = lineEnd.y - lineStart.y;

        const dot = A * C + B * D;
        const lenSq = C * C + D * D;
        
        if (lenSq === 0) {
            // Line start and end are the same point
            return Math.sqrt(A * A + B * B);
        }
        
        let param = dot / lenSq;

        let xx, yy;

        if (param < 0) {
            xx = lineStart.x;
            yy = lineStart.y;
        } else if (param > 1) {
            xx = lineEnd.x;
            yy = lineEnd.y;
        } else {
            xx = lineStart.x + param * C;
            yy = lineStart.y + param * D;
        }

        const dx = point.x - xx;
        const dy = point.y - yy;
        return Math.sqrt(dx * dx + dy * dy);
    }

    private handleClick(event: MouseEvent): void {
        // Don't process click if we just finished dragging
        if (this.hasDragged) {
            this.hasDragged = false;
            return;
        }
        
        const position = this.getMousePosition(event);
        const element = this.getElementAt(position);
        const mode = this.modeManager.getMode();
        
        // console.log('Click:', { position, element, mode });

        switch (mode) {
            case InteractionMode.Select:
                if (element) {
                    this.selectionManager.handleKeyboardSelection(element.id, event);
                } else if (!event.ctrlKey && !event.metaKey) {
                    // Clear selection when clicking empty space
                    this.selectionManager.clearSelection();
                }
                this.emit({
                    type: 'click',
                    position,
                    element,
                    originalEvent: event
                });
                break;
                
            case InteractionMode.CreateNode:
                if (!element) {
                    // Create node at clicked position
                    this.emit({
                        type: 'canvas-click',
                        position,
                        originalEvent: event
                    });
                }
                break;
                
            case InteractionMode.CreateEdge:
                const elementTypeForEdge = element?.type || element?.element_type || '';
                if (element && (elementTypeForEdge === 'task' || elementTypeForEdge.includes('event') || elementTypeForEdge === 'gateway')) {
                    if (!this.edgeCreationSource) {
                        // Start edge creation
                        this.edgeCreationSource = element;
                        this.emit({
                            type: 'edge-start',
                            position,
                            element,
                            originalEvent: event
                        });
                    } else if (element.id !== this.edgeCreationSource.id) {
                        // Complete edge creation
                        this.emit({
                            type: 'edge-end',
                            position,
                            element,
                            sourceElement: this.edgeCreationSource,
                            originalEvent: event
                        });
                        this.edgeCreationSource = undefined;
                        this.edgePreviewTarget = undefined;
                    }
                } else if (!element && this.edgeCreationSource) {
                    // Cancel edge creation
                    this.edgeCreationSource = undefined;
                    this.edgePreviewTarget = undefined;
                    this.render();
                }
                break;
        }
    }

    private handleMouseMove(event: MouseEvent): void {
        const position = this.getMousePosition(event);
        const element = this.getElementAt(position);
        
        // Handle panning
        if (this.isPanning && this.panStart) {
            const deltaX = event.clientX - this.panStart.x;
            const deltaY = event.clientY - this.panStart.y;
            
            this.options.offset.x += deltaX;
            this.options.offset.y += deltaY;
            
            this.panStart = { x: event.clientX, y: event.clientY };
            this.render();
            return;
        }

        if (this.isDragging && this.dragStart) {
            this.hasDragged = true; // Mark that we've actually dragged
            
            // Update positions of all selected elements
            this.dragOffsets.forEach((offset, id) => {
                const elem = this.currentDiagram?.elements[id];
                if (elem?.bounds) {
                    elem.bounds.x = position.x + offset.x;
                    elem.bounds.y = position.y + offset.y;
                }
            });

            this.emit({
                type: 'drag-move',
                position,
                element,
                originalEvent: event
            });

            this.render();
        } else {
            // Check for interface connector hover first
            const interfaceConnector = this.getInterfaceConnectorAt(position);
            
            if (interfaceConnector) {
                // Show interface connector tooltip
                this.showInterfaceTooltip(interfaceConnector, position);
            } else {
                // Hide tooltip if no connector
                this.hideInterfaceTooltip();
                
                // Handle regular element hover
                const newHovered = element?.id || null;
                if (newHovered !== this.selectionManager.getState().hoveredElement) {
                    this.selectionManager.setHover(newHovered);

                    if (element) {
                        this.emit({
                            type: 'hover',
                            position,
                            element,
                            originalEvent: event
                        });
                    }
                }
            }
            
            // Update cursor based on mode and hover state
            const mode = this.modeManager.getMode();
            if (mode === InteractionMode.Select && element && this.selectionManager.isSelected(element.id)) {
                this.canvas.style.cursor = 'grab';
            } else if (mode === InteractionMode.CreateNode) {
                this.canvas.style.cursor = 'crosshair';
            } else if (mode === InteractionMode.CreateEdge) {
                this.canvas.style.cursor = element ? 'pointer' : 'default';
            } else if (mode === InteractionMode.Pan) {
                this.canvas.style.cursor = 'move';
            } else {
                this.canvas.style.cursor = 'default';
            }
            
            // Update edge preview if creating edge
            if (this.edgeCreationSource) {
                this.edgePreviewTarget = position;
                this.render();
            }
        }
    }

    private handleMouseDown(event: MouseEvent): void {
        const position = this.getMousePosition(event);
        const element = this.getElementAt(position);
        const mode = this.modeManager.getMode();
        
        // Handle middle mouse button for panning
        if (event.button === 1 || (event.button === 0 && mode === InteractionMode.Pan)) {
            this.isPanning = true;
            this.panStart = { x: event.clientX, y: event.clientY };
            this.canvas.style.cursor = 'move';
            event.preventDefault();
            return;
        }

        if (element && this.selectionManager.isSelected(element.id) && mode === InteractionMode.Select) {
            this.isDragging = true;
            this.dragStart = position;
            this.canvas.style.cursor = 'grabbing';
            
            // Calculate drag offsets for all selected elements
            this.dragOffsets.clear();
            const selectedIds = this.selectionManager.getSelectedIds();
            
            selectedIds.forEach(id => {
                const elem = this.currentDiagram?.elements[id];
                if (elem?.bounds) {
                    this.dragOffsets.set(id, {
                        x: elem.bounds.x - position.x,
                        y: elem.bounds.y - position.y
                    });
                }
            });

            this.emit({
                type: 'drag-start',
                position,
                element,
                originalEvent: event
            });
        }
    }

    private handleMouseUp(event: MouseEvent): void {
        if (this.isPanning) {
            this.isPanning = false;
            this.panStart = undefined;
            this.canvas.style.cursor = 'default';
            return;
        }
        
        if (this.isDragging) {
            const position = this.getMousePosition(event);
            const element = this.getElementAt(position);

            this.isDragging = false;
            this.dragStart = undefined;
            this.dragOffsets.clear();
            this.canvas.style.cursor = 'default';

            this.emit({
                type: 'drag-end',
                position,
                element,
                originalEvent: event
            });
        }
    }

    private handleWheel(event: WheelEvent): void {
        event.preventDefault();
        
        // Use Ctrl/Cmd for zoom, otherwise pan
        if (event.ctrlKey || event.metaKey) {
            // Zoom
            const scaleFactor = event.deltaY > 0 ? 0.9 : 1.1;
            const newScale = Math.max(this.minScale, Math.min(this.maxScale, this.options.scale * scaleFactor));
            
            const rect = this.canvas.getBoundingClientRect();
            const mouseX = event.clientX - rect.left;
            const mouseY = event.clientY - rect.top;
            
            // Calculate world position before zoom
            const worldX = (mouseX - this.options.offset.x) / this.options.scale;
            const worldY = (mouseY - this.options.offset.y) / this.options.scale;
            
            // Update scale
            this.options.scale = newScale;
            
            // Calculate new offset to keep mouse position fixed
            this.options.offset.x = mouseX - worldX * newScale;
            this.options.offset.y = mouseY - worldY * newScale;
        } else {
            // Pan with scroll wheel
            const sensitivity = 1.0;
            this.options.offset.x -= event.deltaX * sensitivity;
            this.options.offset.y -= event.deltaY * sensitivity;
        }
        
        // Apply pan constraints if we have bounds
        this.constrainPan();
        this.render();
    }

    setDiagram(diagram: DiagramModel): void {
        this.currentDiagram = diagram;
        this.selectionManager.clearSelection();
        this.updateScrollBounds();
        this.render();
    }

    clear(): void {
        this.currentDiagram = undefined;
        this.selectionManager.clearSelection();
        this.render();
    }

    setSelected(elementIds: string[]): void {
        this.selectionManager.selectMultiple(elementIds, false);
    }
    
    getSelectionManager(): SelectionManager {
        return this.selectionManager;
    }
    
    getModeManager(): InteractionModeManager {
        return this.modeManager;
    }

    pan(deltaX: number, deltaY: number): void {
        this.options.offset.x += deltaX;
        this.options.offset.y += deltaY;
        this.render();
    }

    zoom(scaleFactor: number, center?: Position): void {
        const newScale = Math.max(this.minScale, Math.min(this.maxScale, this.options.scale * scaleFactor));
        
        if (center) {
            // Zoom toward specific point
            this.options.offset.x = center.x - (center.x - this.options.offset.x) * (newScale / this.options.scale);
            this.options.offset.y = center.y - (center.y - this.options.offset.y) * (newScale / this.options.scale);
        } else {
            // Zoom toward canvas center
            const centerX = this.canvas.clientWidth / 2;
            const centerY = this.canvas.clientHeight / 2;
            this.options.offset.x = centerX - (centerX - this.options.offset.x) * (newScale / this.options.scale);
            this.options.offset.y = centerY - (centerY - this.options.offset.y) * (newScale / this.options.scale);
        }
        
        this.options.scale = newScale;
        this.constrainPan();
        this.render();
    }

    fitToContent(): void {
        if (!this.currentDiagram) return;

        const elements = Object.values(this.currentDiagram.elements).filter(e => e.bounds);
        if (elements.length === 0) return;

        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

        elements.forEach(element => {
            if (element.bounds) {
                minX = Math.min(minX, element.bounds.x);
                minY = Math.min(minY, element.bounds.y);
                maxX = Math.max(maxX, element.bounds.x + element.bounds.width);
                maxY = Math.max(maxY, element.bounds.y + element.bounds.height);
            }
        });

        const contentWidth = maxX - minX;
        const contentHeight = maxY - minY;
        const canvasWidth = this.canvas.clientWidth;
        const canvasHeight = this.canvas.clientHeight;

        const scaleX = canvasWidth / (contentWidth + 100); // Add padding
        const scaleY = canvasHeight / (contentHeight + 100);
        const scale = Math.min(scaleX, scaleY, 1.0); // Don't zoom in beyond 100%

        this.options.scale = scale;
        this.options.offset.x = (canvasWidth - contentWidth * scale) / 2 - minX * scale;
        this.options.offset.y = (canvasHeight - contentHeight * scale) / 2 - minY * scale;
        
        // Update scroll bounds
        this.updateScrollBounds();
        this.render();
    }

    render(): void {
        this.ctx.save();
        
        // Clear canvas
        this.ctx.fillStyle = this.options.backgroundColor;
        this.ctx.fillRect(0, 0, this.canvas.clientWidth, this.canvas.clientHeight);

        // If no diagram, show empty state
        if (!this.currentDiagram) {
            this.drawEmptyState();
            this.ctx.restore();
            return;
        }

        // Apply transformations
        this.ctx.translate(this.options.offset.x, this.options.offset.y);
        this.ctx.scale(this.options.scale, this.options.scale);

        // Draw grid
        if (this.options.gridEnabled) {
            this.drawGrid();
        }

        // Draw edges first (so they appear behind nodes)
        this.drawEdges();

        // Draw nodes
        this.drawNodes();
        
        // Draw selection rectangle if active
        if (this.isSelectingRect && this.selectionRect) {
            this.drawSelectionRectangle();
        }
        
        // Draw edge preview if creating edge
        if (this.edgeCreationSource && this.edgePreviewTarget) {
            this.drawEdgePreview();
        }

        this.ctx.restore();
    }
    
    private updateScrollBounds(): void {
        if (!this.currentDiagram) return;
        
        const elements = Object.values(this.currentDiagram.elements).filter(e => e.bounds);
        if (elements.length === 0) {
            this.scrollBounds = undefined;
            return;
        }
        
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        
        elements.forEach(element => {
            if (element.bounds) {
                let elementMinX = element.bounds.x;
                let elementMinY = element.bounds.y;
                let elementMaxX = element.bounds.x + element.bounds.width;
                let elementMaxY = element.bounds.y + element.bounds.height;
                
                // For WASM components, account for interface connectors that extend beyond bounds
                const nodeType = element.type || element.element_type || '';
                if (this.isWasmComponentType(nodeType) && nodeType !== 'import-interface' && nodeType !== 'export-interface') {
                    const interfaces = element.properties?.interfaces as any[] || [];
                    if (interfaces.length > 0) {
                        const connectorExtension = 20; // connectors extend ~20px from component edges
                        elementMinX -= connectorExtension; // Left side import connectors
                        elementMaxX += connectorExtension; // Right side export connectors
                        
                        // Account for interface labels
                        const labelExtension = 80; // Interface labels can be quite long
                        elementMinX -= labelExtension; // Left side labels
                        elementMaxX += labelExtension; // Right side labels
                    }
                }
                
                minX = Math.min(minX, elementMinX);
                minY = Math.min(minY, elementMinY);
                maxX = Math.max(maxX, elementMaxX);
                maxY = Math.max(maxY, elementMaxY);
            }
        });
        
        // Add padding around content
        const padding = 200;
        this.scrollBounds = {
            minX: minX - padding,
            minY: minY - padding,
            maxX: maxX + padding,
            maxY: maxY + padding
        };
    }
    
    private constrainPan(): void {
        if (!this.scrollBounds) return;
        
        const canvasWidth = this.canvas.clientWidth;
        const canvasHeight = this.canvas.clientHeight;
        
        // Calculate world bounds at current scale
        const worldMinX = this.scrollBounds.minX * this.options.scale;
        const worldMinY = this.scrollBounds.minY * this.options.scale;
        const worldMaxX = this.scrollBounds.maxX * this.options.scale;
        const worldMaxY = this.scrollBounds.maxY * this.options.scale;
        
        // Calculate content size in screen coordinates
        const contentWidth = worldMaxX - worldMinX;
        const contentHeight = worldMaxY - worldMinY;
        
        // Only constrain if content is larger than viewport
        if (contentWidth > canvasWidth) {
            // Allow scrolling within content bounds
            const maxOffsetX = canvasWidth - worldMinX;
            const minOffsetX = canvasWidth - worldMaxX;
            this.options.offset.x = Math.max(minOffsetX, Math.min(maxOffsetX, this.options.offset.x));
        }
        
        if (contentHeight > canvasHeight) {
            // Allow scrolling within content bounds
            const maxOffsetY = canvasHeight - worldMinY;
            const minOffsetY = canvasHeight - worldMaxY;
            this.options.offset.y = Math.max(minOffsetY, Math.min(maxOffsetY, this.options.offset.y));
        }
    }
    
    getScale(): number {
        return this.options.scale;
    }
    
    getOffset(): Position {
        return { ...this.options.offset };
    }
    
    getWorldBounds(): { minX: number; minY: number; maxX: number; maxY: number } | undefined {
        return this.scrollBounds ? { ...this.scrollBounds } : undefined;
    }
    
    resetView(): void {
        this.options.scale = 1.0;
        this.options.offset = { x: 0, y: 0 };
        this.updateScrollBounds();
        this.render();
    }

    // Public method to refresh scroll bounds when content changes
    refreshScrollBounds(): void {
        this.updateScrollBounds();
        this.render();
    }

    private drawGrid(): void {
        const gridSize = 20;
        const startX = Math.floor(-this.options.offset.x / this.options.scale / gridSize) * gridSize;
        const startY = Math.floor(-this.options.offset.y / this.options.scale / gridSize) * gridSize;
        const endX = startX + (this.canvas.clientWidth / this.options.scale) + gridSize;
        const endY = startY + (this.canvas.clientHeight / this.options.scale) + gridSize;

        this.ctx.strokeStyle = this.options.gridColor;
        this.ctx.lineWidth = 1 / this.options.scale;
        this.ctx.beginPath();

        for (let x = startX; x < endX; x += gridSize) {
            this.ctx.moveTo(x, startY);
            this.ctx.lineTo(x, endY);
        }

        for (let y = startY; y < endY; y += gridSize) {
            this.ctx.moveTo(startX, y);
            this.ctx.lineTo(endX, y);
        }

        this.ctx.stroke();
    }

    private drawNodes(): void {
        if (!this.currentDiagram) return;

        Object.values(this.currentDiagram.elements).forEach(element => {
            const elementType = element.type || element.element_type;
            if (elementType === 'graph' || !element.bounds) return;
            
            this.drawNode(element as Node);
        });
    }

    private drawNode(node: Node): void {
        if (!node.bounds) return;

        const isSelected = this.selectionManager.isSelected(node.id);
        const isHovered = this.selectionManager.isHovered(node.id);
        const nodeType = node.type || node.element_type || '';

        // Check if this is a WASM component type
        if (this.isWasmComponentType(nodeType)) {
            const colors = WasmComponentRendererV2.getDefaultColors();
            
            // Check if component file is missing or not loaded
            const isMissing = this.isComponentMissingFile(node);
            const isLoaded = node.properties?.isLoaded === true;
            
            // Update node properties to include load status
            if (!node.properties) {
                node.properties = {};
            }
            
            const context = {
                ctx: this.ctx,
                scale: this.options.scale,
                isSelected,
                isHovered,
                isMissing: isMissing, // Only show as missing if file is actually missing
                colors
            };

            // Use the new V2 renderer for all WASM components
            // The V2 renderer handles both main components and interface nodes in one unified design
            WasmComponentRendererV2.renderWasmComponent(node, node.bounds, context);
            return;
        }

        // Default rendering for other node types
        this.ctx.fillStyle = isSelected ? this.options.selectedColor : this.options.nodeColor;
        this.ctx.strokeStyle = isHovered ? this.options.selectedColor : this.options.edgeColor;
        this.ctx.lineWidth = isSelected ? 3 : 1;

        // Draw different shapes based on node type
        switch (nodeType) {
            case 'start-event':
                this.drawCircle(node.bounds, true);
                break;
            case 'end-event':
                this.drawCircle(node.bounds, false);
                this.ctx.lineWidth = 3;
                this.ctx.stroke();
                break;
            case 'gateway':
                this.drawDiamond(node.bounds);
                break;
            default:
                this.drawRectangle(node.bounds);
                break;
        }

        // Draw label
        const label = node.label || node.properties?.label;
        if (label) {
            this.drawLabel(label, node.bounds);
        }
    }
    
    private isWasmComponentType(nodeType: string): boolean {
        return [
            'wasm-component',
            'host-component',
            'import-interface',
            'export-interface',
            'composition-root'
        ].includes(nodeType);
    }

    private drawEdges(): void {
        if (!this.currentDiagram) return;

        Object.values(this.currentDiagram.elements).forEach(element => {
            const elementType = element.type || element.element_type || '';
            // Check for various edge types - the backend might use 'flow' instead of 'edge'
            const isEdge = elementType.includes('edge') || 
                          elementType === 'flow' || 
                          elementType === 'association' || 
                          elementType === 'dependency' ||
                          elementType === 'sequence-flow' ||
                          elementType === 'message-flow' ||
                          elementType === 'conditional-flow';
            
            if (!isEdge) return;
            
            this.drawEdge(element as Edge);
        });
    }

    private drawEdge(edge: Edge): void {
        // Handle both direct properties and properties object
        const sourceId = edge.sourceId || edge.properties?.sourceId;
        const targetId = edge.targetId || edge.properties?.targetId;
        
        if (!sourceId || !targetId) {
            console.warn('Edge missing sourceId or targetId:', edge);
            return;
        }
        
        const sourceElement = this.currentDiagram?.elements[sourceId];
        const targetElement = this.currentDiagram?.elements[targetId];

        if (!sourceElement?.bounds || !targetElement?.bounds) return;

        const isSelected = this.selectionManager.isSelected(edge.id);
        const isHovered = this.selectionManager.isHovered(edge.id);

        this.ctx.strokeStyle = isSelected || isHovered ? this.options.selectedColor : this.options.edgeColor;
        this.ctx.lineWidth = isSelected ? 3 : (isHovered ? 2 : 1);

        // Calculate connection points
        const sourceCenter = {
            x: sourceElement.bounds.x + sourceElement.bounds.width / 2,
            y: sourceElement.bounds.y + sourceElement.bounds.height / 2
        };

        const targetCenter = {
            x: targetElement.bounds.x + targetElement.bounds.width / 2,
            y: targetElement.bounds.y + targetElement.bounds.height / 2
        };

        // Draw line
        this.ctx.beginPath();
        this.ctx.moveTo(sourceCenter.x, sourceCenter.y);

        if (edge.routingPoints && edge.routingPoints.length > 0) {
            edge.routingPoints.forEach(point => {
                this.ctx.lineTo(point.x, point.y);
            });
        }

        this.ctx.lineTo(targetCenter.x, targetCenter.y);
        this.ctx.stroke();

        // Draw arrowhead
        this.drawArrowhead(targetCenter, sourceCenter);

        // Draw edge label
        const edgeLabel = edge.label || edge.properties?.label;
        if (edgeLabel) {
            const midPoint = {
                x: (sourceCenter.x + targetCenter.x) / 2,
                y: (sourceCenter.y + targetCenter.y) / 2
            };
            this.drawEdgeLabel(edgeLabel, midPoint);
        }
    }

    private drawCircle(bounds: Bounds, filled: boolean = false): void {
        const centerX = bounds.x + bounds.width / 2;
        const centerY = bounds.y + bounds.height / 2;
        const radius = Math.min(bounds.width, bounds.height) / 2;

        this.ctx.beginPath();
        this.ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
        
        if (filled) {
            this.ctx.fill();
        }
        this.ctx.stroke();
    }

    private drawRectangle(bounds: Bounds): void {
        this.ctx.fillRect(bounds.x, bounds.y, bounds.width, bounds.height);
        this.ctx.strokeRect(bounds.x, bounds.y, bounds.width, bounds.height);
    }

    private drawDiamond(bounds: Bounds): void {
        const centerX = bounds.x + bounds.width / 2;
        const centerY = bounds.y + bounds.height / 2;
        // const halfWidth = bounds.width / 2;
        // const halfHeight = bounds.height / 2;

        this.ctx.beginPath();
        this.ctx.moveTo(centerX, bounds.y);
        this.ctx.lineTo(bounds.x + bounds.width, centerY);
        this.ctx.lineTo(centerX, bounds.y + bounds.height);
        this.ctx.lineTo(bounds.x, centerY);
        this.ctx.closePath();
        
        this.ctx.fill();
        this.ctx.stroke();
    }

    private drawLabel(text: string, bounds: Bounds): void {
        this.ctx.fillStyle = this.options.textColor;
        this.ctx.font = '12px Arial';
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';

        const centerX = bounds.x + bounds.width / 2;
        const centerY = bounds.y + bounds.height / 2;

        this.ctx.fillText(text, centerX, centerY);
    }

    private drawEdgeLabel(text: string, position: Position): void {
        this.ctx.fillStyle = this.options.backgroundColor;
        this.ctx.strokeStyle = this.options.backgroundColor;
        this.ctx.lineWidth = 3;
        this.ctx.font = '10px Arial';
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';

        // Draw background
        const metrics = this.ctx.measureText(text);
        const padding = 2;
        this.ctx.fillRect(
            position.x - metrics.width / 2 - padding,
            position.y - 6 - padding,
            metrics.width + padding * 2,
            12 + padding * 2
        );

        // Draw text
        this.ctx.fillStyle = this.options.textColor;
        this.ctx.fillText(text, position.x, position.y);
    }

    private drawArrowhead(to: Position, from: Position): void {
        const angle = Math.atan2(to.y - from.y, to.x - from.x);
        const arrowLength = 10;
        // const arrowWidth = 6;

        this.ctx.beginPath();
        this.ctx.moveTo(to.x, to.y);
        this.ctx.lineTo(
            to.x - arrowLength * Math.cos(angle - Math.PI / 6),
            to.y - arrowLength * Math.sin(angle - Math.PI / 6)
        );
        this.ctx.moveTo(to.x, to.y);
        this.ctx.lineTo(
            to.x - arrowLength * Math.cos(angle + Math.PI / 6),
            to.y - arrowLength * Math.sin(angle + Math.PI / 6)
        );
        this.ctx.stroke();
    }
    
    private drawSelectionRectangle(): void {
        if (!this.selectionRect) return;
        
        const { start, end } = this.selectionRect;
        const x = Math.min(start.x, end.x);
        const y = Math.min(start.y, end.y);
        const width = Math.abs(end.x - start.x);
        const height = Math.abs(end.y - start.y);
        
        this.ctx.strokeStyle = this.options.selectedColor;
        this.ctx.lineWidth = 1;
        this.ctx.setLineDash([5, 5]);
        this.ctx.strokeRect(x, y, width, height);
        this.ctx.setLineDash([]);
        
        this.ctx.fillStyle = this.options.selectedColor + '20'; // Add transparency
        this.ctx.fillRect(x, y, width, height);
    }
    
    private drawEdgePreview(): void {
        if (!this.edgeCreationSource?.bounds || !this.edgePreviewTarget) return;
        
        const sourceCenter = {
            x: this.edgeCreationSource.bounds.x + this.edgeCreationSource.bounds.width / 2,
            y: this.edgeCreationSource.bounds.y + this.edgeCreationSource.bounds.height / 2
        };
        
        this.ctx.strokeStyle = this.options.selectedColor;
        this.ctx.lineWidth = 2;
        this.ctx.setLineDash([5, 5]);
        
        this.ctx.beginPath();
        this.ctx.moveTo(sourceCenter.x, sourceCenter.y);
        this.ctx.lineTo(this.edgePreviewTarget.x, this.edgePreviewTarget.y);
        this.ctx.stroke();
        
        this.ctx.setLineDash([]);
    }

    // Start edge creation from the given element
    public startEdgeCreation(sourceElement: ModelElement, edgeType: string): void {
        this.edgeCreationSource = sourceElement;
        this.edgeCreationType = edgeType;
        console.log('Started edge creation from:', sourceElement.id, 'Type:', edgeType);
    }
    
    // Set interaction mode (pan, select, etc.)
    public setInteractionMode(mode: string): void {
        if (mode === 'pan') {
            this.modeManager.setMode(InteractionMode.Pan);
            this.canvas.style.cursor = 'grab';
        } else if (mode === 'select') {
            this.modeManager.setMode(InteractionMode.Select);
            this.canvas.style.cursor = 'default';
        } else if (mode === 'create-node') {
            this.modeManager.setMode(InteractionMode.CreateNode);
            this.canvas.style.cursor = 'crosshair';
        } else if (mode === 'create-edge') {
            this.modeManager.setMode(InteractionMode.CreateEdge);
            this.canvas.style.cursor = 'crosshair';
        }
    }

    // Set the MCP client for component status checking
    setMcpClient(client: McpClient): void {
        this.mcpClient = client;
    }

    // Check if a component's file is missing (async, but we'll cache results)
    private isComponentMissingFile(node: Node): boolean {
        // For now, return false and implement async checking later
        // TODO: Implement async component status checking via MCP
        return false;
    }

    // Get missing component info for UI via MCP
    async getMissingComponents(): Promise<Array<{ path: string; component: any; removedAt: number }>> {
        if (!this.mcpClient) return [];
        
        try {
            const missingResource = await this.mcpClient.readResource('wasm://components/missing');
            const data = JSON.parse(missingResource.text || '{}');
            return data.missingComponents || [];
        } catch (error) {
            console.warn('Failed to get missing components:', error);
            return [];
        }
    }

    // Handle component restoration or permanent removal via MCP
    async handleMissingComponent(componentName: string, action: 'restore' | 'remove'): Promise<boolean> {
        if (!this.mcpClient) return false;
        
        if (action === 'remove') {
            try {
                await this.mcpClient.callTool('remove_missing_component', {
                    componentName: componentName
                });
                return true;
            } catch (error) {
                console.error('Failed to remove missing component:', error);
                return false;
            }
        }
        
        // For restore, show the expected file location
        try {
            const result = await this.mcpClient.callTool('check_wasm_component_status', {
                componentName: componentName
            });
            const status = JSON.parse(result.content[0]?.text || '{}');
            if (status.path) {
                console.log(`To restore component "${componentName}", place the WASM file at: ${status.path}`);
                return true;
            }
        } catch (error) {
            console.error('Failed to get component status:', error);
        }
        
        return false;
    }

    private interfaceTooltip?: HTMLElement;

    private getInterfaceConnectorAt(position: Position): {
        element: ModelElement;
        interface: any;
        side: 'left' | 'right';
        connectorPosition: Position;
    } | undefined {
        if (!this.currentDiagram) return undefined;

        // Check each WASM component for interface connectors using V2 renderer logic
        for (const element of Object.values(this.currentDiagram.elements)) {
            if (!element.bounds) continue;
            
            const nodeType = element.type || element.element_type || '';
            if (!this.isWasmComponentType(nodeType) || nodeType === 'import-interface' || nodeType === 'export-interface') {
                continue;
            }

            // Use the V2 renderer's port detection method
            const portInfo = WasmComponentRendererV2.getPortAtPosition(element, element.bounds, position);
            if (portInfo) {
                // Calculate the actual connector position for the found port
                const interfaces = element.properties?.interfaces as any[] || [];
                const inputs = interfaces.filter(i => 
                    i.interface_type === 'import' || i.type === 'import' || i.direction === 'input'
                );
                const outputs = interfaces.filter(i => 
                    i.interface_type === 'export' || i.type === 'export' || i.direction === 'output'
                );

                const isInput = portInfo.type === 'input';
                const portArray = isInput ? inputs : outputs;
                const portIndex = portArray.findIndex(p => p === portInfo.port);
                
                if (portIndex >= 0) {
                    // Calculate position using V2 renderer constants
                    const headerHeight = 40; // V2 HEADER_HEIGHT
                    const portSpacing = 24;  // V2 PORT_SPACING
                    const startY = element.bounds.y + headerHeight + 20;
                    
                    const x = isInput ? element.bounds.x : element.bounds.x + element.bounds.width;
                    const y = startY + (portIndex * portSpacing);

                    return {
                        element,
                        interface: portInfo.port,
                        side: isInput ? 'left' : 'right',
                        connectorPosition: { x, y }
                    };
                }
            }
        }

        return undefined;
    }

    private showInterfaceTooltip(
        connector: {
            element: ModelElement;
            interface: any;
            side: 'left' | 'right';
            connectorPosition: Position;
        },
        mousePosition: Position
    ): void {
        // Remove existing tooltip
        this.hideInterfaceTooltip();

        // Create tooltip element
        this.interfaceTooltip = document.createElement('div');
        this.interfaceTooltip.className = 'interface-tooltip';
        this.interfaceTooltip.innerHTML = `
            <div class="tooltip-header">
                <span class="tooltip-icon">${connector.side === 'left' ? '🔵' : '🟢'}</span>
                <span class="tooltip-title">${connector.interface.name}</span>
            </div>
            <div class="tooltip-type">${connector.side === 'left' ? 'Import' : 'Export'} Interface</div>
            ${connector.interface.functions && connector.interface.functions.length > 0 ? 
                `<div class="tooltip-functions">${connector.interface.functions.length} function(s)</div>` : 
                ''
            }
        `;

        // Style the tooltip
        Object.assign(this.interfaceTooltip.style, {
            position: 'fixed',
            background: 'rgba(0, 0, 0, 0.9)',
            color: 'white',
            padding: '8px 12px',
            borderRadius: '6px',
            fontSize: '12px',
            fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
            zIndex: '10000',
            pointerEvents: 'none',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
            maxWidth: '200px',
            wordWrap: 'break-word'
        });

        // Position tooltip near mouse but avoid edges
        const canvasRect = this.canvas.getBoundingClientRect();
        const tooltipX = canvasRect.left + mousePosition.x + 10;
        const tooltipY = canvasRect.top + mousePosition.y - 10;

        this.interfaceTooltip.style.left = tooltipX + 'px';
        this.interfaceTooltip.style.top = tooltipY + 'px';

        // Add to document
        document.body.appendChild(this.interfaceTooltip);
    }

    private hideInterfaceTooltip(): void {
        if (this.interfaceTooltip) {
            document.body.removeChild(this.interfaceTooltip);
            this.interfaceTooltip = undefined;
        }
    }

    private drawEmptyState(): void {
        const centerX = this.canvas.clientWidth / 2;
        const centerY = this.canvas.clientHeight / 2;
        
        this.ctx.save();
        
        // Draw message
        this.ctx.font = '18px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif';
        this.ctx.fillStyle = this.options.textColor;
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';
        
        this.ctx.fillText('No diagram loaded', centerX, centerY - 20);
        
        this.ctx.font = '14px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif';
        this.ctx.fillStyle = this.options.textColor + '80'; // Add transparency
        this.ctx.fillText('Select or create a diagram to get started', centerX, centerY + 10);
        
        // Draw decorative icon
        this.ctx.font = '48px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif';
        this.ctx.fillStyle = this.options.textColor + '40'; // More transparent
        this.ctx.fillText('📊', centerX, centerY - 80);
        
        this.ctx.restore();
    }
}