/**
 * WASM Component Renderer V2 - Clean n8n/Langflow Style
 * Simplified, professional appearance with clear connection points
 */

import { Bounds, ModelElement } from '../model/diagram.js';

export interface WasmRenderingContextV2 {
    ctx: CanvasRenderingContext2D;
    scale: number;
    isSelected: boolean;
    isHovered: boolean;
    isMissing?: boolean;
    colors: {
        primary: string;      // Main component color
        secondary: string;    // Header/accent color
        input: string;        // Input port color
        output: string;       // Output port color
        selected: string;     // Selection border
        text: string;         // Text color
        textSecondary: string; // Secondary text
        background: string;   // Background
        border: string;       // Border
        status: {
            loaded: string;
            unloaded: string;
            error: string;
        };
    };
}

export class WasmComponentRendererV2 {
    // Standard dimensions for consistency
    private static readonly DEFAULT_WIDTH = 200;
    private static readonly DEFAULT_HEIGHT = 120;
    private static readonly HEADER_HEIGHT = 40;
    private static readonly PORT_RADIUS = 8;
    private static readonly PORT_SPACING = 24;
    private static readonly CORNER_RADIUS = 12;

    static renderWasmComponent(
        element: ModelElement,
        bounds: Bounds,
        context: WasmRenderingContextV2
    ): void {
        const { ctx, isSelected, isHovered, isMissing, colors } = context;

        // Get component properties
        // First check the label field (set by MCP create_node), then fall back to other properties
        const componentName = element.label?.toString() || 
                            element.properties?.label?.toString() || 
                            element.properties?.componentName?.toString() || 
                            'Component';
        const componentType = element.properties?.componentType?.toString() || 'WASM';
        const isLoaded = element.properties?.isLoaded === true;
        const interfaces = element.properties?.interfaces as any[] || [];
        const status = isMissing ? 'error' : (isLoaded ? 'loaded' : 'unloaded');
        
        // Debug interface data
        console.log('WasmComponentRendererV2: Rendering component', componentName);
        console.log('WasmComponentRendererV2: Element properties:', element.properties);
        console.log('WasmComponentRendererV2: Interfaces:', interfaces);
        console.log('WasmComponentRendererV2: Interface count:', interfaces.length);

        // Use consistent sizing
        const width = Math.max(bounds.width, this.DEFAULT_WIDTH);
        const height = Math.max(bounds.height, this.DEFAULT_HEIGHT);
        
        // Center the component if bounds are larger than needed
        const x = bounds.x + (bounds.width - width) / 2;
        const y = bounds.y + (bounds.height - height) / 2;
        
        const actualBounds = { x, y, width, height };

        // Draw main component body
        this.drawComponentBody(ctx, actualBounds, colors, isSelected, isHovered, status);
        
        // Draw header section
        this.drawComponentHeader(ctx, actualBounds, componentName, componentType, colors, status);
        
        // Draw status indicator
        this.drawStatusIndicator(ctx, actualBounds, status, colors);
        
        // Draw input/output ports
        this.drawPorts(ctx, actualBounds, interfaces, colors, isSelected);
        
        // Draw missing indicator if needed
        if (isMissing) {
            this.drawErrorOverlay(ctx, actualBounds, colors);
        }
    }

    private static drawComponentBody(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        colors: any,
        isSelected: boolean,
        isHovered: boolean,
        status: string
    ): void {
        // Main body with subtle gradient
        const gradient = ctx.createLinearGradient(bounds.x, bounds.y, bounds.x, bounds.y + bounds.height);
        gradient.addColorStop(0, colors.background);
        gradient.addColorStop(1, this.adjustBrightness(colors.background, -5));

        ctx.fillStyle = gradient;
        ctx.strokeStyle = isSelected ? colors.selected : (isHovered ? colors.primary : colors.border);
        ctx.lineWidth = isSelected ? 3 : (isHovered ? 2 : 1);

        this.drawRoundedRect(ctx, bounds.x, bounds.y, bounds.width, bounds.height, this.CORNER_RADIUS);
        ctx.fill();
        ctx.stroke();

        // Add subtle inner shadow for depth
        if (!isSelected && !isHovered) {
            ctx.strokeStyle = 'rgba(0, 0, 0, 0.08)';
            ctx.lineWidth = 1;
            this.drawRoundedRect(ctx, bounds.x + 1, bounds.y + 1, bounds.width - 2, bounds.height - 2, this.CORNER_RADIUS - 1);
            ctx.stroke();
        }
    }

