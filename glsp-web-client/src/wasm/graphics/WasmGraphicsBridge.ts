import { CanvasRenderer } from '../../renderer/canvas-renderer.js';

export interface GraphicsContext {
    width: number;
    height: number;
    pixelRatio: number;
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
}

export interface DrawCommand {
    type: 'rect' | 'circle' | 'line' | 'path' | 'text' | 'image' | 'clear';
    params: any;
    style?: DrawStyle;
}

export interface DrawStyle {
    fillColor?: string;
    strokeColor?: string;
    lineWidth?: number;
    lineDash?: number[];
    font?: string;
    textAlign?: CanvasTextAlign;
    textBaseline?: CanvasTextBaseline;
    globalAlpha?: number;
    shadowColor?: string;
    shadowBlur?: number;
    shadowOffsetX?: number;
    shadowOffsetY?: number;
}

export interface GraphicsAPI {
    // Basic shapes
    drawRect(x: number, y: number, width: number, height: number, style?: DrawStyle): void;
    drawCircle(x: number, y: number, radius: number, style?: DrawStyle): void;
    drawLine(x1: number, y1: number, x2: number, y2: number, style?: DrawStyle): void;
    drawPath(points: Array<{x: number, y: number}>, closed?: boolean, style?: DrawStyle): void;
    
    // Text
    drawText(text: string, x: number, y: number, style?: DrawStyle): void;
    measureText(text: string, font?: string): { width: number; height: number };
    
    // Images
    drawImage(imageData: ImageData | HTMLImageElement, x: number, y: number, width?: number, height?: number): void;
    createImageData(width: number, height: number): ImageData;
    getImageData(x: number, y: number, width: number, height: number): ImageData;
    
    // Transformations
    save(): void;
    restore(): void;
    translate(x: number, y: number): void;
    rotate(angle: number): void;
    scale(x: number, y: number): void;
    
    // Canvas operations
    clear(x?: number, y?: number, width?: number, height?: number): void;
    getContext(): GraphicsContext;
    
    // Batch operations
    beginBatch(): void;
    endBatch(): void;
    executeBatch(commands: DrawCommand[]): void;
}

export class WasmGraphicsBridge implements GraphicsAPI {
    private context: GraphicsContext;
    private commandBuffer: DrawCommand[] = [];
    private isBatching: boolean = false;
    private savedStates: any[] = [];
    private renderer?: CanvasRenderer;
    
    constructor(canvas: HTMLCanvasElement, renderer?: CanvasRenderer) {
        const ctx = canvas.getContext('2d');
        if (!ctx) {
            throw new Error('Failed to get 2D context from canvas');
        }
        
        this.context = {
            width: canvas.width,
            height: canvas.height,
            pixelRatio: window.devicePixelRatio || 1,
            canvas,
            ctx
        };
        
        this.renderer = renderer;
        this.setupCanvas();
    }
    
    private setupCanvas(): void {
        const { ctx, pixelRatio } = this.context;
        
        // Enable anti-aliasing
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';
        
        // Set default styles
        ctx.lineJoin = 'round';
        ctx.lineCap = 'round';
    }
    
    // Basic shapes
    drawRect(x: number, y: number, width: number, height: number, style?: DrawStyle): void {
        const command: DrawCommand = {
            type: 'rect',
            params: { x, y, width, height },
            style
        };
        
        if (this.isBatching) {
            this.commandBuffer.push(command);
        } else {
            this.executeCommand(command);
        }
    }
    
    drawCircle(x: number, y: number, radius: number, style?: DrawStyle): void {
        const command: DrawCommand = {
            type: 'circle',
            params: { x, y, radius },
            style
        };
        
        if (this.isBatching) {
            this.commandBuffer.push(command);
        } else {
            this.executeCommand(command);
        }
    }
    
    drawLine(x1: number, y1: number, x2: number, y2: number, style?: DrawStyle): void {
        const command: DrawCommand = {
            type: 'line',
            params: { x1, y1, x2, y2 },
            style
        };
        
        if (this.isBatching) {
            this.commandBuffer.push(command);
        } else {
            this.executeCommand(command);
        }
    }
    
