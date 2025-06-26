/**
 * WebAssembly Component Composer Example
 * Demonstrates creating a complete web application using WASM components
 */

import { WasmComponentConfig, WasmComposition, WasmConnection } from '../glsp-web-client/src/diagrams/wasm-component-types.js';

// Example: Building a Web API with WASM Components
export const webApiComposition: WasmComposition = {
    name: 'Web API Application',
    description: 'A complete web API built from composable WASM components',
    
    components: [
        // Frontend HTTP Handler
        {
            name: 'http-router',
            path: './components/http-router.wasm',
            description: 'Routes HTTP requests to appropriate handlers',
            interfaces: [
                {
                    name: 'wasi:http/incoming-handler',
                    type: 'export',
                    interfaceType: 'wasi:http/incoming-handler',
                    functions: [
                        {
                            name: 'handle',
                            params: [{ name: 'request', type: 'incoming-request' }],
                            returns: [{ name: 'response', type: 'outgoing-response' }]
                        }
                    ]
                },
                {
                    name: 'custom:router/handler',
                    type: 'import',
                    interfaceType: 'custom:router/handler'
                }
            ]
        },

        // Authentication Component
        {
            name: 'auth-middleware',
            path: './components/auth-middleware.wasm',
            description: 'JWT-based authentication middleware',
            interfaces: [
                {
                    name: 'custom:auth/middleware',
                    type: 'export',
                    interfaceType: 'custom:auth/middleware',
                    functions: [
                        {
                            name: 'authenticate',
                            params: [{ name: 'token', type: 'string' }],
                            returns: [{ name: 'user', type: 'option<user-info>' }]
                        },
                        {
                            name: 'authorize',
                            params: [
                                { name: 'user', type: 'user-info' },
                                { name: 'resource', type: 'string' },
                                { name: 'action', type: 'string' }
                            ],
                            returns: [{ name: 'allowed', type: 'bool' }]
                        }
                    ]
                },
                {
                    name: 'custom:crypto/jwt',
                    type: 'import',
                    interfaceType: 'custom:crypto/jwt'
                }
            ]
        },

        // Business Logic Component
        {
            name: 'user-service',
            path: './components/user-service.wasm',
            description: 'User management business logic',
            interfaces: [
                {
                    name: 'custom:user/service',
                    type: 'export',
                    interfaceType: 'custom:user/service',
                    functions: [
                        {
                            name: 'create-user',
                            params: [{ name: 'user-data', type: 'user-create-request' }],
                            returns: [{ name: 'user', type: 'result<user-info, service-error>' }]
                        },
                        {
                            name: 'get-user',
                            params: [{ name: 'user-id', type: 'string' }],
                            returns: [{ name: 'user', type: 'option<user-info>' }]
                        },
                        {
                            name: 'list-users',
                            params: [{ name: 'filters', type: 'user-filters' }],
                            returns: [{ name: 'users', type: 'list<user-info>' }]
                        }
                    ]
                },
                {
                    name: 'custom:database/client',
                    type: 'import',
                    interfaceType: 'custom:database/client'
                },
                {
                    name: 'custom:validation/service',
                    type: 'import',
                    interfaceType: 'custom:validation/service'
                }
            ]
        },

        // Database Component
        {
            name: 'postgres-client',
            path: './components/postgres-client.wasm',
            description: 'PostgreSQL database client',
            interfaces: [
                {
                    name: 'custom:database/client',
                    type: 'export',
                    interfaceType: 'custom:database/client',
                    functions: [
                        {
                            name: 'execute-query',
                            params: [
                                { name: 'sql', type: 'string' },
                                { name: 'params', type: 'list<db-value>' }
                            ],
                            returns: [{ name: 'result', type: 'result<query-result, db-error>' }]
                        },
                        {
                            name: 'execute-transaction',
                            params: [{ name: 'queries', type: 'list<query-with-params>' }],
                            returns: [{ name: 'result', type: 'result<list<query-result>, db-error>' }]
                        }
                    ]
                },
                {
                    name: 'wasi:sockets/tcp',
                    type: 'import',
                    interfaceType: 'wasi:sockets/tcp'
                }
            ]
        },

        // Validation Component
        {
            name: 'json-validator',
            path: './components/json-validator.wasm',
            description: 'JSON schema validation',
            interfaces: [
                {
                    name: 'custom:validation/service',
                    type: 'export',
                    interfaceType: 'custom:validation/service',
                    functions: [
                        {
                            name: 'validate-schema',
                            params: [
                                { name: 'data', type: 'json-value' },
                                { name: 'schema', type: 'json-schema' }
                            ],
                            returns: [{ name: 'result', type: 'result<unit, list<validation-error>>' }]
                        }
                    ]
                }
            ]
        },

        // Crypto Component
        {
            name: 'jwt-crypto',
            path: './components/jwt-crypto.wasm',
            description: 'JWT token creation and verification',
            interfaces: [
                {
                    name: 'custom:crypto/jwt',
                    type: 'export',
                    interfaceType: 'custom:crypto/jwt',
                    functions: [
                        {
                            name: 'sign',
                            params: [
                                { name: 'payload', type: 'json-value' },
                                { name: 'secret', type: 'string' }
                            ],
                            returns: [{ name: 'token', type: 'string' }]
                        },
                        {
                            name: 'verify',
                            params: [
                                { name: 'token', type: 'string' },
                                { name: 'secret', type: 'string' }
                            ],
                            returns: [{ name: 'payload', type: 'result<json-value, crypto-error>' }]
                        }
                    ]
                },
                {
                    name: 'wasi:random/random',
                    type: 'import',
                    interfaceType: 'wasi:random/random'
                }
            ]
        }
    ],

    // Define how components connect to each other
    connections: [
        // HTTP Router uses Auth Middleware
        {
            sourceComponent: 'http-router',
            sourceInterface: 'custom:router/handler',
            targetComponent: 'auth-middleware',
            targetInterface: 'custom:auth/middleware'
        },

        // Auth Middleware uses JWT Crypto
        {
            sourceComponent: 'auth-middleware',
            sourceInterface: 'custom:crypto/jwt',
            targetComponent: 'jwt-crypto',
            targetInterface: 'custom:crypto/jwt'
        },

        // User Service uses Database Client
        {
            sourceComponent: 'user-service',
            sourceInterface: 'custom:database/client',
            targetComponent: 'postgres-client',
            targetInterface: 'custom:database/client'
        },

        // User Service uses Validation
        {
            sourceComponent: 'user-service',
            sourceInterface: 'custom:validation/service',
            targetComponent: 'json-validator',
            targetInterface: 'custom:validation/service'
        }
    ],

    // Host-provided interfaces (runtime capabilities)
    hostInterfaces: [
        {
            name: 'wasi:sockets/tcp',
            type: 'export',
            interfaceType: 'wasi:sockets/tcp'
        },
        {
            name: 'wasi:random/random',
            type: 'export',
            interfaceType: 'wasi:random/random'
        },
        {
            name: 'wasi:clocks/wall-clock',
            type: 'export',
            interfaceType: 'wasi:clocks/wall-clock'
        },
        {
            name: 'wasi:filesystem/preopens',
            type: 'export',
            interfaceType: 'wasi:filesystem/preopens'
        }
    ]
};

