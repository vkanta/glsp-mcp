/**
 * MCP HTTP Client for GLSP
 * Implements Model Context Protocol over HTTP with JSON-RPC
 */

export interface JsonRpcRequest {
    jsonrpc: string;
    method: string;
    params?: any;
    id?: string | number;
}

export interface JsonRpcResponse {
    jsonrpc: string;
    result?: any;
    error?: JsonRpcError;
    id?: string | number;
}

export interface JsonRpcError {
    code: number;
    message: string;
    data?: any;
}

export interface InitializeParams {
    protocolVersion: string;
    capabilities: ClientCapabilities;
    clientInfo: ClientInfo;
}

export interface ClientCapabilities {
    experimental?: Record<string, any>;
    sampling?: SamplingCapabilities;
}

export interface SamplingCapabilities {}

export interface ClientInfo {
    name: string;
    version: string;
}

export interface Resource {
    uri: string;
    name: string;
    description?: string;
    mimeType?: string;
}

export interface ResourceContent {
    uri: string;
    mimeType?: string;
    text?: string;
    blob?: string;
}

export interface Tool {
    name: string;
    description?: string;
    inputSchema: any;
}

export interface CallToolParams {
    name: string;
    arguments?: any;
}

export interface CallToolResult {
    content: TextContent[];
    isError?: boolean;
}

export interface TextContent {
    type: string;
    text: string;
}

export interface Prompt {
    name: string;
    description?: string;
    arguments?: PromptArgument[];
}

export interface PromptArgument {
    name: string;
    description?: string;
    required?: boolean;
}

export interface GetPromptParams {
    name: string;
    arguments?: Record<string, string>;
}

export interface GetPromptResult {
    description?: string;
    messages: PromptMessage[];
}

export interface PromptMessage {
    role: string;
    content: TextContent;
}

export class McpClient {
    private baseUrl: string;
    private requestId: number = 0;
    private connected: boolean = false;
    private pingInterval?: number;
    private reconnectTimeout?: number;
    private reconnectAttempts: number = 0;
    private maxReconnectAttempts: number = 5;
    private connectionListeners: ((connected: boolean) => void)[] = [];
    private sessionId: string | null = null;
    private notificationListeners: Map<string, ((notification: any) => void)[]> = new Map();

    constructor(baseUrl: string = 'http://127.0.0.1:3000') {
        this.baseUrl = baseUrl;
    }

    private nextId(): number {
        return ++this.requestId;
    }

    public addConnectionListener(listener: (connected: boolean) => void): void {
        this.connectionListeners.push(listener);
    }

    public removeConnectionListener(listener: (connected: boolean) => void): void {
        const index = this.connectionListeners.indexOf(listener);
        if (index > -1) {
            this.connectionListeners.splice(index, 1);
        }
    }

    private notifyConnectionChange(connected: boolean): void {
        if (this.connected !== connected) {
            this.connected = connected;
            console.log(`MCP connection status changed: ${connected ? 'Connected' : 'Disconnected'}`);
            this.connectionListeners.forEach(listener => listener(connected));
        }
    }

    public isConnected(): boolean {
        return this.connected;
    }

    private async sendNotification(method: string, params?: any): Promise<void> {
        console.log(`McpClient: Sending notification ${method}`, params);
        const notification: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            // Notifications don't have an ID
        };

