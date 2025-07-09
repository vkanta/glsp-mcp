/**
 * WasmViewTransformer - Handles transformations between WASM component and WIT interface views
 * 
 * This transformer converts WASM component diagrams to show different perspectives:
 * - Component View: Shows WASM components with their connections
 * - Interface View: Shows WIT interfaces, types, and functions extracted from components
 * - Dependencies View: Shows interface dependencies and relationships
 */

import { DiagramModel, ModelElement, Node, Edge } from '../model/diagram.js';
import { ViewTransformer, ViewTransformationResult } from './ViewModeManager.js';

export interface WasmComponentData {
    id: string;
    name: string;
    interfaces: ComponentInterface[];
    position: { x: number; y: number };
    properties: Record<string, unknown>;
}

export interface ComponentInterface {
    name: string;
    type: 'import' | 'export';
    functions?: InterfaceFunction[];
    types?: InterfaceType[];
}

export interface InterfaceFunction {
    name: string;
    parameters?: Parameter[];
    returnType?: string;
}

export interface InterfaceType {
    name: string;
    kind: 'record' | 'variant' | 'enum' | 'flags' | 'resource';
    fields?: TypeField[];
}

export interface Parameter {
    name: string;
    type: string;
}

export interface TypeField {
    name: string;
    type: string;
}

export class WasmViewTransformer implements ViewTransformer {
    
    /**
     * Check if transformation between view modes is supported
     */
    public canTransform(fromView: string, toView: string, diagram: DiagramModel): boolean {
        const supportedViews = ['component', 'wit-interface', 'wit-dependencies'];
        
        if (!supportedViews.includes(fromView) || !supportedViews.includes(toView)) {
            return false;
        }

        // Check if diagram has the necessary data for transformation
        const diagramType = diagram.diagramType || diagram.diagram_type;
        if (diagramType !== 'wasm-component') {
            return false;
        }

        return true;
    }

    /**
     * Transform diagram data for the target view mode
     */
    public transform(diagram: DiagramModel, targetView: string): ViewTransformationResult {
        try {
            switch (targetView) {
                case 'component':
                    return this.transformToComponentView(diagram);
                case 'wit-interface':
                    return this.transformToInterfaceView(diagram);
                case 'wit-dependencies':
                    return this.transformToDependencyView(diagram);
                default:
                    return {
                        success: false,
                        error: `Unsupported view mode: ${targetView}`
                    };
            }
        } catch (error) {
            return {
                success: false,
                error: `Transformation error: ${error instanceof Error ? error.message : String(error)}`
            };
        }
    }

    /**
     * Transform to component view (default WASM component representation)
     */
    private transformToComponentView(diagram: DiagramModel): ViewTransformationResult {
        // Component view uses the original diagram data structure
        // No transformation needed, just ensure elements have correct types
        const elements = Object.values(diagram.elements).map(element => {
            // Ensure WASM component elements have correct types
            if (this.isWitElement(element)) {
                // Convert WIT elements back to WASM component equivalents
                return this.convertWitToWasmElement(element);
            }
            return element;
        });

        return {
            success: true,
            transformedElements: elements,
            additionalData: {
                viewMode: 'component',
                renderingHints: {
                    showComponents: true,
                    showInterfaces: true,
                    showConnections: true
                }
            }
        };
    }

