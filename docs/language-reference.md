# Excalidraw DSL Language Reference

This document provides a complete reference for the Excalidraw DSL syntax and features.

[中文版本](./language-reference-zh.md)

## Table of Contents

1. [File Structure](#file-structure)
2. [Comments](#comments)
3. [Nodes](#nodes)
4. [Edges](#edges)
5. [Containers](#containers)
6. [Groups](#groups)
7. [Styling](#styling)
8. [Component Types](#component-types)
9. [Templates](#templates)
10. [Layout Configuration](#layout-configuration)
11. [Attributes Reference](#attributes-reference)
12. [Examples](#examples)

## File Structure

An Excalidraw DSL file consists of two main sections:

```yaml
---
# Optional YAML front matter for configuration
layout: dagre
font: "Cascadia"
---

# DSL content goes here
node1 "Node 1"
node2 "Node 2"
node1 -> node2
```

### Front Matter

The YAML front matter section is optional and enclosed between `---` markers. It can contain:

- Global configuration options
- Component type definitions
- Template definitions
- Layout settings

## Comments

Comments start with `#` and continue to the end of the line:

```edsl
# This is a comment
node1 "Node 1"  # This is also a comment
```

## Nodes

### Basic Node Syntax

```edsl
# Node with ID only (ID will be used as label)
node_id

# Node with label
node_id "Node Label"

# Node with attributes
node_id "Node Label" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
}

# Node with component type
node_id "Node Label" @service
```

### Node ID Rules

- Must start with a letter or underscore
- Can contain letters, numbers, and underscores
- Case sensitive
- Must be unique within the diagram

### Valid Examples

```edsl
user_service
auth2
_internal_node
Service123
```

## Edges

### Basic Edge Syntax

```edsl
# Simple edge
source -> target

# Edge with label
source -> target "Edge Label"

# Edge with attributes
source -> target {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
}

# Edge with label and attributes
source -> target "Label" {
    strokeWidth: 2
}
```

### Arrow Types

```edsl
# Single arrow (default)
a -> b

# Double arrow
a <-> b

# No arrow (line only)
a --- b
```

### Edge Chains

Create multiple edges in one statement:

```edsl
# Creates a->b, b->c, c->d
a -> b -> c -> d

# With labels
start -> process "Step 1" -> validate "Check" -> end

# Mixed arrow types
a -> b <-> c --- d
```

### Edge Routing

Control how edges are drawn:

```edsl
# Straight line (default)
a -> b

# Orthogonal (right angles)
a -> b @orthogonal

# Curved
a -> b @curved
```

## Containers

Containers group nodes visually and logically.

### Basic Container Syntax

```edsl
# Anonymous container
container {
    node1 "Node 1"
    node2 "Node 2"
}

# Named container with label
container backend "Backend Services" {
    api "API Server"
    db "Database"
}

# Container with ID and label
container backend_container "Backend Services" {
    # content
}
```

### Container Attributes

```edsl
container services "Services" {
    backgroundColor: "#f8f9fa"
    strokeStyle: "dashed"

    service1 "Service 1"
    service2 "Service 2"
}
```

### Nested Containers

```edsl
container system "System" {
    container frontend "Frontend" {
        ui "UI"
        state "State"
    }

    container backend "Backend" {
        api "API"
        db "DB"
    }
}
```

### Container References

Reference nodes inside containers from outside:

```edsl
container backend "Backend" {
    api "API"
    db "Database"
}

frontend "Frontend"

# Reference container nodes
frontend -> backend.api
```

## Groups

Groups provide logical organization without visual boundaries.

### Basic Group Syntax

```edsl
# Basic group
group team {
    alice "Alice"
    bob "Bob"
}

# Group with label
group developers "Development Team" {
    frontend_dev "Frontend Developer"
    backend_dev "Backend Developer"
}
```

### Group Types

```edsl
# Basic group
group basic_group {
    node1 "Node 1"
}

# Semantic group with type
group services:microservices {
    auth "Auth Service"
    user "User Service"
}
```

### Nested Groups

```edsl
group department {
    group frontend_team {
        alice "Alice"
        bob "Bob"
    }

    group backend_team {
        charlie "Charlie"
        david "David"
    }
}
```

## Styling

### Inline Styles

Apply styles directly to elements:

```edsl
# Node styling
server "Web Server" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
    strokeWidth: 2
    textColor: "#ffffff"
    roughness: 1
    roundness: 2
    font: "Cascadia"
}

# Edge styling
client -> server {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
    strokeWidth: 2
    startArrowhead: "dot"
    endArrowhead: "triangle"
}

# Container styling
container backend "Backend" {
    backgroundColor: "#f8f9fa"
    strokeStyle: "dashed"

    # content
}
```

### Global Styles

Set default styles in the front matter:

```yaml
---
# Global defaults
font: "Virgil"
strokeWidth: 2
roughness: 1
backgroundColor: "#ffffff"
---
```

## Component Types

Define reusable style sets:

```yaml
---
component_types:
  service:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
    roundness: 2

  database:
    backgroundColor: "#fce4ec"
    strokeColor: "#c2185b"
    roundness: 1

  queue:
    backgroundColor: "#f3e5f5"
    strokeColor: "#7b1fa2"
---

# Use component types
auth_service "Auth Service" @service
user_db "User Database" @database
message_queue "Message Queue" @queue
```

## Templates

Create reusable component structures:

```yaml
---
templates:
  microservice:
    # Template nodes
    api: "$name API"
    db: "$name Database"
    cache: "$name Cache"

    # Template edges
    edges:
      - api -> db
      - api -> cache

  crud_service:
    controller: "$name Controller"
    service: "$name Service"
    repository: "$name Repository"
    model: "$name Model"

    edges:
      - controller -> service
      - service -> repository
      - repository -> model
---

# Use templates
microservice user_service {
    name: "User"
}

crud_service order_service {
    name: "Order"
}

# Reference template nodes
user_service.api -> order_service.controller
```

## Layout Configuration

### Layout Algorithms

```yaml
---
layout: dagre  # Options: dagre, force, elk
---
```

#### Dagre (Hierarchical)

Best for hierarchical structures, flowcharts, and org charts.

```yaml
---
layout: dagre
layout_options:
  rankdir: "TB"    # TB (top-bottom), BT, LR, RL
  nodesep: 50      # Minimum space between nodes in same rank
  ranksep: 100     # Minimum space between ranks
  marginx: 20      # Horizontal margin
  marginy: 20      # Vertical margin
---
```

#### Force (Force-Directed)

Best for network diagrams and organic layouts.

```yaml
---
layout: force
layout_options:
  iterations: 300     # Number of simulation iterations
  node_repulsion: 50  # Force between nodes
  link_distance: 100  # Ideal edge length
  link_strength: 1    # Edge force strength
---
```

#### ELK (Eclipse Layout Kernel)

Advanced layout with many algorithm options.

```yaml
---
layout: elk
layout_options:
  algorithm: "layered"  # layered, force, stress, mrtree, radial
  direction: "DOWN"     # UP, DOWN, LEFT, RIGHT
  spacing: 50          # Base spacing value
---
```

## Attributes Reference

### Node Attributes

| Attribute | Type | Values | Description |
|-----------|------|--------|-------------|
| `backgroundColor` | color | Hex color | Fill color |
| `strokeColor` | color | Hex color | Border color |
| `strokeWidth` | number | 1-4 | Border thickness |
| `strokeStyle` | string | solid, dashed, dotted | Border style |
| `textColor` | color | Hex color | Text color |
| `font` | string | Virgil, Helvetica, Cascadia | Font family |
| `fontSize` | number | 12-48 | Font size |
| `roughness` | number | 0-2 | Hand-drawn effect |
| `roundness` | number | 0-3 | Corner roundness |
| `fillStyle` | string | solid, hachure, cross-hatch | Fill pattern |

### Edge Attributes

| Attribute | Type | Values | Description |
|-----------|------|--------|-------------|
| `strokeColor` | color | Hex color | Line color |
| `strokeWidth` | number | 1-4 | Line thickness |
| `strokeStyle` | string | solid, dashed, dotted | Line style |
| `startArrowhead` | string | none, triangle, dot, diamond | Start arrow |
| `endArrowhead` | string | none, triangle, dot, diamond | End arrow |
| `curvature` | number | 0-1 | Curve amount (for curved edges) |

### Container Attributes

| Attribute | Type | Values | Description |
|-----------|------|--------|-------------|
| `backgroundColor` | color | Hex color | Fill color |
| `strokeColor` | color | Hex color | Border color |
| `strokeWidth` | number | 1-4 | Border thickness |
| `strokeStyle` | string | solid, dashed, dotted | Border style |
| `textColor` | color | Hex color | Label text color |
| `font` | string | Virgil, Helvetica, Cascadia | Font family |
| `padding` | number | pixels | Inner padding |

### Color Values

Colors can be specified as:
- Hex colors: `"#ff6b6b"`, `"#000000"`
- RGB: `"rgb(255, 107, 107)"`
- Named colors: `"red"`, `"blue"` (limited support)

## Examples

### Example 1: Simple Flow

```edsl
start "Start"
process "Process Data"
decision "Valid?"
success "Success"
error "Error"

start -> process
process -> decision
decision -> success "Yes"
decision -> error "No"
```

### Example 2: Microservices with Styling

```yaml
---
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
---

gateway "API Gateway" @service

container services "Services" {
    auth "Auth" @service
    user "User" @service
    order "Order" @service
}

container data "Data Layer" {
    auth_db "Auth DB" @database
    user_db "User DB" @database
    order_db "Order DB" @database
}

gateway -> services.auth
gateway -> services.user
gateway -> services.order

services.auth -> data.auth_db
services.user -> data.user_db
services.order -> data.order_db
```

### Example 3: Using Templates

```yaml
---
templates:
  layer:
    frontend: "$name Frontend"
    backend: "$name Backend"
    database: "$name Database"
    edges:
      - frontend -> backend
      - backend -> database
---

layer user_management {
    name: "User"
}

layer order_management {
    name: "Order"
}

# Cross-layer connections
user_management.backend -> order_management.backend "API Call"
```

### Example 4: Complex Styling

```edsl
# Header with custom style
header "System Architecture" {
    backgroundColor: "#1a1a1a"
    textColor: "#ffffff"
    fontSize: 24
    roundness: 0
}

# Styled container
container critical "Critical Services" {
    backgroundColor: "#ffebee"
    strokeColor: "#d32f2f"
    strokeWidth: 3
    strokeStyle: "solid"

    payment "Payment Service" {
        backgroundColor: "#ff5252"
        textColor: "#ffffff"
    }

    fraud "Fraud Detection" {
        backgroundColor: "#ff1744"
        textColor: "#ffffff"
    }
}

# Styled edge
header -> critical "Manages" {
    strokeStyle: "dashed"
    strokeColor: "#666666"
    strokeWidth: 2
    startArrowhead: "none"
    endArrowhead: "triangle"
}
```

## Best Practices

1. **Use meaningful IDs**: Choose descriptive IDs that make the DSL readable
2. **Organize with containers**: Group related nodes for better visualization
3. **Define component types**: Create consistent styling across your diagram
4. **Use templates**: For repeated patterns, define templates
5. **Comment complex sections**: Add comments to explain complex relationships
6. **Choose appropriate layouts**: Select the layout algorithm that best fits your diagram type

## Grammar Summary

The complete EBNF grammar for Excalidraw DSL:

```ebnf
diagram = [front_matter] statement*

front_matter = "---" yaml_content "---"

statement = node_def | edge_def | container_def | group_def | comment

node_def = identifier [string] ["@" identifier] [attributes]

edge_def = edge_chain [string] [attributes]

edge_chain = node_ref (edge_op node_ref)*

edge_op = "->" | "<->" | "---"

container_def = "container" [identifier] [string] "{" statement* "}"

group_def = "group" identifier [":" identifier] [string] "{" statement* "}"

attributes = "{" (attribute_pair)* "}"

attribute_pair = identifier ":" value

node_ref = identifier | qualified_ref

qualified_ref = identifier "." identifier
```
