/**
 * Canvas-based diagram renderer
 * Renders diagram elements using HTML5 Canvas
 */

import { DiagramModel, ModelElement, Node, Edge, Bounds, Position } from '../model/diagram.js';
import { SelectionManager } from '../selection/selection-manager.js';
import { InteractionMode, InteractionModeManager } from '../interaction/interaction-mode.js';

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
    private selectionRect?: { start: Position; end: Position };
    private isSelectingRect = false;
    private edgeCreationSource?: ModelElement;
    private edgePreviewTarget?: Position;

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
        const rect = this.canvas.getBoundingClientRect();
        this.canvas.width = rect.width * window.devicePixelRatio;
        this.canvas.height = rect.height * window.devicePixelRatio;
        this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
        this.canvas.style.width = rect.width + 'px';
        this.canvas.style.height = rect.height + 'px';
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
            if (element.type === 'graph') continue;
            
            if (element.bounds && this.isPointInBounds(position, element.bounds)) {
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
                if (element && (element.type === 'task' || element.type.includes('event') || element.type === 'gateway')) {
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
            // Handle hover
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
            
            // Update cursor based on mode and hover state
            const mode = this.modeManager.getMode();
            if (mode === InteractionMode.Select && element && this.selectionManager.isSelected(element.id)) {
                this.canvas.style.cursor = 'grab';
            } else if (mode === InteractionMode.CreateNode) {
                this.canvas.style.cursor = 'crosshair';
            } else if (mode === InteractionMode.CreateEdge) {
                this.canvas.style.cursor = element ? 'pointer' : 'default';
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

        if (element && this.selectionManager.isSelected(element.id)) {
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
        
        const scaleFactor = event.deltaY > 0 ? 0.9 : 1.1;
        const newScale = Math.max(0.1, Math.min(5.0, this.options.scale * scaleFactor));
        
        const mousePos = this.getMousePosition(event);
        
        // Zoom towards mouse position
        this.options.offset.x -= (mousePos.x * (newScale - this.options.scale));
        this.options.offset.y -= (mousePos.y * (newScale - this.options.scale));
        
        this.options.scale = newScale;
        this.render();
    }

    setDiagram(diagram: DiagramModel): void {
        this.currentDiagram = diagram;
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

    zoom(scale: number, center?: Position): void {
        if (center) {
            this.options.offset.x -= (center.x * (scale - this.options.scale));
            this.options.offset.y -= (center.y * (scale - this.options.scale));
        }
        this.options.scale = Math.max(0.1, Math.min(5.0, scale));
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

        this.render();
    }

    render(): void {
        if (!this.currentDiagram) return;

        this.ctx.save();
        
        // Clear canvas
        this.ctx.fillStyle = this.options.backgroundColor;
        this.ctx.fillRect(0, 0, this.canvas.clientWidth, this.canvas.clientHeight);

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
            if (element.type === 'graph' || !element.bounds) return;
            
            this.drawNode(element as Node);
        });
    }

    private drawNode(node: Node): void {
        if (!node.bounds) return;

        const isSelected = this.selectionManager.isSelected(node.id);
        const isHovered = this.selectionManager.isHovered(node.id);

        // Draw node background
        this.ctx.fillStyle = isSelected ? this.options.selectedColor : this.options.nodeColor;
        this.ctx.strokeStyle = isHovered ? this.options.selectedColor : this.options.edgeColor;
        this.ctx.lineWidth = isSelected ? 3 : 1;

        // Draw different shapes based on node type
        switch (node.type) {
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
        if (node.label) {
            this.drawLabel(node.label, node.bounds);
        }
    }

    private drawEdges(): void {
        if (!this.currentDiagram) return;

        Object.values(this.currentDiagram.elements).forEach(element => {
            if (!element.type.includes('edge')) return;
            
            this.drawEdge(element as Edge);
        });
    }

    private drawEdge(edge: Edge): void {
        const sourceElement = this.currentDiagram?.elements[edge.sourceId];
        const targetElement = this.currentDiagram?.elements[edge.targetId];

        if (!sourceElement?.bounds || !targetElement?.bounds) return;

        const isSelected = this.selectionManager.isSelected(edge.id);
        const isHovered = this.selectionManager.isHovered(edge.id);

        this.ctx.strokeStyle = isSelected || isHovered ? this.options.selectedColor : this.options.edgeColor;
        this.ctx.lineWidth = isSelected ? 2 : 1;

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
        if (edge.label) {
            const midPoint = {
                x: (sourceCenter.x + targetCenter.x) / 2,
                y: (sourceCenter.y + targetCenter.y) / 2
            };
            this.drawEdgeLabel(edge.label, midPoint);
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
}