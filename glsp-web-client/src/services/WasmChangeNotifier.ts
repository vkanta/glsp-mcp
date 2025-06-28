import { WasmFileChangeEvent, WasmChangeType } from '../diagrams/wasm-file-watcher.js';
import { McpService } from './McpService.js';

export interface WasmChangeListener {
    onWasmChange(event: WasmFileChangeEvent): void;
}

export class WasmChangeNotifier {
    private mcpService: McpService;
    private listeners: Set<WasmChangeListener> = new Set();
    private isListening: boolean = false;
    
    constructor(mcpService: McpService) {
        this.mcpService = mcpService;
        console.log('WasmChangeNotifier initialized with MCP');
    }
    
    public start(): void {
        if (this.isListening) {
            console.log('MCP notifications already listening');
            return;
        }
        
        this.startListening();
    }
    
    private startListening(): void {
        try {
            console.log('Starting WASM change polling via MCP...');
            
            // Since HTTP MCP doesn't support push notifications,
            // we'll poll for changes using the scan_wasm_components tool
            this.startPolling();
            
            this.isListening = true;
            console.log('WASM change polling started');
            
        } catch (error) {
            console.error('Failed to start WASM change polling:', error);
        }
    }
    
    private async startPolling(): Promise<void> {
        // Poll every 30 seconds for WASM component changes (reduced from 5 seconds)
        setInterval(async () => {
            if (this.isListening && this.mcpService.isConnected()) {
                try {
                    // Trigger a scan to check for new/changed components
                    await this.mcpService.callTool('scan_wasm_components', {});
                    
                    // Read the components list to see if anything changed
                    const resource = await this.mcpService.readResource('wasm://components/list');
                    if (resource && resource.text) {
                        const data = JSON.parse(resource.text);
                        this.checkForChanges(data.components || []);
                    }
                } catch (error) {
                    // Silently handle errors to avoid spam
                    console.debug('WASM polling error:', error);
                }
            }
        }, 30000);
    }
    
    private lastComponentsSnapshot: any[] = [];
    
    private checkForChanges(currentComponents: any[]): void {
        // Simple change detection by comparing component lists
        const currentNames = new Set(currentComponents.map(c => c.name));
        const lastNames = new Set(this.lastComponentsSnapshot.map(c => c.name));
        
        // Detect added components
        for (const component of currentComponents) {
            if (!lastNames.has(component.name)) {
                this.notifyChange('added', component);
            }
        }
        
        // Detect removed components
        for (const component of this.lastComponentsSnapshot) {
            if (!currentNames.has(component.name)) {
                this.notifyChange('removed', component);
            }
        }
        
        this.lastComponentsSnapshot = currentComponents;
    }
    
    private notifyChange(eventType: string, component: any): void {
        const clientEvent: WasmFileChangeEvent = {
            type: eventType as WasmChangeType,
            path: component.path || '',
            component: {
                name: component.name,
                path: component.path || '',
                description: component.description || `WASM component: ${component.name}`,
                interfaces: component.interfaces || []
            }
        };
        
        console.log(`WASM component ${eventType}:`, component.name);
        
        // Notify all listeners
        this.listeners.forEach(listener => {
            try {
                listener.onWasmChange(clientEvent);
            } catch (error) {
                console.error('Error in WASM change listener:', error);
            }
        });
    }
    
    private handleMcpNotification(notification: any): void {
        try {
            console.log('Received MCP WASM notification:', notification);
            
            if (notification.params) {
                const serverChange = notification.params as WasmServerChange;
                this.handleWasmChange(serverChange);
            }
        } catch (error) {
            console.error('Failed to handle MCP notification:', error);
        }
    }
    
    private handleWasmChange(serverChange: WasmServerChange): void {
        // Convert server change format to client format
        const clientEvent: WasmFileChangeEvent = {
            type: serverChange.event_type as WasmChangeType,
            path: serverChange.path,
            component: {
                name: serverChange.component_name,
                path: serverChange.path,
                description: `WASM component: ${serverChange.component_name}`,
                interfaces: [] // Will be populated by WasmComponentManager
            }
        };
        
        // Notify all listeners
        this.listeners.forEach(listener => {
            try {
                listener.onWasmChange(clientEvent);
            } catch (error) {
                console.error('Error in WASM change listener:', error);
            }
        });
    }
    
    private notifyConnectionStatus(connected: boolean): void {
        // Could add connection status listeners if needed
        console.log(`MCP notification status: ${connected ? 'listening' : 'stopped'}`);
    }
    
    public addListener(listener: WasmChangeListener): void {
        this.listeners.add(listener);
    }
    
    public removeListener(listener: WasmChangeListener): void {
        this.listeners.delete(listener);
    }
    
    public stop(): void {
        this.isListening = false;
        console.log('MCP notification listener stopped');
    }
    
    public isConnected(): boolean {
        return this.isListening && this.mcpService.isConnected();
    }
}

interface WasmServerChange {
    event_type: string;
    path: string;
    component_name: string;
    timestamp: number;
}