        try {
            // Build URL with session ID if available
            let url = `${this.baseUrl}/messages`;
            if (this.sessionId) {
                url += `?session_id=${encodeURIComponent(this.sessionId)}`;
            }
            
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                },
                body: JSON.stringify(notification)
            });

            // Notifications don't expect a response body
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
        } catch (error) {
            console.error('MCP notification failed:', error);
            throw error;
        }
    }

    private async sendRequest(method: string, params?: any): Promise<any> {
        console.log(`McpClient: Sending request ${method}`, params);
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id: this.nextId()
        };

        try {
            // Build URL with session ID if available
            let url = `${this.baseUrl}/messages`;
            if (this.sessionId) {
                url += `?session_id=${encodeURIComponent(this.sessionId)}`;
            }
            
            // Create AbortController for timeout
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 8000); // 8 second timeout

            try {
                const response = await fetch(url, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Accept': 'application/json',
                    },
                    body: JSON.stringify(request),
                    signal: controller.signal
                });

                clearTimeout(timeoutId);

                if (!response.ok) {
                    this.notifyConnectionChange(false);
                    this.scheduleReconnect();
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                // Check if response has content
                const contentLength = response.headers.get('content-length');
                if (contentLength === '0' || response.status === 204) {
                    // No content response - might be using legacy SSE mode
                    throw new Error('Empty response - server might be using SSE mode');
                }

                // Extract session ID from response headers if present
                const sessionHeader = response.headers.get('Mcp-Session-Id');
                if (sessionHeader && !this.sessionId) {
                    this.sessionId = sessionHeader;
                    console.log('McpClient: Session ID set:', this.sessionId);
                }
                
                const jsonResponse: JsonRpcResponse = await response.json();

                if (jsonResponse.error) {
                    throw new Error(`MCP Error: ${jsonResponse.error.message}`);
                }

                // Successful request - update connection status
                this.notifyConnectionChange(true);
                this.reconnectAttempts = 0;

                return jsonResponse.result;
            } catch (fetchError: any) {
                clearTimeout(timeoutId);
                
                // Check if it's an abort error (timeout)
                if (fetchError.name === 'AbortError') {
                    throw new Error('Request timeout after 8 seconds');
                }
                throw fetchError;
            }
        } catch (error) {
            console.error('MCP request failed:', error);
            this.notifyConnectionChange(false);
            this.scheduleReconnect();
            throw error;
        }
    }

    private scheduleReconnect(): void {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
        }

        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.log('Max reconnection attempts reached');
            return;
        }

        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000); // Exponential backoff, max 30s
        this.reconnectAttempts++;

        console.log(`Scheduling reconnection attempt ${this.reconnectAttempts} in ${delay}ms`);
        this.reconnectTimeout = window.setTimeout(async () => {
            try {
                await this.initialize();
            } catch (error) {
                console.error('Reconnection attempt failed:', error);
            }
        }, delay);
    }

    private startPing(): void {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
        }

        this.pingInterval = window.setInterval(async () => {
            try {
                await this.ping();
            } catch (error) {
                console.error('Ping failed:', error);
                this.notifyConnectionChange(false);
                this.scheduleReconnect();
            }
        }, 30000); // Ping every 30 seconds
    }

    private stopPing(): void {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = undefined;
        }
    }

    public disconnect(): void {
        this.stopPing();
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = undefined;
        }
        this.notifyConnectionChange(false);
    }

    async ping(): Promise<any> {
        // Try ping first, fall back to a simpler method if ping isn't supported
        try {
            return await this.sendRequest('ping', {});
        } catch (error: any) {
            // If ping method is not supported, try listing tools as a health check
            if (error.message && error.message.includes('Unknown method')) {
                console.log('MCP server does not support ping method, using listTools as health check');
                return await this.listTools();
            }
            throw error;
        }
    }

    async initialize(): Promise<any> {
        console.log('McpClient: Initializing...');
        // Reset session ID on new initialization
        this.sessionId = null;
        
        try {
            const params: InitializeParams = {
                protocolVersion: '2024-11-05',
                capabilities: {
                    experimental: {},
                    sampling: {}
                },
                clientInfo: {
                    name: 'GLSP Web Client',
                    version: '0.1.0'
                }
            };

            const result = await this.sendRequest('initialize', params);
            
            // Send initialized notification (as a notification, not a request)
            await this.sendNotification('initialized', {});
            
            // Explicitly set connection to true after successful initialization
            console.log('MCP client successfully initialized and connected');
            this.notifyConnectionChange(true);
            
            // Start pinging to maintain connection
            this.startPing();
            
            return result;
        } catch (error) {
            console.error('MCP initialization failed:', error);
            this.notifyConnectionChange(false);
            this.scheduleReconnect();
            throw error;
        }
    }

    async listTools(): Promise<Tool[]> {
        const result = await this.sendRequest('tools/list');
        return result.tools;
    }

    async callTool(name: string, args?: any): Promise<CallToolResult> {
        const params: CallToolParams = {
            name,
            arguments: args
        };
        return await this.sendRequest('tools/call', params);
    }

    async listResources(): Promise<Resource[]> {
        const result = await this.sendRequest('resources/list');
        return result.resources;
    }

    async readResource(uri: string): Promise<ResourceContent> {
        return await this.sendRequest('resources/read', { uri });
    }

    async listPrompts(): Promise<Prompt[]> {
        const result = await this.sendRequest('prompts/list');
        return result.prompts;
    }

    async getPrompt(name: string, args?: Record<string, string>): Promise<GetPromptResult> {
        const params: GetPromptParams = {
            name,
            arguments: args
        };
        return await this.sendRequest('prompts/get', params);
    }

    async healthCheck(): Promise<any> {
        // Use ping instead of separate health endpoint
        return await this.ping();
    }
    
    public addNotificationListener(method: string, listener: (notification: any) => void): void {
        if (!this.notificationListeners.has(method)) {
            this.notificationListeners.set(method, []);
        }
        this.notificationListeners.get(method)!.push(listener);
    }
    
    public removeNotificationListener(method: string, listener: (notification: any) => void): void {
        const listeners = this.notificationListeners.get(method);
        if (listeners) {
            const index = listeners.indexOf(listener);
            if (index > -1) {
                listeners.splice(index, 1);
            }
        }
    }
    
    private handleNotification(notification: any): void {
        console.log('Received MCP notification:', notification);
        const listeners = this.notificationListeners.get(notification.method);
        if (listeners) {
            listeners.forEach(listener => {
                try {
                    listener(notification);
                } catch (error) {
                    console.error('Error in notification listener:', error);
                }
            });
        }
    }
}