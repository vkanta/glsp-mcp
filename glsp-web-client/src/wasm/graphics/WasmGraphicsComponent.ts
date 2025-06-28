import { WasmGraphicsBridge, GraphicsAPI } from './WasmGraphicsBridge.js';

export interface GraphicsComponentConfig {
    width: number;
    height: number;
    backgroundColor?: string;
    antialiasing?: boolean;
    pixelRatio?: number;
}

export interface AnimationFrame {
    time: number;
    deltaTime: number;
    frameCount: number;
}

export interface GraphicsComponent {
    // Lifecycle methods
    initialize(graphics: GraphicsAPI, config: GraphicsComponentConfig): Promise<void>;
    render(graphics: GraphicsAPI, frame: AnimationFrame): void;
    resize(width: number, height: number): void;
    destroy(): void;
    
    // Optional methods
    onMouseMove?(x: number, y: number): void;
    onMouseDown?(x: number, y: number, button: number): void;
    onMouseUp?(x: number, y: number, button: number): void;
    onKeyDown?(key: string, code: string): void;
    onKeyUp?(key: string, code: string): void;
}

export abstract class BaseGraphicsComponent implements GraphicsComponent {
    protected graphics!: GraphicsAPI;
    protected config!: GraphicsComponentConfig;
    protected isInitialized: boolean = false;
    
    async initialize(graphics: GraphicsAPI, config: GraphicsComponentConfig): Promise<void> {
        this.graphics = graphics;
        this.config = config;
        
        // Clear canvas with background color
        if (config.backgroundColor) {
            const ctx = graphics.getContext().ctx;
            ctx.fillStyle = config.backgroundColor;
            ctx.fillRect(0, 0, config.width, config.height);
        }
        
        await this.onInitialize();
        this.isInitialized = true;
    }
    
    abstract render(graphics: GraphicsAPI, frame: AnimationFrame): void;
    
    resize(width: number, height: number): void {
        this.config.width = width;
        this.config.height = height;
        this.onResize(width, height);
    }
    
    destroy(): void {
        this.onDestroy();
        this.isInitialized = false;
    }
    
    // Override these in subclasses
    protected async onInitialize(): Promise<void> {}
    protected onResize(width: number, height: number): void {}
    protected onDestroy(): void {}
}

// Example: Animated sine wave component
export class SineWaveComponent extends BaseGraphicsComponent {
    private amplitude: number = 50;
    private frequency: number = 0.02;
    private speed: number = 0.05;
    private points: number = 100;
    
    render(graphics: GraphicsAPI, frame: AnimationFrame): void {
        const { width, height } = this.config;
        
        // Clear canvas
        graphics.clear();
        
        // Draw background
        graphics.drawRect(0, 0, width, height, {
            fillColor: '#0A0E1A'
        });
        
        // Calculate wave points
        const wavePoints: Array<{x: number, y: number}> = [];
        const centerY = height / 2;
        
        for (let i = 0; i <= this.points; i++) {
            const x = (i / this.points) * width;
            const y = centerY + Math.sin((x * this.frequency) + (frame.time * this.speed)) * this.amplitude;
            wavePoints.push({ x, y });
        }
        
        // Draw wave
        graphics.drawPath(wavePoints, false, {
            strokeColor: '#4A9EFF',
            lineWidth: 3,
            shadowColor: '#4A9EFF',
            shadowBlur: 10
        });
        
        // Draw points
        wavePoints.forEach((point, index) => {
            if (index % 10 === 0) {
                graphics.drawCircle(point.x, point.y, 4, {
                    fillColor: '#00D4AA',
                    shadowColor: '#00D4AA',
                    shadowBlur: 5
                });
            }
        });
        
        // Draw info text
        graphics.drawText(`Frame: ${frame.frameCount}`, 10, 20, {
            fillColor: '#E5E9F0',
            font: '14px monospace'
        });
    }
    
    onMouseMove(x: number, y: number): void {
        // Adjust amplitude based on mouse Y position
        this.amplitude = Math.max(10, Math.min(100, y - this.config.height / 2));
    }
}

// Example: Particle system component
export class ParticleSystemComponent extends BaseGraphicsComponent {
    private particles: Particle[] = [];
    private maxParticles: number = 100;
    private emitRate: number = 2;
    private lastEmitTime: number = 0;
    
    protected async onInitialize(): Promise<void> {
        // Initialize particle system
        this.particles = [];
    }
    
    render(graphics: GraphicsAPI, frame: AnimationFrame): void {
        const { width, height } = this.config;
        
        // Clear with fade effect
        graphics.drawRect(0, 0, width, height, {
            fillColor: 'rgba(10, 14, 26, 0.1)'
        });
        
        // Emit new particles
        if (frame.time - this.lastEmitTime > 1000 / this.emitRate) {
            this.emitParticle(width / 2, height / 2);
            this.lastEmitTime = frame.time;
        }
        
        // Update and draw particles
        graphics.beginBatch();
        
        this.particles = this.particles.filter(particle => {
            // Update particle
            particle.x += particle.vx * frame.deltaTime / 16;
            particle.y += particle.vy * frame.deltaTime / 16;
            particle.life -= frame.deltaTime / 1000;
            
            if (particle.life <= 0) return false;
            
            // Draw particle
            const alpha = particle.life;
            const size = particle.size * (1 + (1 - particle.life) * 0.5);
            
            graphics.drawCircle(particle.x, particle.y, size, {
                fillColor: particle.color,
                globalAlpha: alpha,
                shadowColor: particle.color,
                shadowBlur: size * 2
            });
            
            return true;
        });
        
        graphics.endBatch();
        
        // Draw particle count
        graphics.drawText(`Particles: ${this.particles.length}`, 10, 20, {
            fillColor: '#E5E9F0',
            font: '14px monospace'
        });
    }
    
    private emitParticle(x: number, y: number): void {
        if (this.particles.length >= this.maxParticles) return;
        
        const angle = Math.random() * Math.PI * 2;
        const speed = Math.random() * 2 + 1;
        const colors = ['#4A9EFF', '#00D4AA', '#654FF0', '#F0B72F'];
        
        this.particles.push({
            x,
            y,
            vx: Math.cos(angle) * speed,
            vy: Math.sin(angle) * speed,
            size: Math.random() * 3 + 2,
            color: colors[Math.floor(Math.random() * colors.length)],
            life: 1
        });
    }
    
    onMouseDown(x: number, y: number): void {
        // Emit burst of particles at mouse position
        for (let i = 0; i < 10; i++) {
            this.emitParticle(x, y);
        }
    }
}

interface Particle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    size: number;
    color: string;
    life: number;
}

// Graphics component factory
export class GraphicsComponentFactory {
    private static components = new Map<string, new() => GraphicsComponent>();
    
    static register(name: string, componentClass: new() => GraphicsComponent): void {
        this.components.set(name, componentClass);
    }
    
    static create(name: string): GraphicsComponent | null {
        const ComponentClass = this.components.get(name);
        return ComponentClass ? new ComponentClass() : null;
    }
    
    static getAvailable(): string[] {
        return Array.from(this.components.keys());
    }
}

// Register example components
GraphicsComponentFactory.register('sine-wave', SineWaveComponent);
GraphicsComponentFactory.register('particles', ParticleSystemComponent);