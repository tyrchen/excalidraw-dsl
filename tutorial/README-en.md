# Excalidraw DSL Tutorial

Welcome to the Excalidraw DSL tutorial! This guide will teach you how to create beautiful diagrams using a simple text-based domain-specific language (DSL) that compiles to Excalidraw JSON format.

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Basic Syntax](#basic-syntax)
4. [Creating Nodes](#creating-nodes)
5. [Connecting Nodes with Edges](#connecting-nodes-with-edges)
6. [Using Containers](#using-containers)
7. [Working with Groups](#working-with-groups)
8. [Styling Elements](#styling-elements)
9. [Advanced Features](#advanced-features)
10. [Command Line Usage](#command-line-usage)
11. [Examples](#examples)

## Introduction

Excalidraw DSL is a text-based language for creating diagrams. Instead of drawing shapes manually, you describe your diagram using simple text commands, and the tool automatically generates a beautiful Excalidraw diagram for you.

### Why Use Excalidraw DSL?

- **Version Control Friendly**: Text files work perfectly with Git
- **Faster Creation**: Type faster than you can draw
- **Consistent Layouts**: Automatic layout algorithms ensure clean diagrams
- **Reusable Components**: Define templates and reuse them
- **Programmatic Generation**: Generate diagrams from data

## Installation

First, make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

Then clone and build the project:

```bash
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl
cargo build --release
```

The binary will be available at `target/release/edsl`.

## Basic Syntax

The DSL uses a simple, intuitive syntax. Here's the basic structure:

```
# Comments start with #

# Define nodes
node_id "Node Label"

# Connect nodes
node1 -> node2

# That's it for the basics!
```

### File Structure

A typical `.edsl` file has this structure:

```yaml
---
layout: dagre  # Optional configuration
---

# Your diagram definition goes here
```

## Creating Nodes

Nodes are the basic building blocks of your diagrams. Each node has an ID and an optional label.

### Simple Nodes

```
# Node with label
start "Start Here"

# Node without label (ID will be used as label)
process

# Multiple nodes
input "User Input"
process "Process Data"
output "Show Results"
```

### Node IDs

- Must start with a letter
- Can contain letters, numbers, and underscores
- Case sensitive
- Must be unique

Examples:
```
user_input "User Input"
step1 "First Step"
dataStore "Database"
API_endpoint "REST API"
```

## Connecting Nodes with Edges

Edges connect nodes together. Excalidraw DSL supports various arrow types and styles.

### Basic Connections

```
# Simple arrow
start -> process

# Arrow with label
process -> output "Results"

# Chain multiple connections
input -> process -> output
```

### Arrow Types

```
# Single arrow (default)
a -> b

# Double arrow
a <-> b

# No arrow (line only)
a --- b
```

### Edge Chains

You can create multiple connections in one line:

```
# Creates: a->b, b->c, c->d
a -> b -> c -> d

# With labels
start -> process "Step 1" -> validate "Check" -> end
```

## Using Containers

Containers group related nodes together visually.

### Basic Container

```
container {
    node1 "First Node"
    node2 "Second Node"
    node1 -> node2
}
```

### Named Containers

```
container backend "Backend Services" {
    api "API Server"
    db "Database"
    cache "Redis Cache"

    api -> db
    api -> cache
}
```

### Nested Containers

```
container system "System Architecture" {
    container frontend "Frontend" {
        ui "React App"
        state "Redux Store"
    }

    container backend "Backend" {
        api "API Server"
        db "PostgreSQL"
    }

    # Connect across containers
    ui -> api
}
```

## Working with Groups

Groups are logical collections of nodes that can be styled together.

### Basic Groups

```
group team {
    alice "Alice"
    bob "Bob"
    charlie "Charlie"
}

# Connect group members to external nodes
alice -> task1
bob -> task2
```

### Semantic Groups

Groups can have semantic meaning:

```
group services {
    auth "Auth Service"
    payment "Payment Service"
    notification "Notification Service"
}

group databases {
    userDB "User Database"
    orderDB "Order Database"
}

# Connect between groups
auth -> userDB
payment -> orderDB
```

## Styling Elements

Customize the appearance of your diagrams with style attributes.

### Node Styles

```
# Colored node
server "Web Server" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
    textColor: "#ffffff"
}

# Rounded corners
database "PostgreSQL" {
    roundness: 3
}
```

### Edge Styles

```
# Dashed connection
client -> server {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
}

# Thick important connection
server -> database {
    strokeWidth: 3
    strokeColor: "#ff6b6b"
}
```

### Global Styles

Set default styles in the configuration:

```yaml
---
layout: dagre
font: "Cascadia"
strokeWidth: 2
background_color: "#ffffff"
---
```

### Available Style Properties

**For Nodes:**
- `backgroundColor`: Hex color (e.g., "#ff6b6b")
- `strokeColor`: Border color
- `strokeWidth`: Border thickness (1-4)
- `textColor`: Text color
- `roughness`: Hand-drawn effect (0-2)
- `roundness`: Corner roundness (0-3)
- `font`: Font family ("Virgil", "Helvetica", "Cascadia")

**For Edges:**
- `strokeColor`: Line color
- `strokeWidth`: Line thickness
- `strokeStyle`: "solid", "dashed", or "dotted"
- `startArrowhead`: "triangle", "dot", "diamond", "none"
- `endArrowhead`: "triangle", "dot", "diamond", "none"

## Advanced Features

### Templates

Define reusable component templates:

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

# Use template
microservice user_service {
    name: "User"
}

microservice order_service {
    name: "Order"
}

# Connect services
user_service.api -> order_service.api
```

### Component Types

Define custom component types:

```yaml
---
component_types:
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
    roundness: 2

  service:
    backgroundColor: "#f3e5f5"
    strokeColor: "#7b1fa2"
---

# Use component types
auth "Auth Service" @service
userDB "User Database" @database

auth -> userDB
```

### Layout Algorithms

Choose different layout algorithms:

```yaml
---
layout: force  # Options: dagre, force, elk, manual
layout_options:
  rankdir: "TB"  # Top-to-bottom (or LR, RL, BT)
  nodesep: 100   # Node separation
  ranksep: 150   # Rank separation
---
```

### Edge Routing

Control how edges are drawn:

```
# Orthogonal routing (right angles)
a -> b @orthogonal

# Curved routing
c -> d @curved

# Straight line (default)
e -> f @straight
```

## Command Line Usage

### Basic Compilation

```bash
# Compile to Excalidraw JSON
edsl input.edsl -o output.excalidraw

# With specific layout
edsl input.edsl -o output.excalidraw --layout elk
```

### Watch Mode

Automatically recompile when the file changes:

```bash
edsl input.edsl -o output.excalidraw --watch
```

### Web Server Mode

Start a web server for live preview:

```bash
edsl --server
# Opens at http://localhost:3030
```

### Validation

Check syntax without generating output:

```bash
edsl input.edsl --validate
```

### CLI Options

```
Usage: edsl [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input .edsl file

Options:
  -o, --output <OUTPUT>           Output file path
  -l, --layout <LAYOUT>           Layout algorithm [default: dagre]
  -w, --watch                     Watch for file changes
  -s, --server                    Start web server
  -p, --port <PORT>              Server port [default: 3030]
  -v, --validate                  Validate only
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples

### Example 1: Simple Flow Diagram

```
# Simple authentication flow
start "User Login"
auth "Authenticate"
success "Dashboard"
failure "Error Page"

start -> auth
auth -> success "Valid"
auth -> failure "Invalid"
```

### Example 2: System Architecture

```yaml
---
layout: dagre
font: "Helvetica"
---

container frontend "Frontend Layer" {
    web "Web App"
    mobile "Mobile App"
}

container backend "Backend Layer" {
    api "API Gateway"
    auth "Auth Service"
    users "User Service"

    api -> auth
    api -> users
}

container data "Data Layer" {
    postgres "PostgreSQL"
    redis "Redis Cache"
    s3 "S3 Storage"
}

# Connect layers
web -> api
mobile -> api
auth -> postgres
auth -> redis
users -> postgres
users -> s3
```

### Example 3: Microservices with Styling

```yaml
---
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
    roundness: 2
---

# Services
auth "Auth Service" @service
user "User Service" @service
order "Order Service" @service
payment "Payment Service" @service

# Databases
authDB "Auth DB" @database
userDB "User DB" @database
orderDB "Order DB" @database

# Message Queue
queue "RabbitMQ" {
    backgroundColor: "#fff3e0"
    strokeColor: "#e65100"
}

# Connections
auth -> authDB
user -> userDB
order -> orderDB

# Service communication
user -> order "Get Orders"
order -> payment "Process Payment"
payment -> queue "Payment Event"
auth -> queue "Login Event"
```

### Example 4: State Machine

```
# State machine for order processing
initial "New Order"
pending "Pending Payment"
paid "Paid"
processing "Processing"
shipped "Shipped"
delivered "Delivered"
cancelled "Cancelled"

initial -> pending "Submit"
pending -> paid "Payment Success"
pending -> cancelled "Payment Failed"
paid -> processing "Start Fulfillment"
processing -> shipped "Ship Order"
shipped -> delivered "Confirm Delivery"

# Cancellation possible from multiple states
paid -> cancelled "Cancel"
processing -> cancelled "Cancel"
```

### Example 5: Network Topology

```yaml
---
layout: force
---

container cloud "Cloud Infrastructure" {
    lb "Load Balancer"

    container servers "App Servers" {
        app1 "App Server 1"
        app2 "App Server 2"
        app3 "App Server 3"
    }

    lb -> app1
    lb -> app2
    lb -> app3
}

container onprem "On-Premise" {
    corp "Corporate Network"
    vpn "VPN Gateway"
}

# External connections
internet "Internet"
users "Users"

users -> internet
internet -> lb
corp -> vpn
vpn -> lb "Secure Connection" {
    strokeStyle: "dashed"
    strokeColor: "#f03e3e"
}
```

## Tips and Best Practices

1. **Use Meaningful IDs**: Choose descriptive IDs that make the DSL readable
2. **Organize with Containers**: Group related nodes in containers
3. **Consistent Styling**: Define component types for consistent appearance
4. **Comment Your Diagrams**: Use comments to explain complex relationships
5. **Start Simple**: Begin with nodes and edges, add styling later
6. **Use Templates**: For repeated patterns, create templates
7. **Version Control**: Keep your .edsl files in Git
8. **Watch Mode**: Use watch mode during development for instant feedback

## Next Steps

- Explore more examples in the `examples/` directory
- Read the [Language Reference](../docs/language-reference.md) for complete syntax
- Join our community for support and sharing diagrams
- Contribute to the project on GitHub

Happy diagramming!
