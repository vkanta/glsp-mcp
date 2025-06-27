/**
 * Unified Status Manager
 * Manages connection status across all UI components
 */

export interface ConnectionStatus {
    mcp: boolean;
    ai: boolean;
    message: string;
    lastUpdated: Date;
}

export type StatusListener = (status: ConnectionStatus) => void;

export class StatusManager {
    private status: ConnectionStatus = {
        mcp: false,
        ai: false,
        message: 'Initializing...',
        lastUpdated: new Date()
    };
    
    private listeners: StatusListener[] = [];

    addListener(listener: StatusListener): void {
        this.listeners.push(listener);
        // Immediately call with current status
        listener(this.status);
    }

    removeListener(listener: StatusListener): void {
        const index = this.listeners.indexOf(listener);
        if (index > -1) {
            this.listeners.splice(index, 1);
        }
    }

    private notifyListeners(): void {
        this.status.lastUpdated = new Date();
        this.listeners.forEach(listener => listener(this.status));
    }

    setMcpStatus(connected: boolean): void {
        console.log('StatusManager: Setting MCP status to:', connected);
        this.status.mcp = connected;
        this.updateMessage();
        this.notifyListeners();
    }

    setAiStatus(connected: boolean): void {
        console.log('StatusManager: Setting AI status to:', connected);
        this.status.ai = connected;
        this.updateMessage();
        this.notifyListeners();
    }

    private updateMessage(): void {
        if (this.status.mcp && this.status.ai) {
            this.status.message = 'All services connected';
        } else if (this.status.mcp) {
            this.status.message = 'MCP connected, AI offline';
        } else if (this.status.ai) {
            this.status.message = 'AI connected, MCP offline';
        } else {
            this.status.message = 'Connecting to services...';
        }
    }

    getStatus(): ConnectionStatus {
        return { ...this.status };
    }

    isFullyConnected(): boolean {
        return this.status.mcp && this.status.ai;
    }

    isMcpConnected(): boolean {
        return this.status.mcp;
    }

    isAiConnected(): boolean {
        return this.status.ai;
    }
}

// Global instance
export const statusManager = new StatusManager();