    /**
     * Transform to WIT interface view
     */
    private transformToInterfaceView(diagram: DiagramModel): ViewTransformationResult {
        const elements = Object.values(diagram.elements);
        const wasmComponents = this.extractWasmComponents(elements);
        const witElements: ModelElement[] = [];

        let nodeIdCounter = 1;
        let edgeIdCounter = 1;

        // Transform each WASM component into WIT interface representation
        wasmComponents.forEach((component, index) => {
            const baseX = 150 + (index * 300);
            const baseY = 150;

            // Create package node for each component
            const packageNode: Node = {
                id: `wit-package-${nodeIdCounter++}`,
                type: 'wit-package',
                element_type: 'wit-package',
                label: `${component.name} Package`,
                bounds: {
                    x: baseX,
                    y: baseY,
                    width: 200,
                    height: 60
                },
                properties: {
                    componentId: component.id,
                    interfaceCount: component.interfaces.length
                }
            };
            witElements.push(packageNode);

            // Create interface nodes for each interface
            component.interfaces.forEach((iface, ifaceIndex) => {
                const interfaceY = baseY + 100 + (ifaceIndex * 150);
                
                const interfaceNode: Node = {
                    id: `wit-interface-${nodeIdCounter++}`,
                    type: 'wit-interface',
                    element_type: 'wit-interface',
                    label: iface.name,
                    bounds: {
                        x: baseX,
                        y: interfaceY,
                        width: 160,
                        height: 120
                    },
                    properties: {
                        componentId: component.id,
                        interfaceType: iface.type,
                        functionCount: iface.functions?.length || 0,
                        typeCount: iface.types?.length || 0,
                        functions: iface.functions || [],
                        types: iface.types || []
                    }
                };
                witElements.push(interfaceNode);

                // Create edge from package to interface
                const packageEdge: Edge = {
                    id: `wit-contains-${edgeIdCounter++}`,
                    type: 'wit-contains',
                    element_type: 'wit-contains',
                    source: packageNode.id,
                    target: interfaceNode.id,
                    label: iface.type === 'export' ? 'exports' : 'imports'
                };
                witElements.push(packageEdge);

                // Create function nodes if detailed view is needed
                iface.functions?.forEach((func, funcIndex) => {
                    const functionY = interfaceY + 140 + (funcIndex * 80);
                    
                    const functionNode: Node = {
                        id: `wit-function-${nodeIdCounter++}`,
                        type: 'wit-function',
                        element_type: 'wit-function',
                        label: func.name,
                        bounds: {
                            x: baseX + 200,
                            y: functionY,
                            width: 160,
                            height: 60
                        },
                        properties: {
                            interfaceId: interfaceNode.id,
                            parameters: func.parameters || [],
                            returnType: func.returnType || 'void'
                        }
                    };
                    witElements.push(functionNode);

                    // Create edge from interface to function
                    const functionEdge: Edge = {
                        id: `wit-contains-func-${edgeIdCounter++}`,
                        type: 'wit-contains',
                        element_type: 'wit-contains',
                        source: interfaceNode.id,
                        target: functionNode.id,
                        label: 'contains'
                    };
                    witElements.push(functionEdge);
                });
            });
        });

        return {
            success: true,
            transformedElements: witElements,
            additionalData: {
                viewMode: 'wit-interface',
                renderingHints: {
                    showPackages: true,
                    showInterfaces: true,
                    showFunctions: true,
                    showTypes: true
                }
            }
        };
    }

    /**
     * Transform to dependency view
     */
    private transformToDependencyView(diagram: DiagramModel): ViewTransformationResult {
        const elements = Object.values(diagram.elements);
        const wasmComponents = this.extractWasmComponents(elements);
        const dependencyElements: ModelElement[] = [];

        let nodeIdCounter = 1;
        let edgeIdCounter = 1;

        // Create interface nodes grouped by name (to show dependencies)
        const interfaceMap = new Map<string, { exporters: string[], importers: string[] }>();

        // Analyze interfaces to find dependencies
        wasmComponents.forEach(component => {
            component.interfaces.forEach(iface => {
                if (!interfaceMap.has(iface.name)) {
                    interfaceMap.set(iface.name, { exporters: [], importers: [] });
                }
                const entry = interfaceMap.get(iface.name)!;
                
                if (iface.type === 'export') {
                    entry.exporters.push(component.name);
                } else {
                    entry.importers.push(component.name);
                }
            });
        });

        // Create nodes for interfaces that have both exporters and importers (dependencies)
        let yOffset = 150;
        interfaceMap.forEach((deps, interfaceName) => {
            if (deps.exporters.length > 0 && deps.importers.length > 0) {
                // Create interface node
                const interfaceNode: Node = {
                    id: `dep-interface-${nodeIdCounter++}`,
                    type: 'wit-interface',
                    element_type: 'wit-interface',
                    label: interfaceName,
                    bounds: {
                        x: 400,
                        y: yOffset,
                        width: 200,
                        height: 80
                    },
                    properties: {
                        exporters: deps.exporters,
                        importers: deps.importers,
                        isDependencyInterface: true
                    }
                };
                dependencyElements.push(interfaceNode);

                // Create exporter nodes
                deps.exporters.forEach((exporter, index) => {
                    const exporterNode: Node = {
                        id: `dep-exporter-${nodeIdCounter++}`,
                        type: 'wasm-component',
                        element_type: 'wasm-component',
                        label: exporter,
                        bounds: {
                            x: 100,
                            y: yOffset + (index * 100),
                            width: 150,
                            height: 60
                        },
                        properties: {
                            role: 'exporter',
                            interfaceName: interfaceName
                        }
                    };
                    dependencyElements.push(exporterNode);

                    // Create export edge
                    const exportEdge: Edge = {
                        id: `dep-export-${edgeIdCounter++}`,
                        type: 'wit-export',
                        element_type: 'wit-export',
                        source: exporterNode.id,
                        target: interfaceNode.id,
                        label: 'exports'
                    };
                    dependencyElements.push(exportEdge);
                });

                // Create importer nodes
                deps.importers.forEach((importer, index) => {
                    const importerNode: Node = {
                        id: `dep-importer-${nodeIdCounter++}`,
                        type: 'wasm-component',
                        element_type: 'wasm-component',
                        label: importer,
                        bounds: {
                            x: 700,
                            y: yOffset + (index * 100),
                            width: 150,
                            height: 60
                        },
                        properties: {
                            role: 'importer',
                            interfaceName: interfaceName
                        }
                    };
                    dependencyElements.push(importerNode);

                    // Create import edge
                    const importEdge: Edge = {
                        id: `dep-import-${edgeIdCounter++}`,
                        type: 'wit-import',
                        element_type: 'wit-import',
                        source: interfaceNode.id,
                        target: importerNode.id,
                        label: 'imported by'
                    };
                    dependencyElements.push(importEdge);
                });

                yOffset += Math.max(deps.exporters.length, deps.importers.length) * 100 + 150;
            }
        });

        return {
            success: true,
            transformedElements: dependencyElements,
            additionalData: {
                viewMode: 'wit-dependencies',
                renderingHints: {
                    showDependencies: true,
                    showExporters: true,
                    showImporters: true,
                    groupByInterface: true
                }
            }
        };
    }

