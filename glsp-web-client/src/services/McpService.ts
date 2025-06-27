import { McpClient } from '../mcp/client.js';

export class McpService {
    private mcpClient: McpClient;

    constructor() {
        this.mcpClient = new McpClient();
    }

    async initialize(): Promise<void> {
        await this.mcpClient.initialize();
    }

    public isConnected(): boolean {
        return this.mcpClient.isConnected();
    }

    public addConnectionListener(listener: (connected: boolean) => void): void {
        this.mcpClient.addConnectionListener(listener);
    }

    public removeConnectionListener(listener: (connected: boolean) => void): void {
        this.mcpClient.removeConnectionListener(listener);
    }

    public disconnect(): void {
        this.mcpClient.disconnect();
    }

    async callTool(toolName: string, params: any): Promise<any> {
        return this.mcpClient.callTool(toolName, params);
    }

    async createDiagram(diagramType: string, name: string): Promise<any> {
        return this.callTool('create_diagram', { diagramType, name });
    }

    async deleteDiagram(diagramId: string): Promise<any> {
        return this.callTool('delete_diagram', { diagramId });
    }

    async createNode(diagramId: string, nodeType: string, position: { x: number; y: number }, label: string): Promise<any> {
        return this.callTool('create_node', { diagramId, nodeType, position, label });
    }

    async createEdge(diagramId: string, edgeType: string, sourceId: string, targetId: string, label?: string): Promise<any> {
        return this.callTool('create_edge', { diagramId, edgeType, sourceId, targetId, label });
    }

    async updateElement(diagramId: string, elementId: string, position: { x: number, y: number }): Promise<any> {
        return this.callTool('update_element', { diagramId, elementId, position });
    }

    async exportDiagram(diagramId: string, format: string): Promise<any> {
        return this.callTool('export_diagram', { diagramId, format });
    }

    async deleteElement(diagramId: string, elementId: string): Promise<any> {
        return this.callTool('delete_element', { diagramId, elementId });
    }

    async applyLayout(diagramId: string, algorithm: string): Promise<any> {
        return this.callTool('apply_layout', { diagramId, algorithm });
    }

    async listDiagrams(): Promise<any> {
        const resource = await this.readResource('diagram://list');
        console.log('McpService: List diagrams resource:', resource);
        if (resource && resource.text) {
            const parsed = JSON.parse(resource.text);
            console.log('McpService: Parsed diagram list:', parsed);
            return parsed;
        }
        console.log('McpService: No diagram list found, returning empty');
        return { diagrams: [] };
    }

    async readResource(uri: string): Promise<any> {
        const result = await this.mcpClient.readResource(uri);
        console.log('McpService: Raw resource result for', uri, ':', result);
        // The MCP client returns { contents: [{ text: "...", uri: "...", mime_type: "..." }] }
        if (result.contents && result.contents.length > 0) {
            return result.contents[0]; // Return the first content item
        }
        return result; // Fallback for different format
    }

    async getDiagramModel(diagramId: string): Promise<any> {
        const resource = await this.readResource(`diagram://model/${diagramId}`);
        console.log('McpService: Diagram model resource for', diagramId, ':', resource);
        console.log('McpService: Resource has text?', !!resource?.text);
        console.log('McpService: Resource keys:', resource ? Object.keys(resource) : 'resource is null');
        if (resource && resource.text) {
            try {
                const parsed = JSON.parse(resource.text);
                console.log('McpService: Successfully parsed diagram model:', parsed);
                return parsed;
            } catch (e) {
                console.error('McpService: Failed to parse diagram model JSON:', e);
                console.error('McpService: Raw text was:', resource.text);
                return undefined;
            }
        }
        console.log('McpService: No valid resource.text found, returning undefined');
        return undefined;
    }

    public getClient(): McpClient {
        return this.mcpClient;
    }
}