// Example: IoT Data Processing Pipeline
export const iotPipelineComposition: WasmComposition = {
    name: 'IoT Data Processing Pipeline',
    description: 'Real-time IoT sensor data processing and analytics',
    
    components: [
        {
            name: 'mqtt-collector',
            path: './components/mqtt-collector.wasm',
            description: 'Collects data from MQTT sensors',
            interfaces: [
                {
                    name: 'custom:iot/data-collector',
                    type: 'export',
                    interfaceType: 'custom:iot/data-collector'
                },
                {
                    name: 'wasi:sockets/tcp',
                    type: 'import',
                    interfaceType: 'wasi:sockets/tcp'
                }
            ]
        },

        {
            name: 'data-transformer',
            path: './components/data-transformer.wasm',
            description: 'Transforms raw sensor data into structured format',
            interfaces: [
                {
                    name: 'custom:iot/transformer',
                    type: 'export',
                    interfaceType: 'custom:iot/transformer'
                }
            ]
        },

        {
            name: 'anomaly-detector',
            path: './components/anomaly-detector.wasm',
            description: 'ML-based anomaly detection for sensor data',
            interfaces: [
                {
                    name: 'custom:ml/anomaly-detector',
                    type: 'export',
                    interfaceType: 'custom:ml/anomaly-detector'
                }
            ]
        },

        {
            name: 'timeseries-store',
            path: './components/timeseries-store.wasm',
            description: 'Time-series database storage',
            interfaces: [
                {
                    name: 'custom:storage/timeseries',
                    type: 'export',
                    interfaceType: 'custom:storage/timeseries'
                }
            ]
        },

        {
            name: 'alert-manager',
            path: './components/alert-manager.wasm',
            description: 'Manages alerts and notifications',
            interfaces: [
                {
                    name: 'custom:alerts/manager',
                    type: 'export',
                    interfaceType: 'custom:alerts/manager'
                },
                {
                    name: 'wasi:http/outgoing-handler',
                    type: 'import',
                    interfaceType: 'wasi:http/outgoing-handler'
                }
            ]
        }
    ],

    connections: [
        {
            sourceComponent: 'mqtt-collector',
            sourceInterface: 'custom:iot/data-collector',
            targetComponent: 'data-transformer',
            targetInterface: 'custom:iot/transformer'
        },
        {
            sourceComponent: 'data-transformer',
            sourceInterface: 'custom:iot/transformer',
            targetComponent: 'anomaly-detector',
            targetInterface: 'custom:ml/anomaly-detector'
        },
        {
            sourceComponent: 'data-transformer',
            sourceInterface: 'custom:iot/transformer',
            targetComponent: 'timeseries-store',
            targetInterface: 'custom:storage/timeseries'
        },
        {
            sourceComponent: 'anomaly-detector',
            sourceInterface: 'custom:ml/anomaly-detector',
            targetComponent: 'alert-manager',
            targetInterface: 'custom:alerts/manager'
        }
    ],

    hostInterfaces: [
        {
            name: 'wasi:sockets/tcp',
            type: 'export',
            interfaceType: 'wasi:sockets/tcp'
        },
        {
            name: 'wasi:http/outgoing-handler',
            type: 'export',
            interfaceType: 'wasi:http/outgoing-handler'
        },
        {
            name: 'wasi:clocks/wall-clock',
            type: 'export',
            interfaceType: 'wasi:clocks/wall-clock'
        }
    ]
};

