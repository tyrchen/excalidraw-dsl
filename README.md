# ExcaliDraw-DSL (EDSL)

A domain-specific language for generating [Excalidraw](https://excalidraw.com/) diagrams. EDSL combines the simplicity of Mermaid-style syntax with native Excalidraw visual properties and hand-drawn aesthetics.

## Features

- **Declarative Syntax**: Focus on "what" to draw, not "how" to draw it
- **Excalidraw-Native**: First-class support for Excalidraw's unique visual properties (roughness, fill styles, fonts)
- **Container Support**: Powerful grouping and hierarchical organization
- **Multiple Layout Algorithms**: Dagre (hierarchical) and Force-directed layouts
- **Progressive Complexity**: Simple basics, powerful advanced features
- **CLI and Library**: Use as a command-line tool or integrate into your applications

## Installation

```bash
# Install from source
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl
cargo install --path .

# Or build locally
cargo build --release
```

## Quick Start

### Basic Usage

Create a simple diagram:

```edsl
# simple.edsl
user[User] -> web[Web Server] -> db[Database] {
  shape: cylinder;
  fill: hachure;
}
```

Compile to Excalidraw JSON:

```bash
edsl simple.edsl -o diagram.json
```

### EDSL Syntax

#### Global Configuration

```edsl
---
layout: dagre              # Layout algorithm (dagre, force)
theme: dark               # Theme (light, dark)
font: Virgil              # Font family
sketchiness: 2            # Hand-drawn appearance (0-2)
---
```

#### Node Definitions

```edsl
# Basic node
web_server

# Node with label
web_server[Web Server]

# Node with styling
db_server[Database] {
  shape: cylinder;
  fill: hachure;
  fillWeight: 2;
  strokeColor: '#868e96';
  roughness: 2;
  backgroundColor: '#f8f9fa';
}
```

#### Edge Definitions

```edsl
# Basic connection
user -> api_gateway

# Labeled connection
user -> api_gateway: "HTTP Request"

# Styled connection
user -> api_gateway: "POST /data" {
  strokeStyle: dotted;
  startArrowhead: dot;
  endArrowhead: triangle;
  strokeColor: '#ff6b35';
}

# Chain connections
A -> B -> C -> D
```

#### Container Definitions

```edsl
container "Backend Services" as backend {
  style: {
    labelPosition: top;
    backgroundColor: '#f8f9fa';
    roughness: 0;
    strokeStyle: dashed;
  }
  
  api_gateway[API Gateway]
  user_service[User Service]
  auth_service[Auth Service]
  
  api_gateway -> user_service;
  api_gateway -> auth_service;
}

# External connections to containers
frontend -> backend.api_gateway;
```

### Visual Properties

#### Shape Types
- `rectangle` (default)
- `ellipse`
- `diamond`
- `cylinder`
- `text`

#### Stroke Properties
- `strokeColor`: Color hex code
- `strokeWidth`: Line thickness
- `strokeStyle`: solid, dotted, dashed

#### Fill Properties
- `backgroundColor`: Fill color
- `fill`: none, solid, hachure, cross-hatch
- `fillWeight`: Hachure density (1-5)

#### Excalidraw-Specific
- `roughness`: 0 (precise) to 2 (very rough)
- `font`: Virgil, Helvetica, Cascadia

## CLI Usage

The `edsl` command provides several subcommands for different workflows:

### Convert (Compile)

Convert EDSL files to Excalidraw JSON:

```bash
# Basic conversion (outputs to input.json)
edsl convert diagram.edsl

# Specify output file
edsl convert diagram.edsl -o output.json

# Choose layout algorithm
edsl convert diagram.edsl --layout force

# Verbose output
edsl convert diagram.edsl -v

# Alias: compile
edsl compile diagram.edsl
```

### Validate

Check EDSL syntax without generating output:

```bash
# Validate syntax
edsl validate diagram.edsl

# Verbose validation (shows element counts)
edsl validate diagram.edsl -v
```

### Watch

Watch EDSL file for changes and recompile automatically:

```bash
# Watch and auto-compile
edsl watch diagram.edsl

# Specify output file
edsl watch diagram.edsl -o output.json

# Verbose mode
edsl watch diagram.edsl -v
```

### Server

Run HTTP/WebSocket server for real-time compilation:

```bash
# Start server on default port (3002)
edsl server

# Specify port
edsl server --port 8080

# Specify host
edsl server --host 127.0.0.1

# Verbose logging
edsl server -v
```

### Usage Examples

```bash
# Convert all examples
make examples

# Run server and UI together
make run-full

# Install globally
cargo install --path . --features server
```

## Library Usage

```rust
use excalidraw_dsl::EDSLCompiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let edsl_source = r#"
    ---
    layout: dagre
    ---
    
    user[User] -> api[API] -> db[Database]
    "#;
    
    let compiler = EDSLCompiler::new();
    let json_output = compiler.compile(edsl_source)?;
    
    println!("{}", json_output);
    Ok(())
}
```

### Advanced Features

#### LLM Layout Optimization (Optional)

Enable AI-powered layout optimization:

```rust
use excalidraw_dsl::EDSLCompiler;

let compiler = EDSLCompiler::new()
    .with_llm_optimization("your-openai-api-key".to_string());
    
let optimized_output = compiler.compile(edsl_source)?;
```

## Examples

See the `examples/` directory for more complex diagrams:

- `examples/simple.edsl` - Basic three-tier architecture
- `examples/containers.edsl` - Using containers and grouping
- `examples/styling.edsl` - Advanced styling examples

## Architecture

EDSL follows a clean pipeline architecture:

```
EDSL Source → Parser → IGR → Layout Engine → Generator → Excalidraw JSON
                      ↓
                 Validation
```

1. **Parser**: pest-based grammar parser
2. **IGR**: Intermediate Graph Representation using petgraph
3. **Layout Engine**: Multiple algorithms (Dagre, Force-directed)
4. **Generator**: ExcalidrawElementSkeleton generation
5. **Validation**: Comprehensive error checking

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Build without LLM features
cargo build --no-default-features
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure all tests pass
5. Submit a pull request

## Roadmap

- [ ] More layout algorithms (ELK, custom layouts)
- [ ] Web-based playground
- [ ] VS Code extension
- [ ] Additional shape types
- [ ] Animation support
- [ ] Export to other formats

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

## Related Projects

- [Excalidraw](https://excalidraw.com/) - The amazing whiteboard tool
- [Mermaid](https://mermaid-js.github.io/) - Inspiration for declarative syntax
- [D2](https://d2lang.com/) - Modern diagram scripting language

## Acknowledgments

This project is inspired by the excellent work of the Excalidraw team and the broader diagram-as-code community.