    drawPath(points: Array<{x: number, y: number}>, closed: boolean = false, style?: DrawStyle): void {
        const command: DrawCommand = {
            type: 'path',
            params: { points, closed },
            style
        };
        
        if (this.isBatching) {
            this.commandBuffer.push(command);
        } else {
            this.executeCommand(command);
        }
    }
    
    // Text
    drawText(text: string, x: number, y: number, style?: DrawStyle): void {
        const command: DrawCommand = {
            type: 'text',
            params: { text, x, y },
            style
        };
        
        if (this.isBatching) {
            this.commandBuffer.push(command);
        } else {
            this.executeCommand(command);
        }
    }
    
    measureText(text: string, font?: string): { width: number; height: number } {
        const ctx = this.context.ctx;
        
        if (font) {
            const oldFont = ctx.font;
            ctx.font = font;
            const metrics = ctx.measureText(text);
            ctx.font = oldFont;
            
            return {
                width: metrics.width,
                height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent
            };
        }
        
        const metrics = ctx.measureText(text);
        return {
            width: metrics.width,
            height: metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent
        };
    }
    
    // Images
    drawImage(imageData: ImageData | HTMLImageElement, x: number, y: number, width?: number, height?: number): void {
        const ctx = this.context.ctx;
        
        if (imageData instanceof ImageData) {
            ctx.putImageData(imageData, x, y);
        } else {
            if (width !== undefined && height !== undefined) {
                ctx.drawImage(imageData, x, y, width, height);
            } else {
                ctx.drawImage(imageData, x, y);
            }
        }
    }
    
    createImageData(width: number, height: number): ImageData {
        return this.context.ctx.createImageData(width, height);
    }
    
    getImageData(x: number, y: number, width: number, height: number): ImageData {
        return this.context.ctx.getImageData(x, y, width, height);
    }
    
    // Transformations
    save(): void {
        this.context.ctx.save();
        this.savedStates.push({
            transform: this.context.ctx.getTransform()
        });
    }
    
    restore(): void {
        this.context.ctx.restore();
        this.savedStates.pop();
    }
    
    translate(x: number, y: number): void {
        this.context.ctx.translate(x, y);
    }
    
    rotate(angle: number): void {
        this.context.ctx.rotate(angle);
    }
    
    scale(x: number, y: number): void {
        this.context.ctx.scale(x, y);
    }
    
    // Canvas operations
    clear(x?: number, y?: number, width?: number, height?: number): void {
        const ctx = this.context.ctx;
        
        if (x !== undefined && y !== undefined && width !== undefined && height !== undefined) {
            ctx.clearRect(x, y, width, height);
        } else {
            ctx.clearRect(0, 0, this.context.width, this.context.height);
        }
    }
    
    getContext(): GraphicsContext {
        return { ...this.context };
    }
    
    // Batch operations
    beginBatch(): void {
        this.isBatching = true;
        this.commandBuffer = [];
    }
    
    endBatch(): void {
        if (!this.isBatching) return;
        
        this.isBatching = false;
        this.executeBatch(this.commandBuffer);
        this.commandBuffer = [];
    }
    
    executeBatch(commands: DrawCommand[]): void {
        // Save context state
        this.context.ctx.save();
        
        // Execute all commands
        commands.forEach(command => this.executeCommand(command));
        
        // Restore context state
        this.context.ctx.restore();
    }
    
    // Private execution methods
    private executeCommand(command: DrawCommand): void {
        const ctx = this.context.ctx;
        
        // Apply style if provided
        if (command.style) {
            this.applyStyle(command.style);
        }
        
        switch (command.type) {
            case 'rect':
                this.executeRect(command.params);
                break;
            case 'circle':
                this.executeCircle(command.params);
                break;
            case 'line':
                this.executeLine(command.params);
                break;
            case 'path':
                this.executePath(command.params);
                break;
            case 'text':
                this.executeText(command.params);
                break;
            case 'clear':
                this.clear(command.params.x, command.params.y, command.params.width, command.params.height);
                break;
        }
    }
    
