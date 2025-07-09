# GLSP-MCP Architecture Documentation

## Overview

GLSP-MCP is a sophisticated AI-native graphical modeling platform that combines multiple advanced technologies to provide a comprehensive solution for diagram creation, manipulation, and simulation. This document describes the actual implemented architecture, which is significantly more advanced than the simplified documentation suggests.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend Layer                                │
│  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────────┐   │
│  │  TypeScript UI  │  │ Canvas Render│  │   AI Agent Client   │   │
│  │   (React/Vite)  │  │   (Konva.js) │  │  (Ollama Bridge)    │   │
│  └────────┬────────┘  └──────┬───────┘  └──────────┬──────────┘   │
│           └───────────────────┴──────────────────────┘              │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ MCP Protocol (JSON-RPC)
┌───────────────────────────────┴─────────────────────────────────────┐
│                         Backend Layer (Rust)                         │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    MCP Server Implementation                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │   │
│  │  │Resources  │  │  Tools   │  │ Prompts  │  │Transport │   │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Core Services & Components                       │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐   │   │
│  │  │  Diagram    │  │    WASM     │  │   Simulation     │   │   │
│  │  │  Manager    │  │  Runtime    │  │     Engine       │   │   │
│  │  └─────────────┘  └─────────────┘  └──────────────────┘   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐   │   │
│  │  │  Pipeline   │  │   Security  │  │  Sensor Bridge   │   │   │
│  │  │   Engine    │  │   Scanner   │  │                  │   │   │
│  │  └─────────────┘  └─────────────┘  └──────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────────┐
│                      Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────┐  │
│  │ PostgreSQL  │  │  InfluxDB   │  │    Redis    │  │  SQLite  │  │
│  │  (Diagrams) │  │(Time-series)│  │  (Caching)  │  │  (Local) │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └──────────┘  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   WASM Component Storage                     │   │
│  │  (20+ components: sensors, AI, control, graphics, etc.)     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. MCP Server Implementation

The MCP server is the heart of the system, providing:

- **Full MCP 0.3.0 Protocol Support**: Resources, Tools, and Prompts
- **Multi-Transport Support**: HTTP, HTTP-streaming, WebSocket, stdio
- **Async Architecture**: Built on Tokio for high performance
- **Comprehensive Tool Set**:
  - `create_diagram`: Create new diagrams
  - `create_node`: Add nodes to diagrams
  - `create_edge`: Connect nodes
  - `delete_element`: Remove elements
  - `update_element`: Modify properties
  - `apply_layout`: Auto-layout algorithms
  - `export_diagram`: Export to various formats

### 2. WASM Runtime Integration

Advanced WebAssembly runtime capabilities:

- **Wasmtime Integration**: Full wasmtime runtime for server-side execution
- **Component Model Support**: WASI Preview 2 components
- **Security Sandboxing**: Complete isolation with resource limits
- **WIT Interface Analysis**: Automatic interface discovery and validation
- **Execution Engine**:
  ```rust
  pub struct WasmExecutionEngine {
      runtime: Arc<WasmRuntime>,
      thread_pool: ThreadPool,
      active_executions: Arc<Mutex<HashMap<Uuid, ExecutionHandle>>>,
      resource_manager: Arc<ResourceManager>,
  }
  ```

### 3. Database Layer

Multi-backend database support with factory pattern:

- **Supported Backends**:
  - PostgreSQL: Relational data, diagram storage
  - InfluxDB: Time-series sensor data
  - Redis: Caching and session management
  - SQLite: Local development and testing
  - Mock: Unit testing

- **Features**:
  - Connection pooling
  - Health monitoring
  - Transaction support
  - Dataset management abstraction
  - Sensor data bridges

### 4. Simulation Framework

Comprehensive simulation capabilities:

```rust
pub struct SimulationEngine {
    pipelines: HashMap<String, WasmPipelineEngine>,
    sensor_bridge: Arc<SensorDataBridge>,
    scenarios: Vec<SimulationScenario>,
    execution_state: SimulationState,
}
```

- **Time-Driven Simulation**: Deterministic execution with configurable time steps
- **Scenario Management**: Complex multi-pipeline scenarios
- **Sensor Data Integration**: Real-time and replay modes
- **Resource Management**: CPU, memory, and I/O limits
- **Performance Monitoring**: Detailed metrics collection

### 5. Pipeline Engine

Component composition and orchestration:

- **Pipeline Configuration**: Define complex component graphs
- **Data Flow Management**: Inter-component data routing
- **Parallel Execution**: Automatic parallelization
- **Dependency Resolution**: Topological sorting
- **Error Handling**: Fault tolerance and recovery

### 6. Security Scanner

Comprehensive WASM component security:

- **Static Analysis**: Bytecode inspection
- **Import Validation**: Capability checking
- **Resource Analysis**: Memory and CPU profiling
- **Vulnerability Detection**: Known CVE scanning
- **Policy Enforcement**: Configurable security policies

## Data Flow Architecture

### 1. Diagram Creation Flow

```
User Input → Frontend → MCP Tool Call → Backend Processing
                                            ↓
                                    Diagram Manager
                                            ↓
                                    Database Storage
                                            ↓
                                    Event Notification → Frontend Update
```

### 2. WASM Component Execution Flow

