import { DiagramState, DiagramModel } from '../model/diagram.js';
import { McpService } from './McpService.js';
import { diagramTypeRegistry } from '../diagrams/diagram-type-registry.js';

export class DiagramService {
    private diagramState: DiagramState;
    private mcpService: McpService;
    private currentDiagramId?: string;

    constructor(mcpService: McpService) {
        this.mcpService = mcpService;
        this.diagramState = new DiagramState();
    }

    public getDiagramState(): DiagramState {
        return this.diagramState;
    }

    public getCurrentDiagramId(): string | undefined {
        return this.currentDiagramId;
    }

    public setCurrentDiagramId(id: string | undefined): void {
        this.currentDiagramId = id;
    }

    public async loadDiagram(diagramId: string): Promise<DiagramModel | undefined> {
        try {
            console.log('DiagramService: Loading diagram:', diagramId);
            const diagram: DiagramModel = await this.mcpService.getDiagramModel(diagramId);
            console.log('DiagramService: Got diagram from MCP service:', diagram);
            if (diagram) {
                this.diagramState.updateDiagram(diagram);
                this.currentDiagramId = diagramId;
                console.log(`DiagramService: Successfully loaded diagram: ${diagram.diagramType || 'unknown-type'}`);
                return diagram;
            } else {
                console.warn('DiagramService: getDiagramModel returned null/undefined for:', diagramId);
            }
        } catch (error) {
            console.error('DiagramService: Failed to load diagram:', error);
            // If the diagram doesn't exist on the server, clear our local reference
            if (this.currentDiagramId === diagramId) {
                console.warn(`DiagramService: Diagram ${diagramId} no longer exists on server, clearing local reference`);
                this.currentDiagramId = undefined;
            }
        }
        console.log('DiagramService: Returning undefined for diagram:', diagramId);
        return undefined;
    }

    public async createSampleDiagram(): Promise<DiagramModel | undefined> {
        try {
            const result = await this.mcpService.createDiagram('workflow', 'Sample Workflow');

            console.log('Created diagram:', result);

            const match = result.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (match) {
                const diagramId = match[1];
                this.currentDiagramId = diagramId;
                await this.addSampleNodes(diagramId);
                const diagram = await this.loadDiagram(diagramId);
                return diagram;
            }
        } catch (error) {
            console.error('Failed to create sample diagram:', error);
        }
        return undefined;
    }

    private async addSampleNodes(diagramId: string): Promise<void> {
        try {
            const nodeIds: string[] = [];
            
            const startResult = await this.mcpService.callTool('create_node', {
                diagramId,
                nodeType: 'start-event',
                position: { x: 50, y: 100 },
                label: 'Start'
            });
            const startMatch = startResult.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (startMatch) nodeIds.push(startMatch[1]);

            const taskResult = await this.mcpService.callTool('create_node', {
                diagramId,
                nodeType: 'task',
                position: { x: 200, y: 100 },
                label: 'Process Order'
            });
            const taskMatch = taskResult.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (taskMatch) nodeIds.push(taskMatch[1]);

            const gatewayResult = await this.mcpService.callTool('create_node', {
                diagramId,
                nodeType: 'gateway',
                position: { x: 350, y: 100 },
                label: 'Valid?'
            });
            const gatewayMatch = gatewayResult.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (gatewayMatch) nodeIds.push(gatewayMatch[1]);

            const endResult = await this.mcpService.callTool('create_node', {
                diagramId,
                nodeType: 'end-event',
                position: { x: 500, y: 100 },
                label: 'End'
            });
            const endMatch = endResult.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (endMatch) nodeIds.push(endMatch[1]);

            console.log('Sample nodes created:', nodeIds);
            
            if (nodeIds.length > 1) {
                for (let i = 0; i < nodeIds.length - 1; i++) {
                    await this.mcpService.createEdge(diagramId, 'flow', nodeIds[i], nodeIds[i + 1]);
                    console.log(`Created edge: ${nodeIds[i]} → ${nodeIds[i + 1]}`);
                }
            }
            
        } catch (error) {
            console.error('Failed to create sample nodes:', error);
        }
    }

    public async updateSelectedElementPositions(diagramId: string, selectedElements: any[]): Promise<void> {
        if (!diagramId) return;
        
        try {
            for (const element of selectedElements) {
                if (element?.bounds) {
                    await this.mcpService.updateElement(diagramId, element.id, { x: element.bounds.x, y: element.bounds.y });
                }
            }
            await this.loadDiagram(diagramId);
            console.log(`Moved ${selectedElements.length} element(s)`);
        } catch (error) {
            console.error('Failed to update element positions:', error);
        }
    }

    public async createNewDiagram(diagramType: string, name: string): Promise<string | undefined> {
        try {
            const result = await this.mcpService.createDiagram(diagramType, name);

            const match = result.content?.[0]?.text?.match(/ID: ([a-f0-9-]+)/);
            if (match) {
                const diagramId = match[1];
                await this.loadDiagram(diagramId);
                return diagramId;
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
        }
        return undefined;
    }

    public async saveDiagram(diagramId: string): Promise<void> {
        if (!diagramId) return;

        try {
            const result = await this.mcpService.exportDiagram(diagramId, 'json');
            console.log('Diagram saved:', result);
        } catch (error) {
            console.error('Failed to save diagram:', error);
        }
    }

    public async applyLayout(diagramId: string, algorithm: string): Promise<void> {
        try {
            await this.mcpService.applyLayout(diagramId, algorithm);
            await this.loadDiagram(diagramId);
        } catch (error) {
            console.error('Failed to apply layout:', error);
        }
    }

    public async deleteElements(diagramId: string, elementIds: string[]): Promise<void> {
        if (!diagramId || elementIds.length === 0) return;

        try {
            for (const elementId of elementIds) {
                await this.mcpService.deleteElement(diagramId, elementId);
            }
            await this.loadDiagram(diagramId);
            console.log(`Deleted ${elementIds.length} element(s)`);
        } catch (error) {
            console.error('Failed to delete elements:', error);
        }
    }

    public async createNode(diagramId: string, nodeType: string, position: { x: number; y: number }, label: string): Promise<void> {
        if (!diagramId) return;
        
        try {
            await this.mcpService.createNode(diagramId, nodeType, position, label);
            await this.loadDiagram(diagramId);
        } catch (error) {
            console.error('Failed to create node:', error);
        }
    }

    public async createEdge(diagramId: string, edgeType: string, sourceId: string, targetId: string, label?: string): Promise<void> {
        if (!diagramId) return;

        try {
            await this.mcpService.createEdge(diagramId, edgeType, sourceId, targetId, label);
            await this.loadDiagram(diagramId);
        } catch (error) {
            console.error('Failed to create edge:', error);
        }
    }

    public async getAvailableDiagrams(): Promise<any[]> {
        try {
            const data = await this.mcpService.listDiagrams();
            return data.diagrams || [];
        } catch (error) {
            console.error('Failed to get available diagrams:', error);
        }
        return [];
    }

    public async deleteDiagram(diagramId: string): Promise<boolean> {
        try {
            const result = await this.mcpService.deleteDiagram(diagramId);
            console.log('DiagramService: Delete result:', result);
            
            // Clear from local state if it's the current diagram
            if (this.currentDiagramId === diagramId) {
                this.currentDiagramId = undefined;
                // The diagram will be removed from the list when we refresh
            }
            
            return true;
        } catch (error) {
            console.error('Failed to delete diagram:', error);
            return false;
        }
    }

    public getAvailableDiagramTypes(): any[] {
        return diagramTypeRegistry.getAvailableTypes();
    }
}