    private static drawComponentHeader(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        componentName: string,
        componentType: string,
        colors: any,
        status: string
    ): void {
        // Header background with component color
        const headerGradient = ctx.createLinearGradient(
            bounds.x, bounds.y, 
            bounds.x, bounds.y + this.HEADER_HEIGHT
        );
        headerGradient.addColorStop(0, colors.primary);
        headerGradient.addColorStop(1, this.adjustBrightness(colors.primary, -10));

        ctx.fillStyle = headerGradient;
        this.drawRoundedRect(
            ctx, 
            bounds.x + 1, 
            bounds.y + 1, 
            bounds.width - 2, 
            this.HEADER_HEIGHT, 
            this.CORNER_RADIUS - 1, 
            true // top corners only
        );
        ctx.fill();

        // Component icon/logo area (left side of header)
        const iconSize = 24;
        const iconX = bounds.x + 12;
        const iconY = bounds.y + (this.HEADER_HEIGHT - iconSize) / 2;
        
        this.drawComponentIcon(ctx, iconX, iconY, iconSize, componentType, colors);

        // Component name (center-left of header)
        ctx.fillStyle = 'white';
        ctx.font = 'bold 14px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'middle';
        
        const textX = iconX + iconSize + 8;
        const textY = bounds.y + this.HEADER_HEIGHT / 2;
        
        // Truncate long names
        const maxTextWidth = bounds.width - textX - bounds.x - 50; // Leave space for type badge
        const truncatedName = this.truncateText(ctx, componentName, maxTextWidth);
        ctx.fillText(truncatedName, textX, textY);

        // Component type badge (right side of header)
        this.drawTypeBadge(ctx, bounds, componentType, colors);
    }

    private static drawComponentIcon(
        ctx: CanvasRenderingContext2D,
        x: number,
        y: number,
        size: number,
        componentType: string,
        colors: any
    ): void {
        // Simple geometric icon based on component type
        ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.7)';
        ctx.lineWidth = 2;

        const centerX = x + size / 2;
        const centerY = y + size / 2;

