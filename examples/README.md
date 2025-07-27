# Excalidraw DSL Examples

This directory contains examples demonstrating the capabilities of the Excalidraw DSL, organized by complexity and use case. Each example showcases different features and architectural patterns suitable for system design and documentation.

## 📚 Getting Started

### Quick Start
```bash
# Compile any example to Excalidraw JSON
cargo run -- examples/basic-flow.edsl

# Specify output file
cargo run -- examples/microservices-architecture.edsl -o output.excalidraw

# Enable different layout algorithms
cargo run -- examples/cloud-native-platform.edsl --layout force
```

## 🎯 Examples by Complexity

### 🟢 Beginner Level

**[basic-flow.edsl](./basic-flow.edsl)** - *Process Flow*
- Simple workflow with decision points
- Node styling and colors
- Edge labels and routing
- **Features**: Basic nodes, styled connections, conditional flows

**[component-types-simple.edsl](./component-types-simple.edsl)** - *Component System*
- Component type definitions
- Reusable styling patterns
- **Features**: Component types, style inheritance

### 🟡 Intermediate Level

**[microservices-simple.edsl](./microservices-simple.edsl)** - *Microservices Overview*
- Service decomposition patterns
- Container organization
- Inter-service communication
- **Features**: Containers, service mesh representation

**[decision-tree.edsl](./decision-tree.edsl)** - *Decision Logic*
- Complex branching logic
- Hierarchical decision making
- **Features**: Tree structures, conditional routing

**[state-machine-simple.edsl](./state-machine-simple.edsl)** - *State Management*
- State transitions
- Event-driven flows
- **Features**: State machines, event handling

### 🔴 Advanced Level

**[microservices-architecture.edsl](./microservices-architecture.edsl)** - *Complete Microservices Platform*
- Multi-layer architecture (Frontend, API Gateway, Services, Data)
- External service integrations
- Message queuing and caching
- **Features**: Complex containers, cross-layer connections, external dependencies

**[system-architecture.edsl](./system-architecture.edsl)** - *Enterprise System Architecture*
- Layered architecture pattern
- Analytics pipeline integration
- Data flow visualization
- **Features**: Multi-tier architecture, data pipelines, analytics components

**[pingora-architecture.edsl](./pingora-architecture.edsl)** - *Proxy Architecture*
- Load balancing strategies
- Proxy server configurations
- Network traffic routing
- **Features**: Network topology, load balancers

## 🏗️ Examples by Architecture Pattern

### Cloud-Native & DevOps

**[cloud-native-platform.edsl](./cloud-native-platform.edsl)** - *Cloud-Native Application Platform*
- Kubernetes-native architecture
- Service mesh (Istio/Linkerd)
- GitOps deployment pipeline
- Observability stack (Prometheus, Grafana, Jaeger)
- **Features**: Cloud services, orchestration, monitoring

**[devops-pipeline.edsl](./devops-pipeline.edsl)** - *CI/CD Pipeline*
- Complete DevOps workflow
- Build, test, deploy stages
- Infrastructure as Code
- **Features**: Pipeline stages, automation, infrastructure

**[event-driven-ecommerce.edsl](./event-driven-ecommerce.edsl)** - *Event-Driven E-commerce*
- Event sourcing patterns
- CQRS implementation
- Saga pattern for distributed transactions
- **Features**: Event streams, command/query separation

### Enterprise & Distributed Systems

**[distributed-data-platform.edsl](./distributed-data-platform.edsl)** - *Data Platform Architecture*
- Data lake and warehouse integration
- Real-time and batch processing
- ML pipeline integration
- **Features**: Data flow, processing pipelines, ML components

**[zero-trust-security.edsl](./zero-trust-security.edsl)** - *Zero Trust Security Architecture*
- Identity and access management
- Network segmentation
- Security monitoring
- **Features**: Security zones, identity flows, threat detection

**[multi-region-deployment.edsl](./multi-region-deployment.edsl)** - *Multi-Region Deployment*
- Global load balancing
- Cross-region replication
- Disaster recovery patterns
- **Features**: Geographic distribution, failover, replication

### Monitoring & Observability

**[observability-stack.edsl](./observability-stack.edsl)** - *Complete Observability Platform*
- Metrics, logs, and traces (Three Pillars)
- Alerting and incident management
- SLI/SLO monitoring
- **Features**: Monitoring components, alert flows, dashboards

