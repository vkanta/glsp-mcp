/**
 * Thin Client Validation Service
 * 
 * This service acts as a proxy to the backend validation services,
 * replacing heavy client-side validation with server-side validation.
 * Part of the architecture transformation to move business logic to the backend.
 */

import { McpService } from './McpService.js';

export interface SecurityIssue {
    issue_type: any;
    risk_level: 'Low' | 'Medium' | 'High' | 'Critical';
    description: string;
    recommendation: string;
    location?: string;
}

export interface SecurityAnalysis {
    component_name: string;
    component_path: string;
    scan_timestamp: string;
    overall_risk: 'Low' | 'Medium' | 'High' | 'Critical';
    issues: SecurityIssue[];
    permissions_requested: string[];
    imports_analyzed: number;
    exports_analyzed: number;
    scan_duration_ms: number;
    is_component_valid: boolean;
    trusted_signature?: string;
}

export interface WitValidationIssue {
    issue_type: any;
    severity: 'Info' | 'Warning' | 'Error' | 'Critical';
    message: string;
    suggestion?: string;
    location?: string;
}

export interface ComponentWitAnalysis {
    component_name: string;
    world_name?: string;
    imports: any[];
    exports: any[];
    types: any[];
    dependencies: any[];
    raw_wit?: string;
    validation_results: WitValidationIssue[];
    compatibility_report?: any;
    analysis_timestamp: string;
}

export interface ValidationSummary {
    total_components: number;
    security_coverage: number;
    wit_coverage: number;
    critical_issues: number;
    high_issues: number;
    overall_health: 'Excellent' | 'Good' | 'Degraded' | 'Critical';
}

export class ValidationService {
    private mcpService: McpService;
    private validationCache: Map<string, any> = new Map();
    private cacheTimeout = 5 * 60 * 1000; // 5 minutes

    constructor(mcpService: McpService) {
        this.mcpService = mcpService;
    }

    /**
     * Request security analysis for a WASM component from the backend
     */
    async requestSecurityAnalysis(componentName: string): Promise<SecurityAnalysis | null> {
        console.log(`ValidationService: Requesting security analysis for ${componentName}`);

        try {
            // Check cache first
            const cacheKey = `security_${componentName}`;
            const cached = this.getFromCache(cacheKey);
            if (cached) {
                console.log(`ValidationService: Using cached security analysis for ${componentName}`);
                return cached;
            }

            // Request from backend via MCP
            const result = await this.mcpService.callTool('analyze_component_security', {
                component_name: componentName
            });

            if (result && result.content && result.content[0]) {
                const analysis = JSON.parse(result.content[0].text);
                this.setCache(cacheKey, analysis);
                console.log(`ValidationService: Received security analysis for ${componentName}:`, analysis);
                return analysis;
            }

            return null;
        } catch (error) {
            console.error(`ValidationService: Failed to get security analysis for ${componentName}:`, error);
            return null;
        }
    }

    /**
     * Request WIT validation for a WASM component from the backend
     */
    async requestWitValidation(componentName: string): Promise<ComponentWitAnalysis | null> {
        console.log(`ValidationService: Requesting WIT validation for ${componentName}`);

        try {
            // Check cache first
            const cacheKey = `wit_${componentName}`;
            const cached = this.getFromCache(cacheKey);
            if (cached) {
                console.log(`ValidationService: Using cached WIT analysis for ${componentName}`);
                return cached;
            }

            // Request from backend via MCP
            const result = await this.mcpService.callTool('analyze_component_wit', {
                component_name: componentName
            });

            if (result && result.content && result.content[0]) {
                const analysis = JSON.parse(result.content[0].text);
                this.setCache(cacheKey, analysis);
                console.log(`ValidationService: Received WIT analysis for ${componentName}:`, analysis);
                return analysis;
            }

            return null;
        } catch (error) {
            console.error(`ValidationService: Failed to get WIT validation for ${componentName}:`, error);
            return null;
        }
    }

    /**
     * Request compatibility analysis between two components
     */
    async requestCompatibilityAnalysis(componentA: string, componentB: string): Promise<any | null> {
        console.log(`ValidationService: Requesting compatibility analysis between ${componentA} and ${componentB}`);

        try {
            const result = await this.mcpService.callTool('analyze_component_compatibility', {
                component_a: componentA,
                component_b: componentB
            });

            if (result && result.content && result.content[0]) {
                const analysis = JSON.parse(result.content[0].text);
                console.log(`ValidationService: Received compatibility analysis:`, analysis);
                return analysis;
            }

            return null;
        } catch (error) {
            console.error(`ValidationService: Failed to get compatibility analysis:`, error);
            return null;
        }
    }

