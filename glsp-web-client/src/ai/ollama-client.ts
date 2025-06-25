/**
 * Ollama Client for local LLM communication
 * Connects to Ollama API to process natural language requests
 */

export interface OllamaMessage {
    role: 'system' | 'user' | 'assistant';
    content: string;
}

export interface OllamaRequest {
    model: string;
    messages: OllamaMessage[];
    stream?: boolean;
    options?: {
        temperature?: number;
        top_p?: number;
        top_k?: number;
    };
}

export interface OllamaResponse {
    model: string;
    created_at: string;
    message: OllamaMessage;
    done: boolean;
    total_duration?: number;
    load_duration?: number;
    prompt_eval_count?: number;
    prompt_eval_duration?: number;
    eval_count?: number;
    eval_duration?: number;
}

export class OllamaClient {
    private baseUrl: string;
    private defaultModel: string;

    constructor(baseUrl: string = 'http://127.0.0.1:11434', defaultModel: string = 'llama2') {
        this.baseUrl = baseUrl;
        this.defaultModel = defaultModel;
    }

    async chat(messages: OllamaMessage[], model?: string): Promise<OllamaResponse> {
        const request: OllamaRequest = {
            model: model || this.defaultModel,
            messages,
            stream: false,
            options: {
                temperature: 0.7,
                top_p: 0.9
            }
        };

        const response = await fetch(`${this.baseUrl}/api/chat`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });

        if (!response.ok) {
            throw new Error(`Ollama error: ${response.status} ${response.statusText}`);
        }

        return await response.json();
    }

    async generateResponse(userMessage: string, systemPrompt?: string, model?: string): Promise<string> {
        const messages: OllamaMessage[] = [];

        if (systemPrompt) {
            messages.push({
                role: 'system',
                content: systemPrompt
            });
        }

        messages.push({
            role: 'user',
            content: userMessage
        });

        const response = await this.chat(messages, model);
        return response.message.content;
    }

    async listModels(): Promise<{name: string, size: number, modified: string}[]> {
        const response = await fetch(`${this.baseUrl}/api/tags`);
        
        if (!response.ok) {
            throw new Error(`Ollama error: ${response.status} ${response.statusText}`);
        }

        const data = await response.json();
        return data.models || [];
    }

    async getAvailableModelNames(): Promise<string[]> {
        const models = await this.listModels();
        return models.map(model => model.name);
    }

    async autoSelectModel(): Promise<string | null> {
        try {
            const modelNames = await this.getAvailableModelNames();
            
            if (modelNames.length === 0) {
                return null;
            }

            // Prefer common models in order
            const preferredModels = [
                'llama3.2', 'llama3.1', 'llama3', 'llama2', 
                'mistral', 'codellama', 'phi3', 'gemma'
            ];

            for (const preferred of preferredModels) {
                const found = modelNames.find(name => name.toLowerCase().includes(preferred));
                if (found) {
                    this.defaultModel = found;
                    return found;
                }
            }

            // If no preferred model, use the first available
            this.defaultModel = modelNames[0];
            return modelNames[0];

        } catch (error) {
            console.warn('Failed to auto-select model:', error);
            return null;
        }
    }

    async checkConnection(): Promise<boolean> {
        try {
            const response = await fetch(`${this.baseUrl}/api/tags`, {
                method: 'GET',
                signal: AbortSignal.timeout(5000) // 5 second timeout
            });
            return response.ok;
        } catch (error) {
            return false;
        }
    }

    setDefaultModel(model: string): void {
        this.defaultModel = model;
    }

    getDefaultModel(): string {
        return this.defaultModel;
    }
}