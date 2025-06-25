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

    constructor(baseUrl: string = 'http://localhost:3000') {
        this.baseUrl = baseUrl;
    }

    private nextId(): number {
        return ++this.requestId;
    }

    private async sendRequest(method: string, params?: any): Promise<any> {
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id: this.nextId()
        };

        const response = await fetch(`${this.baseUrl}/mcp/rpc`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonResponse: JsonRpcResponse = await response.json();

        if (jsonResponse.error) {
            throw new Error(`MCP Error: ${jsonResponse.error.message}`);
        }

        return jsonResponse.result;
    }

    async initialize(): Promise<any> {
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
        
        // Send initialized notification
        await this.sendRequest('initialized', {});
        
        return result;
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
        const response = await fetch(`${this.baseUrl}/health`);
        return await response.json();
    }
}