// Helper function to generate diagram elements from a composition
export function compositionToDiagramElements(composition: WasmComposition) {
    const elements: any[] = [];
    let x = 50;
    let y = 50;
    const componentSpacing = 200;
    const rowHeight = 150;
    let componentsPerRow = 3;
    let currentRow = 0;
    let currentCol = 0;

    // Add composition root
    elements.push({
        id: 'composition-root',
        type: 'composition-root',
        bounds: { x: x + componentSpacing, y: 20, width: 200, height: 60 },
        properties: {
            label: composition.name,
            description: composition.description
        }
    });

    // Add components
    composition.components.forEach(component => {
        const componentX = x + (currentCol * componentSpacing);
        const componentY = y + (currentRow * rowHeight);

        elements.push({
            id: component.name,
            type: 'wasm-component',
            bounds: { x: componentX, y: componentY, width: 160, height: 100 },
            properties: {
                label: component.name,
                description: component.description,
                interfaces: component.interfaces,
                path: component.path
            }
        });

        currentCol++;
        if (currentCol >= componentsPerRow) {
            currentCol = 0;
            currentRow++;
        }
    });

    // Add host components for host interfaces
    if (composition.hostInterfaces.length > 0) {
        const hostY = y + ((currentRow + 1) * rowHeight);
        elements.push({
            id: 'host-runtime',
            type: 'host-component',
            bounds: { x: x, y: hostY, width: 140, height: 80 },
            properties: {
                label: 'Host Runtime',
                description: 'Provides WASI and host capabilities',
                interfaces: composition.hostInterfaces
            }
        });
    }

    // Add connections as edges
    composition.connections.forEach((connection, index) => {
        elements.push({
            id: `connection-${index}`,
            type: 'interface-connection',
            sourceId: connection.sourceComponent,
            targetId: connection.targetComponent,
            properties: {
                sourceInterface: connection.sourceInterface,
                targetInterface: connection.targetInterface
            }
        });
    });

    return elements;
}

// Export examples for use in the application
export const WASM_COMPOSITION_EXAMPLES = {
    'web-api': webApiComposition,
    'iot-pipeline': iotPipelineComposition
};