```
Pipeline Config → Pipeline Engine → Component Discovery
                                          ↓
                                    Security Scan
                                          ↓
                                    Component Loading
                                          ↓
                    ┌─────────────────────┴─────────────────────┐
                    ↓                                           ↓
            Sensor Bridge ← Database                    WASM Runtime
                    ↓                                           ↓
            Sensor Data Stream                         Component Execution
                    ↓                                           ↓
            Component Input ──────────────────────> Processing Logic
                                                               ↓
                                                        Output Data
                                                               ↓
                                                    Next Component/Storage
```

### 3. Simulation Execution Flow

```
Simulation Config → Scenario Loader → Pipeline Instantiation
                                            ↓
                                    Timing Controller
                                            ↓
                    ┌───────────────────────┴───────────────────────┐
                    ↓                                               ↓
            Sensor Data Replay                              Component Graph
                    ↓                                               ↓
            Time Synchronization ←─────────────────→ Execution Scheduling
                    ↓                                               ↓
            Frame Generation                              Result Collection
                    ↓                                               ↓
            Statistics ←──────────────────────────────── Performance Metrics
```

## Component Composition

### WebAssembly Component Model

The system uses the WebAssembly Component Model for composition:

1. **WIT Interfaces**: Type-safe component contracts
2. **WAC Format**: WebAssembly Composition for linking
3. **Bazel Build System**: Reproducible builds
4. **Component Registry**: Discovery and management

### Example: ADAS System Composition

```
┌─────────────────────────────────────────────────────────────┐
│                    ADAS Complete System                      │
│                                                              │
│  Sensors → AI Processing → Fusion → Control → Actuators    │
│     ↓           ↓            ↓         ↓          ↓        │
│  6 types    2 models    3 stages   2 logic    CAN/HMI     │
│                                                              │
│  Total: 20+ interconnected WASM components                  │
└─────────────────────────────────────────────────────────────┘
```

## Performance Architecture

### Async Execution

- **Tokio Runtime**: Multi-threaded async execution
- **Thread Pool**: Dedicated pools for WASM execution
- **Backpressure**: Automatic flow control
- **Resource Limits**: Per-component constraints

### Caching Strategy

- **Redis Layer**: Hot data caching
- **Component Cache**: Pre-loaded WASM modules
- **Result Cache**: Computation memoization
- **Sensor Buffer**: Time-series data buffering

### Scalability

- **Horizontal Scaling**: Multiple backend instances
- **Load Balancing**: Request distribution
- **Database Sharding**: Data partitioning
- **Component Distribution**: Remote execution

## Security Architecture

### Defense in Depth

1. **Network Security**: TLS, authentication, authorization
2. **WASM Sandboxing**: Complete isolation
3. **Resource Limits**: Memory, CPU, I/O caps
4. **Input Validation**: Type and range checking
5. **Audit Logging**: Complete operation history

### Component Security

- **Capability-Based**: Explicit permission grants
- **Static Analysis**: Pre-execution validation
- **Runtime Monitoring**: Behavior tracking
- **Policy Enforcement**: Configurable rules

## Deployment Architecture

### Container Support

```dockerfile
# Multi-stage build for optimization
FROM rust:latest as builder
# Build backend
FROM node:18 as frontend-builder  
# Build frontend
FROM debian:slim
# Runtime with minimal footprint
```

### Configuration Management

- **Environment Variables**: Runtime configuration
- **Config Files**: Complex settings
- **Secret Management**: Secure credential storage
- **Feature Flags**: Progressive rollout

## Integration Points

### AI Integration

- **Ollama Bridge**: Local LLM integration
- **MCP Protocol**: Universal AI compatibility
- **Prompt Engineering**: Optimized prompts
- **Result Processing**: AI output parsing

### External Systems

- **REST API**: HTTP/JSON interface
- **WebSocket**: Real-time updates
- **Database Connectors**: Multiple backends
- **File System**: Component storage

## Monitoring and Observability

### Metrics Collection

- **Performance Metrics**: Latency, throughput
- **Resource Metrics**: CPU, memory, I/O
- **Business Metrics**: Diagram operations
- **Error Metrics**: Failure tracking

### Logging

- **Structured Logging**: JSON format
- **Log Levels**: Configurable verbosity
- **Correlation IDs**: Request tracking
- **Log Aggregation**: Centralized storage

## Future Architecture Considerations

### Planned Enhancements

1. **Distributed Execution**: Multi-node component execution
2. **GPU Acceleration**: WASI-NN GPU support
3. **Event Streaming**: Kafka/Pulsar integration
4. **GraphQL API**: Alternative query interface
5. **Plugin System**: Extensible architecture

### Scalability Roadmap

1. **Microservices**: Service decomposition
2. **Event Sourcing**: State management
3. **CQRS**: Read/write separation
4. **Service Mesh**: Advanced networking

## Conclusion

The GLSP-MCP architecture is a sophisticated, production-ready system that goes far beyond simple diagram creation. It provides a complete platform for AI-native graphical modeling with advanced features including WASM component execution, multi-database support, comprehensive simulation capabilities, and enterprise-grade security.

The modular architecture ensures extensibility while maintaining performance and security. The use of modern technologies like Rust, WebAssembly, and async programming provides a solid foundation for future growth.