**[chaos-engineering.edsl](./chaos-engineering.edsl)** - *Chaos Engineering Platform*
- Fault injection systems
- Resilience testing
- Monitoring and recovery
- **Features**: Failure scenarios, testing frameworks, recovery patterns

## 🎨 Feature Showcase

### Core Features

**[edge-chains.edsl](./edge-chains.edsl)** - *Edge Chain Syntax*
- Demonstrates `A -> B -> C -> D` syntax
- Multiple connection types in chains
- **Features**: Edge chains, connection shortcuts

**[groups-demo.edsl](./groups-demo.edsl)** - *Grouping and Organization*
- Semantic grouping
- Visual organization
- Group styling and themes
- **Features**: Groups, semantic organization, visual hierarchy

**[nested-containers.edsl](./nested-containers.edsl)** - *Container Nesting*
- Hierarchical container structures
- Complex organizational patterns
- **Features**: Nested containers, hierarchy, organization

**[routing-showcase.edsl](./routing-showcase.edsl)** - *Advanced Routing*
- Different routing algorithms
- Connection optimization
- Path finding with obstacles
- **Features**: Routing types, path optimization, obstacle avoidance

### Advanced Styling

**[advanced-connections.edsl](./advanced-connections.edsl)** - *Connection Styling*
- Custom connection styles
- Different arrow types
- Connection routing options
- **Features**: Connection customization, styling, routing

**[component-types.edsl](./component-types.edsl)** - *Advanced Component Types*
- Complex component definitions
- Style inheritance and overrides
- **Features**: Advanced component systems, style management

## 🚀 Layout Algorithms

Most examples support multiple layout algorithms. Experiment with different layouts:

- **Dagre** (default): Hierarchical, directed graph layout
- **Force**: Physics-based layout for complex relationships
- **ELK**: Advanced layout with multiple algorithms
- **Manual**: Custom positioning for precise control

### Layout Examples
```bash
# Compare different layouts for the same diagram
cargo run -- examples/microservices-architecture.edsl --layout dagre
cargo run -- examples/microservices-architecture.edsl --layout force
cargo run -- examples/microservices-architecture.edsl --layout elk
```

## 📖 Learning Path

1. **Start with basic-flow.edsl** to understand fundamental syntax
2. **Progress to microservices-simple.edsl** for containers and organization
3. **Explore component-types.edsl** for reusable patterns
4. **Study microservices-architecture.edsl** for complex systems
5. **Examine cloud-native-platform.edsl** for modern architectures
6. **Review specialized examples** for specific patterns you need

## 🔧 Common Patterns

### Container Organization
```edsl
container "Layer Name" {
    service1[Service One]
    service2[Service Two]

    service1 -> service2
}
```

### Component Types
```edsl
component Database {
    shape: cylinder;
    backgroundColor: "#fde047";
    strokeColor: "#facc15";
}

main_db: Database[Main Database]
```

### Styled Connections
```edsl
api -> database: "SQL Query" {
    routing: orthogonal;
    strokeStyle: dashed;
    strokeColor: "#ef4444";
}
```

### Configuration Options
```yaml
---
layout: dagre
direction: TB
theme: light
node_spacing: 100
edge_spacing: 50
---
```

## 💡 Tips for Creating Effective Diagrams

1. **Use consistent naming** - Keep node IDs descriptive and consistent
2. **Group related components** - Use containers for logical grouping
3. **Style meaningfully** - Use colors and shapes to convey meaning
4. **Label connections** - Add descriptions to important connections
5. **Choose appropriate layouts** - Dagre for hierarchies, Force for networks
6. **Keep it readable** - Avoid overcrowding, use spacing effectively

## 🤝 Contributing Examples

When adding new examples:
1. Include comprehensive comments explaining the architecture
2. Use realistic, production-like scenarios
3. Demonstrate specific features or patterns
4. Add the example to this README in the appropriate section
5. Include both .edsl source and generated .excalidraw files

## 🔗 Related Resources

- [Excalidraw DSL Documentation](../README.md)
- [Language Reference](../docs/language-reference.md)
- [Best Practices Guide](../docs/best-practices.md)
- [Contributing Guidelines](../CONTRIBUTING.md)