    /**
     * Extract WASM component data from diagram elements
     */
    private extractWasmComponents(elements: ModelElement[]): WasmComponentData[] {
        const components: WasmComponentData[] = [];

        elements.forEach(element => {
            const elementType = element.type || element.element_type;
            
            if (elementType === 'wasm-component') {
                const node = element as Node;
                const component: WasmComponentData = {
                    id: node.id,
                    name: node.label || `Component ${node.id}`,
                    interfaces: this.extractComponentInterfaces(node),
                    position: { 
                        x: node.bounds?.x || 0, 
                        y: node.bounds?.y || 0 
                    },
                    properties: node.properties || {}
                };
                components.push(component);
            }
        });

        return components;
    }

    /**
     * Extract interface information from a WASM component node
     */
    private extractComponentInterfaces(component: Node): ComponentInterface[] {
        const interfaces: ComponentInterface[] = [];
        
        // Try to extract interfaces from component properties
        if (component.properties?.interfaces) {
            if (Array.isArray(component.properties.interfaces)) {
                // If interfaces is already an array of interface objects
                return component.properties.interfaces as ComponentInterface[];
            } else if (typeof component.properties.interfaces === 'number') {
                // If interfaces is a count, create mock interfaces
                const count = component.properties.interfaces;
                for (let i = 0; i < count; i++) {
                    interfaces.push({
                        name: `interface-${i + 1}`,
                        type: i % 2 === 0 ? 'import' : 'export',
                        functions: [
                            {
                                name: `function-${i + 1}`,
                                parameters: [{ name: 'input', type: 'string' }],
                                returnType: 'string'
                            }
                        ]
                    });
                }
            }
        }

        // If no interfaces found, create default ones
        if (interfaces.length === 0) {
            interfaces.push(
                {
                    name: 'main-interface',
                    type: 'export',
                    functions: [
                        {
                            name: 'process',
                            parameters: [{ name: 'data', type: 'bytes' }],
                            returnType: 'result'
                        }
                    ]
                },
                {
                    name: 'config-interface',
                    type: 'import',
                    functions: [
                        {
                            name: 'get-config',
                            parameters: [],
                            returnType: 'config'
                        }
                    ]
                }
            );
        }

        return interfaces;
    }

    /**
     * Check if an element is a WIT element type
     */
    private isWitElement(element: ModelElement): boolean {
        const elementType = element.type || element.element_type;
        return elementType?.startsWith('wit-') || false;
    }

    /**
     * Convert WIT element back to WASM component equivalent
     */
    private convertWitToWasmElement(element: ModelElement): ModelElement {
        const elementType = element.type || element.element_type;
        
        if (elementType === 'wit-package') {
            // Convert package back to WASM component
            return {
                ...element,
                type: 'wasm-component',
                element_type: 'wasm-component'
            };
        }
        
        // For other WIT elements, keep as-is or remove them in component view
        return element;
    }
}