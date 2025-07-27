# Excalidraw DSL

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/excalidraw-dsl.svg)](https://crates.io/crates/excalidraw-dsl)

A powerful domain-specific language (DSL) for generating [Excalidraw](https://excalidraw.com/) diagrams using text. Write diagrams as code and get beautiful, hand-drawn style visualizations.

[中文文档](./README-zh.md) | [Tutorial](./tutorial/README.md) | [Examples](./examples/)

## ✨ Features

- 📝 **Simple Text Syntax** - Write diagrams using intuitive text commands
- 🎨 **Automatic Layouts** - Multiple layout algorithms (Dagre, Force, ELK)
- 🎯 **Smart Styling** - Consistent styling with component types and themes
- 📦 **Containers & Groups** - Organize complex diagrams with hierarchical structures
- 🔄 **Live Preview** - Built-in web server with real-time updates
- 🚀 **Fast Compilation** - Instant diagram generation
- 🎭 **Hand-drawn Style** - Beautiful Excalidraw aesthetics
- 🌈 **Full Styling Control** - Colors, fonts, line styles, and more

## 🚀 Quick Start

### Installation

```bash
# Install from source
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl
cargo install --path .

# Or install from crates.io (when published)
cargo install excalidraw-dsl
```

### Your First Diagram

Create a file `hello.edsl`:

```
start "Hello"
world "World"
start -> world
```

Compile it:

```bash
edsl hello.edsl -o hello.excalidraw
```

Open `hello.excalidraw` in [Excalidraw](https://excalidraw.com/) and see your diagram!

## 📖 Language Overview

### Basic Syntax

```edsl
# Comments start with #

# Nodes
node_id "Node Label"

# Edges
source -> target
source -> target "Edge Label"

# Containers
container name "Container Label" {
    node1 "Node 1"
    node2 "Node 2"
    node1 -> node2
}

# Styling
styled_node "Styled Node" {
    backgroundColor: "#ff6b6b"
    textColor: "#ffffff"
}
```

### Advanced Features

#### Component Types

Define reusable styles:

```yaml
---
component_types:
  service:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
  database:
    backgroundColor: "#fce4ec"
    strokeColor: "#c2185b"
---

auth "Auth Service" @service
userDB "User Database" @database
auth -> userDB
```

#### Templates

Create reusable components:

```yaml
---
templates:
  microservice:
    api: "$name API"
    db: "$name DB"
    cache: "$name Cache"
    edges:
      - api -> db
      - api -> cache
---

microservice user_service {
    name: "User"
}
```

#### Layout Algorithms

Choose from multiple layout engines:

```yaml
---
layout: dagre  # Options: dagre, force, elk
layout_options:
  rankdir: "TB"  # Top-bottom, LR, RL, BT
  nodesep: 100
  ranksep: 150
---
```

## 🎯 Real-World Example

```yaml
---
layout: dagre
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
    roundness: 2
---

# Microservices Architecture
gateway "API Gateway" @service

container services "Microservices" {
    auth "Auth Service" @service
    user "User Service" @service
    order "Order Service" @service
    payment "Payment Service" @service
}

container databases "Databases" {
    authDB "Auth DB" @database
    userDB "User DB" @database
    orderDB "Order DB" @database
}

queue "Message Queue" {
    backgroundColor: "#fff3e0"
    strokeColor: "#e65100"
}

# Connections
gateway -> auth
gateway -> user
gateway -> order

auth -> authDB
user -> userDB
order -> orderDB

order -> payment "Process Payment"
payment -> queue "Payment Events"
```

## 🛠️ CLI Usage

```bash
# Basic compilation
edsl input.edsl -o output.excalidraw

# Watch mode - auto-recompile on changes
edsl input.edsl -o output.excalidraw --watch

# Start web server for live preview
edsl --server
# Visit http://localhost:3030

# Validate syntax without output
edsl input.edsl --validate

# Use specific layout algorithm
edsl input.edsl -o output.excalidraw --layout elk
```

### All Options

```
Usage: edsl [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input .edsl file

Options:
  -o, --output <OUTPUT>       Output file path
  -l, --layout <LAYOUT>       Layout algorithm [default: dagre]
                             Possible values: dagre, force, elk
  -w, --watch                Watch for file changes
  -s, --server               Start web server
  -p, --port <PORT>          Server port [default: 3030]
  -v, --validate             Validate only
      --watch-delay <MS>      Delay before recompiling [default: 100]
  -h, --help                 Print help
  -V, --version              Print version
```

## 📚 Documentation

- 📖 **[Tutorial](./tutorial/README.md)** - Step-by-step guide for beginners
- 🌏 **[中文教程](./tutorial/README-zh.md)** - Chinese tutorial
- 📝 **[Language Reference](./docs/language-reference.md)** - Complete syntax reference
- 🎨 **[Examples](./examples/)** - Sample diagrams and patterns
- 🏗️ **[Architecture](./docs/architecture.md)** - Technical documentation

## 🧩 Examples

Check out the [examples directory](./examples/) for more complex diagrams:

- [Microservices Architecture](./examples/microservices.edsl)
- [State Machines](./examples/state-machine.edsl)
- [Network Topology](./examples/network.edsl)
- [System Architecture](./examples/system-architecture.edsl)
- [Flow Charts](./examples/flowchart.edsl)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl

# Build the project
cargo build

# Run tests
cargo test

# Run with example
cargo run -- examples/basic.edsl -o output.excalidraw
```

### Project Structure

```
excalidraw-dsl/
├── src/
│   ├── ast.rs          # Abstract syntax tree definitions
│   ├── parser.rs       # Pest-based parser
│   ├── igr.rs          # Intermediate graph representation
│   ├── layout/         # Layout algorithms
│   ├── generator.rs    # Excalidraw JSON generator
│   └── main.rs         # CLI entry point
├── grammar/
│   └── edsl.pest       # Grammar definition
├── examples/           # Example diagrams
├── tests/             # Integration tests
└── tutorial/          # Tutorial and documentation
```

## 🚦 Roadmap

- [ ] **VSCode Extension** - Syntax highlighting and live preview
- [ ] **More Layouts** - Hierarchical, circular, and custom layouts
- [ ] **Theming** - Built-in color themes
- [ ] **Export Formats** - SVG, PNG, PDF export
- [ ] **Interactive Mode** - REPL for diagram creation
- [ ] **Web Playground** - Online editor and compiler
- [ ] **Diagram Libraries** - Reusable diagram components
- [ ] **AI Integration** - Generate diagrams from descriptions

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Excalidraw](https://excalidraw.com/) - For the amazing drawing tool
- [Graphviz](https://graphviz.org/) - Inspiration for the DSL design
- [Mermaid](https://mermaid-js.github.io/) - Ideas for diagram syntax
- [Pest](https://pest.rs/) - Excellent parser generator

---

Made with ❤️ by the Excalidraw DSL community
