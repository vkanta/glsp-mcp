/**
 * AI Diagram Agent
 * Combines Ollama LLM with MCP-GLSP to create diagrams from natural language
 */

import { OllamaClient } from './ollama-client.js';
import { McpClient } from '../mcp/client.js';

export interface DiagramRequest {
    description: string;
    diagramType?: string;
    style?: string;
}

export interface AgentResponse {
    success: boolean;
    message: string;
    diagramId?: string;
    steps: string[];
    errors?: string[];
}

export class DiagramAgent {
    private ollama: OllamaClient;
    private mcpClient: McpClient;
    private systemPrompt: string;

    constructor(ollamaClient: OllamaClient, mcpClient: McpClient) {
        this.ollama = ollamaClient;
        this.mcpClient = mcpClient;
        this.systemPrompt = this.createSystemPrompt();
    }

    private createSystemPrompt(): string {
        return `You are an expert diagram creation agent that helps users create diagrams from natural language descriptions.

You have access to the following MCP tools:
- create_diagram(diagramType, name): Create a new diagram
- create_node(diagramId, nodeType, position, label): Add nodes to diagrams  
- create_edge(diagramId, edgeType, sourceId, targetId): Connect nodes
- apply_layout(diagramId, algorithm): Apply automatic layout

Node types available:
- "start-event": Process start points
- "end-event": Process completion points  
- "task": Work activities
- "user-task": Manual user activities
- "service-task": Automated system activities
- "gateway": Decision points or parallel splits
- "intermediate-event": Milestones or wait points

Edge types available:
- "sequence-flow": Normal process flow
- "conditional-flow": Decision branches
- "message-flow": Communication between processes

When a user describes a process or workflow, you should:
1. Parse the description to identify the main steps, decisions, and flow
2. Generate a JSON array of MCP tool calls to create the diagram
3. Include proper positioning (spread elements across x-axis, use consistent y positions)
4. Apply logical flow connections between elements
5. End with apply_layout to organize the diagram

Respond with a JSON object containing:
{
  "analysis": "Brief analysis of the user's request",
  "toolCalls": [
    {
      "tool": "create_diagram", 
      "arguments": {"diagramType": "workflow", "name": "Process Name"}
    },
    {
      "tool": "create_node",
      "arguments": {"diagramId": "DIAGRAM_ID", "nodeType": "start-event", "position": {"x": 50, "y": 100}, "label": "Start"}
    }
    // ... more tool calls
  ]
}

IMPORTANT: 
- Use "DIAGRAM_ID" as placeholder - it will be replaced with actual ID
- Position elements logically (x: 50, 200, 350, 500... y: consistent like 100)
- Always include start and end events for processes
- Add meaningful labels that reflect the user's description
- Keep it simple but complete`;
    }

    async createDiagramFromDescription(request: DiagramRequest): Promise<AgentResponse> {
        const steps: string[] = [];
        const errors: string[] = [];

        try {
            steps.push("🤖 Analyzing description with AI...");

            // Get AI analysis and tool calls
            const aiResponse = await this.ollama.generateResponse(
                `Create a ${request.diagramType || 'workflow'} diagram for: "${request.description}"`,
                this.systemPrompt
            );

            steps.push("✅ AI analysis complete");

            // Parse AI response
            let parsedResponse;
            try {
                // Extract JSON from AI response (it might have extra text)
                const jsonMatch = aiResponse.match(/\{[\s\S]*\}/);
                if (!jsonMatch) {
                    throw new Error("No JSON found in AI response");
                }
                parsedResponse = JSON.parse(jsonMatch[0]);
            } catch (parseError) {
                errors.push(`Failed to parse AI response: ${parseError}`);
                return {
                    success: false,
                    message: "AI response was not in expected format",
                    steps,
                    errors
                };
            }

            steps.push(`💭 AI Analysis: ${parsedResponse.analysis}`);

            // Execute tool calls
            let diagramId: string | undefined;
            
            for (const toolCall of parsedResponse.toolCalls) {
                try {
                    steps.push(`🔧 Executing: ${toolCall.tool}`);

                    // Replace DIAGRAM_ID placeholder with actual ID
                    if (toolCall.arguments.diagramId === "DIAGRAM_ID" && diagramId) {
                        toolCall.arguments.diagramId = diagramId;
                    }

                    const result = await this.mcpClient.callTool(toolCall.tool, toolCall.arguments);
                    
                    // Extract diagram ID if this was create_diagram
                    if (toolCall.tool === "create_diagram" && result.content?.[0]?.text) {
                        const match = result.content[0].text.match(/ID: ([a-f0-9-]+)/);
                        if (match) {
                            diagramId = match[1];
                        }
                    }

                    steps.push(`✅ ${toolCall.tool} completed`);

                } catch (toolError) {
                    const errorMsg = `Failed to execute ${toolCall.tool}: ${toolError}`;
                    errors.push(errorMsg);
                    steps.push(`❌ ${errorMsg}`);
                }
            }

            if (errors.length > 0) {
                return {
                    success: false,
                    message: `Diagram partially created with ${errors.length} errors`,
                    diagramId,
                    steps,
                    errors
                };
            }

            return {
                success: true,
                message: "Diagram created successfully!",
                diagramId,
                steps
            };

        } catch (error) {
            errors.push(`Agent error: ${error}`);
            return {
                success: false,
                message: "Failed to create diagram",
                steps,
                errors
            };
        }
    }