        if (componentType.toLowerCase().includes('ai') || componentType.toLowerCase().includes('ml')) {
            // Brain/neural network icon for AI components
            this.drawNeuralIcon(ctx, centerX, centerY, size * 0.4);
        } else if (componentType.toLowerCase().includes('sensor')) {
            // Sensor icon
            this.drawSensorIcon(ctx, centerX, centerY, size * 0.4);
        } else if (componentType.toLowerCase().includes('ecu') || componentType.toLowerCase().includes('control')) {
            // Control unit icon
            this.drawControlIcon(ctx, centerX, centerY, size * 0.4);
        } else {
            // Default WASM cube icon
            this.drawCubeIcon(ctx, centerX, centerY, size * 0.4);
        }
    }

    private static drawStatusIndicator(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        status: string,
        colors: any
    ): void {
        const indicatorSize = 8;
        const x = bounds.x + bounds.width - indicatorSize - 8;
        const y = bounds.y + this.HEADER_HEIGHT + 8;

        // Status dot
        ctx.fillStyle = colors.status[status as keyof typeof colors.status];
        ctx.beginPath();
        ctx.arc(x, y, indicatorSize / 2, 0, 2 * Math.PI);
        ctx.fill();

        // Status text
        ctx.fillStyle = colors.textSecondary;
        ctx.font = '10px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        ctx.textAlign = 'right';
        ctx.textBaseline = 'middle';
        
        const statusText = status === 'loaded' ? 'Active' : status === 'error' ? 'Error' : 'Inactive';
        ctx.fillText(statusText, x - indicatorSize, y);
    }

    private static drawPorts(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        interfaces: any[],
        colors: any,
        isSelected: boolean
    ): void {
        console.log('WasmComponentRendererV2: drawPorts called with', interfaces.length, 'interfaces');
        console.log('WasmComponentRendererV2: Interface details:', interfaces);
        
        // Separate input and output interfaces
        const inputs = interfaces.filter(i => 
            i.interface_type === 'import' || i.type === 'import' || i.direction === 'input'
        );
        const outputs = interfaces.filter(i => 
            i.interface_type === 'export' || i.type === 'export' || i.direction === 'output'
        );
        
        console.log('WasmComponentRendererV2: Filtered inputs:', inputs.length, inputs);
        console.log('WasmComponentRendererV2: Filtered outputs:', outputs.length, outputs);

        // Draw input ports (left side)
        this.drawPortGroup(ctx, bounds, inputs, 'input', colors, isSelected);
        
        // Draw output ports (right side)
        this.drawPortGroup(ctx, bounds, outputs, 'output', colors, isSelected);
    }

    private static drawPortGroup(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        ports: any[],
        type: 'input' | 'output',
        colors: any,
        isSelected: boolean
    ): void {
        if (ports.length === 0) return;

        const isInput = type === 'input';
        const portColor = isInput ? colors.input : colors.output;
        
        // Calculate port positions
        const startY = bounds.y + this.HEADER_HEIGHT + 20;
        const availableHeight = bounds.height - this.HEADER_HEIGHT - 40;
        const spacing = Math.min(this.PORT_SPACING, availableHeight / Math.max(ports.length, 1));

        ports.forEach((port, index) => {
            const x = isInput ? bounds.x : bounds.x + bounds.width;
            const y = startY + (index * spacing);

            // Draw port circle
            ctx.fillStyle = portColor;
            ctx.strokeStyle = isSelected ? colors.selected : 'white';
            ctx.lineWidth = 2;

            ctx.beginPath();
            ctx.arc(x, y, this.PORT_RADIUS, 0, 2 * Math.PI);
            ctx.fill();
            ctx.stroke();

            // Add connection hint
            ctx.fillStyle = 'white';
            ctx.beginPath();
            ctx.arc(x, y, this.PORT_RADIUS - 3, 0, 2 * Math.PI);
            ctx.fill();

            // Port label (on hover or selection)
            if (isSelected) {
                this.drawPortLabel(ctx, port.name || `${type}-${index}`, x, y, isInput, colors);
            }
        });
    }

    private static drawPortLabel(
        ctx: CanvasRenderingContext2D,
        label: string,
        x: number,
        y: number,
        isInput: boolean,
        colors: any
    ): void {
        ctx.font = '10px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        ctx.textBaseline = 'middle';
        
        const metrics = ctx.measureText(label);
        const padding = 6;
        const labelWidth = metrics.width + padding * 2;
        const labelHeight = 18;
        
        const labelX = isInput ? x - this.PORT_RADIUS - 8 - labelWidth : x + this.PORT_RADIUS + 8;
        
        // Label background
        ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
        this.drawRoundedRect(ctx, labelX, y - labelHeight/2, labelWidth, labelHeight, 4);
        ctx.fill();
        
        // Label text
        ctx.fillStyle = 'white';
        ctx.textAlign = isInput ? 'right' : 'left';
        ctx.fillText(label, labelX + (isInput ? labelWidth - padding : padding), y);
    }

    private static drawTypeBadge(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        componentType: string,
        colors: any
    ): void {
        const badgeWidth = 45;
        const badgeHeight = 16;
        const badgeX = bounds.x + bounds.width - badgeWidth - 8;
        const badgeY = bounds.y + (this.HEADER_HEIGHT - badgeHeight) / 2;

        // Badge background
        ctx.fillStyle = 'rgba(255, 255, 255, 0.2)';
        this.drawRoundedRect(ctx, badgeX, badgeY, badgeWidth, badgeHeight, 8);
        ctx.fill();

        // Badge text
        ctx.fillStyle = 'white';
        ctx.font = 'bold 9px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        
        const badgeText = componentType.toUpperCase().substring(0, 6);
        ctx.fillText(badgeText, badgeX + badgeWidth / 2, badgeY + badgeHeight / 2);
    }

    private static drawErrorOverlay(
        ctx: CanvasRenderingContext2D,
        bounds: Bounds,
        colors: any
    ): void {
        // Semi-transparent red overlay
        ctx.fillStyle = 'rgba(244, 67, 54, 0.1)';
        this.drawRoundedRect(ctx, bounds.x + 1, bounds.y + 1, bounds.width - 2, bounds.height - 2, this.CORNER_RADIUS - 1);
        ctx.fill();

        // Error icon in bottom-right
        const iconSize = 16;
        const iconX = bounds.x + bounds.width - iconSize - 8;
        const iconY = bounds.y + bounds.height - iconSize - 8;

        ctx.fillStyle = colors.status.error;
        ctx.beginPath();
        ctx.arc(iconX + iconSize/2, iconY + iconSize/2, iconSize/2, 0, 2 * Math.PI);
        ctx.fill();

        ctx.fillStyle = 'white';
        ctx.font = 'bold 10px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('!', iconX + iconSize/2, iconY + iconSize/2);
    }

    // Icon drawing helpers
    private static drawCubeIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number): void {
        const s = size;
        ctx.beginPath();
        // Simple isometric cube
        ctx.moveTo(x - s/2, y);
        ctx.lineTo(x, y - s/2);
        ctx.lineTo(x + s/2, y);
        ctx.lineTo(x, y + s/2);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
    }

    private static drawNeuralIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number): void {
        // Simple neural network representation
        const nodeRadius = size / 6;
        const nodes = [
            { x: x - size/3, y: y - size/4 },
            { x: x - size/3, y: y + size/4 },
            { x: x + size/3, y: y }
        ];

        // Draw connections
        ctx.beginPath();
        nodes.forEach((node, i) => {
            nodes.forEach((other, j) => {
                if (i !== j) {
                    ctx.moveTo(node.x, node.y);
                    ctx.lineTo(other.x, other.y);
                }
            });
        });
        ctx.stroke();

        // Draw nodes
        nodes.forEach(node => {
            ctx.beginPath();
            ctx.arc(node.x, node.y, nodeRadius, 0, 2 * Math.PI);
            ctx.fill();
        });
    }

    private static drawSensorIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number): void {
        // Radar/sensor waves
        const waveCount = 3;
        for (let i = 0; i < waveCount; i++) {
            const radius = (size / 2) * (i + 1) / waveCount;
            ctx.beginPath();
            ctx.arc(x, y, radius, -Math.PI/3, Math.PI/3);
            ctx.stroke();
        }
    }

    private static drawControlIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number): void {
        // Circuit board pattern
        const s = size / 3;
        ctx.beginPath();
        ctx.rect(x - s, y - s, s*2, s*2);
        ctx.moveTo(x - s, y);
        ctx.lineTo(x + s, y);
        ctx.moveTo(x, y - s);
        ctx.lineTo(x, y + s);
        ctx.stroke();
        
        // Corner nodes
        [-s, s].forEach(dx => {
            [-s, s].forEach(dy => {
                ctx.beginPath();
                ctx.arc(x + dx, y + dy, s/4, 0, 2 * Math.PI);
                ctx.fill();
            });
        });
    }

    // Utility methods
    private static drawRoundedRect(
        ctx: CanvasRenderingContext2D,
        x: number,
        y: number,
        width: number,
        height: number,
        radius: number,
        topOnly: boolean = false
    ): void {
        ctx.beginPath();
        ctx.moveTo(x + radius, y);
        ctx.lineTo(x + width - radius, y);
        ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
        
        if (topOnly) {
            ctx.lineTo(x + width, y + height);
            ctx.lineTo(x, y + height);
            ctx.lineTo(x, y + radius);
        } else {
            ctx.lineTo(x + width, y + height - radius);
            ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
            ctx.lineTo(x + radius, y + height);
            ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
            ctx.lineTo(x, y + radius);
        }
        
        ctx.quadraticCurveTo(x, y, x + radius, y);
        ctx.closePath();
    }

    private static adjustBrightness(color: string, percent: number): string {
        // Simple brightness adjustment for hex colors
        if (color.startsWith('#')) {
            const num = parseInt(color.slice(1), 16);
            const amt = Math.round(2.55 * percent);
            const R = (num >> 16) + amt;
            const G = (num >> 8 & 0x00FF) + amt;
            const B = (num & 0x0000FF) + amt;
            return `#${(0x1000000 + (R < 255 ? R < 1 ? 0 : R : 255) * 0x10000 +
                (G < 255 ? G < 1 ? 0 : G : 255) * 0x100 +
                (B < 255 ? B < 1 ? 0 : B : 255)).toString(16).slice(1)}`;
        }
        return color;
    }

    private static truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
        if (ctx.measureText(text).width <= maxWidth) return text;
        
        let truncated = text;
        while (ctx.measureText(truncated + '...').width > maxWidth && truncated.length > 0) {
            truncated = truncated.slice(0, -1);
        }
        return truncated + (truncated.length < text.length ? '...' : '');
    }

    // Default color scheme inspired by modern workflow tools
    static getDefaultColors() {
        return {
            primary: '#4F46E5',        // Modern purple-blue
            secondary: '#6366F1',      // Lighter purple
            input: '#10B981',          // Green for inputs
            output: '#F59E0B',         // Orange for outputs
            selected: '#EC4899',       // Pink for selection
            text: '#1F2937',          // Dark gray text
            textSecondary: '#6B7280',  // Light gray text
            background: '#FFFFFF',     // White background
            border: '#E5E7EB',        // Light border
            status: {
                loaded: '#10B981',     // Green
                unloaded: '#6B7280',   // Gray
                error: '#EF4444'       // Red
            }
        };
    }

    // Check if a position is within a port area (for connection logic)
    static getPortAtPosition(
        element: ModelElement,
        bounds: Bounds,
        position: { x: number; y: number }
    ): { port: any; type: 'input' | 'output' } | null {
        const interfaces = element.properties?.interfaces as any[] || [];
        const inputs = interfaces.filter(i => 
            i.interface_type === 'import' || i.type === 'import' || i.direction === 'input'
        );
        const outputs = interfaces.filter(i => 
            i.interface_type === 'export' || i.type === 'export' || i.direction === 'output'
        );

        // Check input ports
        const startY = bounds.y + this.HEADER_HEIGHT + 20;
        const availableHeight = bounds.height - this.HEADER_HEIGHT - 40;
        const spacing = Math.min(this.PORT_SPACING, availableHeight / Math.max(inputs.length || outputs.length, 1));

        // Check inputs (left side)
        for (let i = 0; i < inputs.length; i++) {
            const portX = bounds.x;
            const portY = startY + (i * spacing);
            const distance = Math.sqrt(Math.pow(position.x - portX, 2) + Math.pow(position.y - portY, 2));
            
            if (distance <= this.PORT_RADIUS + 4) { // Small tolerance
                return { port: inputs[i], type: 'input' };
            }
        }

        // Check outputs (right side)
        for (let i = 0; i < outputs.length; i++) {
            const portX = bounds.x + bounds.width;
            const portY = startY + (i * spacing);
            const distance = Math.sqrt(Math.pow(position.x - portX, 2) + Math.pow(position.y - portY, 2));
            
            if (distance <= this.PORT_RADIUS + 4) { // Small tolerance
                return { port: outputs[i], type: 'output' };
            }
        }

        return null;
    }
}