    /**
     * Get validation summary for all components
     */
    async getValidationSummary(): Promise<ValidationSummary | null> {
        console.log('ValidationService: Requesting validation summary');

        try {
            // Check cache first
            const cacheKey = 'validation_summary';
            const cached = this.getFromCache(cacheKey);
            if (cached) {
                console.log('ValidationService: Using cached validation summary');
                return cached;
            }

            // Request from backend via MCP
            const result = await this.mcpService.callTool('get_validation_summary', {});

            if (result && result.content && result.content[0]) {
                const summary = JSON.parse(result.content[0].text);
                this.setCache(cacheKey, summary, 60000); // Cache for 1 minute only
                console.log('ValidationService: Received validation summary:', summary);
                return summary;
            }

            return null;
        } catch (error) {
            console.error('ValidationService: Failed to get validation summary:', error);
            return null;
        }
    }

    /**
     * Trigger a refresh of all component validations
     */
    async refreshAllValidations(): Promise<boolean> {
        console.log('ValidationService: Requesting validation refresh');

        try {
            const result = await this.mcpService.callTool('refresh_component_validations', {});
            
            // Clear cache to force fresh data
            this.clearCache();
            
            console.log('ValidationService: Validation refresh completed');
            return true;
        } catch (error) {
            console.error('ValidationService: Failed to refresh validations:', error);
            return false;
        }
    }

    /**
     * Subscribe to real-time validation updates via HTTP streaming
     */
    setupValidationStreaming(): void {
        const mcpClient = this.mcpService.getClient();
        
        // Listen for security analysis updates
        mcpClient.addStreamListener('security_analysis_update', (data: any) => {
            console.log('ValidationService: Received security analysis update:', data);
            
            // Update cache with new data
            if (data.component_name) {
                const cacheKey = `security_${data.component_name}`;
                this.setCache(cacheKey, data.analysis);
            }
            
            // Notify UI components
            this.notifyValidationUpdate('security', data);
        });

        // Listen for WIT analysis updates
        mcpClient.addStreamListener('wit_analysis_update', (data: any) => {
            console.log('ValidationService: Received WIT analysis update:', data);
            
            // Update cache with new data
            if (data.component_name) {
                const cacheKey = `wit_${data.component_name}`;
                this.setCache(cacheKey, data.analysis);
            }
            
            // Notify UI components
            this.notifyValidationUpdate('wit', data);
        });

        // Listen for component discovery updates
        mcpClient.addStreamListener('component_discovered', (data: any) => {
            console.log('ValidationService: New component discovered:', data);
            
            // Clear summary cache to get updated counts
            this.clearCache('validation_summary');
            
            // Notify UI components
            this.notifyValidationUpdate('discovery', data);
        });
    }

    private notifyValidationUpdate(type: string, data: any): void {
        // Dispatch custom events that UI components can listen to
        const event = new CustomEvent('validation-update', {
            detail: { type, data }
        });
        window.dispatchEvent(event);
    }

    private getFromCache(key: string): any | null {
        const item = this.validationCache.get(key);
        if (item && Date.now() - item.timestamp < this.cacheTimeout) {
            return item.data;
        }
        if (item) {
            this.validationCache.delete(key);
        }
        return null;
    }

    private setCache(key: string, data: any, customTimeout?: number): void {
        this.validationCache.set(key, {
            data,
            timestamp: Date.now()
        });
        
        // Set custom timeout if provided
        if (customTimeout) {
            setTimeout(() => {
                this.validationCache.delete(key);
            }, customTimeout);
        }
    }

    private clearCache(pattern?: string): void {
        if (pattern) {
            for (const key of this.validationCache.keys()) {
                if (key.includes(pattern)) {
                    this.validationCache.delete(key);
                }
            }
        } else {
            this.validationCache.clear();
        }
    }

    /**
     * Get cached validation data for quick UI updates
     */
    getCachedSecurityAnalysis(componentName: string): SecurityAnalysis | null {
        return this.getFromCache(`security_${componentName}`);
    }

    /**
     * Get cached WIT analysis data for quick UI updates
     */
    getCachedWitAnalysis(componentName: string): ComponentWitAnalysis | null {
        return this.getFromCache(`wit_${componentName}`);
    }

    /**
     * Check if streaming is active
     */
    isStreamingActive(): boolean {
        return this.mcpService.getClient().isStreaming();
    }
}