    async optimizeDiagram(diagramId: string, criteria: string = "readability"): Promise<AgentResponse> {
        const steps: string[] = [];
        const errors: string[] = [];

        try {
            steps.push("🤖 Getting optimization recommendations...");

            const prompt = await this.mcpClient.getPrompt("optimize_layout", {
                diagram_id: diagramId,
                criteria
            });

            const aiResponse = await this.ollama.generateResponse(
                prompt.messages[0].content.text,
                "You are a diagram optimization expert. Provide specific, actionable recommendations."
            );

            steps.push("✅ AI recommendations generated");
            steps.push(`💡 Recommendations: ${aiResponse.substring(0, 200)}...`);

            // Apply layout optimization
            steps.push("📐 Applying layout optimization...");
            await this.mcpClient.callTool("apply_layout", {
                diagramId,
                algorithm: "hierarchical"
            });
            steps.push("✅ Layout optimization applied");

            return {
                success: true,
                message: "Diagram optimized successfully!",
                diagramId,
                steps
            };

        } catch (error) {
            errors.push(`Optimization error: ${error}`);
            return {
                success: false,
                message: "Failed to optimize diagram",
                steps,
                errors
            };
        }
    }

    async analyzeDiagram(diagramId: string, focus: string = "general"): Promise<AgentResponse> {
        const steps: string[] = [];

        try {
            steps.push("📊 Reading diagram data...");

            // Get diagram model
            const modelResource = await this.mcpClient.readResource(`diagram://model/${diagramId}`);
            const diagramData = JSON.parse(modelResource.text || '{}');

            // Get validation results  
            const validationResource = await this.mcpClient.readResource(`diagram://validation/${diagramId}`);
            const validationData = JSON.parse(validationResource.text || '{}');

            steps.push("🤖 Analyzing with AI...");

            const analysisPrompt = `Analyze this diagram:
            
Elements: ${Object.keys(diagramData.elements || {}).length}
Revision: ${diagramData.revision}
Valid: ${validationData.isValid}
Issues: ${validationData.issues?.length || 0}

Focus on: ${focus}

Provide insights about the diagram structure, potential improvements, and any issues found.`;

            const aiAnalysis = await this.ollama.generateResponse(
                analysisPrompt,
                "You are a process analysis expert. Provide clear, actionable insights about diagram quality and potential improvements."
            );

            steps.push("✅ Analysis complete");

            return {
                success: true,
                message: aiAnalysis,
                diagramId,
                steps
            };

        } catch (error) {
            return {
                success: false,
                message: `Analysis failed: ${error}`,
                steps,
                errors: [String(error)]
            };
        }
    }

    async checkConnections(): Promise<{ollama: boolean, mcp: boolean}> {
        const ollamaConnected = await this.ollama.checkConnection();
        
        let mcpConnected = false;
        try {
            await this.mcpClient.healthCheck();
            mcpConnected = true;
        } catch (error) {
            mcpConnected = false;
        }

        return { ollama: ollamaConnected, mcp: mcpConnected };
    }
}