    private applyStyle(style: DrawStyle): void {
        const ctx = this.context.ctx;
        
        if (style.fillColor !== undefined) ctx.fillStyle = style.fillColor;
        if (style.strokeColor !== undefined) ctx.strokeStyle = style.strokeColor;
        if (style.lineWidth !== undefined) ctx.lineWidth = style.lineWidth;
        if (style.lineDash !== undefined) ctx.setLineDash(style.lineDash);
        if (style.font !== undefined) ctx.font = style.font;
        if (style.textAlign !== undefined) ctx.textAlign = style.textAlign;
        if (style.textBaseline !== undefined) ctx.textBaseline = style.textBaseline;
        if (style.globalAlpha !== undefined) ctx.globalAlpha = style.globalAlpha;
        if (style.shadowColor !== undefined) ctx.shadowColor = style.shadowColor;
        if (style.shadowBlur !== undefined) ctx.shadowBlur = style.shadowBlur;
        if (style.shadowOffsetX !== undefined) ctx.shadowOffsetX = style.shadowOffsetX;
        if (style.shadowOffsetY !== undefined) ctx.shadowOffsetY = style.shadowOffsetY;
    }
    
    private executeRect(params: any): void {
        const ctx = this.context.ctx;
        const { x, y, width, height } = params;
        
        ctx.beginPath();
        ctx.rect(x, y, width, height);
        
        if (ctx.fillStyle) ctx.fill();
        if (ctx.strokeStyle) ctx.stroke();
    }
    
    private executeCircle(params: any): void {
        const ctx = this.context.ctx;
        const { x, y, radius } = params;
        
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, Math.PI * 2);
        
        if (ctx.fillStyle) ctx.fill();
        if (ctx.strokeStyle) ctx.stroke();
    }
    
    private executeLine(params: any): void {
        const ctx = this.context.ctx;
        const { x1, y1, x2, y2 } = params;
        
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
    }
    
    private executePath(params: any): void {
        const ctx = this.context.ctx;
        const { points, closed } = params;
        
        if (points.length < 2) return;
        
        ctx.beginPath();
        ctx.moveTo(points[0].x, points[0].y);
        
        for (let i = 1; i < points.length; i++) {
            ctx.lineTo(points[i].x, points[i].y);
        }
        
        if (closed) {
            ctx.closePath();
        }
        
        if (ctx.fillStyle && closed) ctx.fill();
        if (ctx.strokeStyle) ctx.stroke();
    }
    
    private executeText(params: any): void {
        const ctx = this.context.ctx;
        const { text, x, y } = params;
        
        if (ctx.fillStyle) {
            ctx.fillText(text, x, y);
        }
        if (ctx.strokeStyle) {
            ctx.strokeText(text, x, y);
        }
    }
    
    // Integration with renderer
    public integrateWithRenderer(renderer: CanvasRenderer): void {
        this.renderer = renderer;
    }
    
    // Create offscreen canvas for WASM components
    public createOffscreenCanvas(width: number, height: number): OffscreenCanvas | HTMLCanvasElement {
        if (typeof OffscreenCanvas !== 'undefined') {
            return new OffscreenCanvas(width, height);
        } else {
            // Fallback for browsers without OffscreenCanvas
            const canvas = document.createElement('canvas');
            canvas.width = width;
            canvas.height = height;
            return canvas;
        }
    }
    
    // Export canvas content
    public exportCanvas(format: 'png' | 'jpeg' | 'webp' = 'png', quality?: number): Promise<Blob | null> {
        return new Promise((resolve) => {
            this.context.canvas.toBlob(
                (blob) => resolve(blob),
                `image/${format}`,
                quality
            );
        });
    }
    
    // Resize canvas
    public resizeCanvas(width: number, height: number): void {
        const { canvas, ctx, pixelRatio } = this.context;
        
        // Store current content
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        
        // Resize canvas
        canvas.width = width * pixelRatio;
        canvas.height = height * pixelRatio;
        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;
        
        // Update context
        this.context.width = canvas.width;
        this.context.height = canvas.height;
        
        // Scale for high DPI
        ctx.scale(pixelRatio, pixelRatio);
        
        // Restore content (if it fits)
        if (imageData.width <= canvas.width && imageData.height <= canvas.height) {
            ctx.putImageData(imageData, 0, 0);
        }
        
        // Re-apply setup
        this.setupCanvas();
    }
}