# EDSL Improvement Specification

## Overview

This document outlines improvements to the Excalidraw Domain Specific Language (EDSL) based on analysis of complex architectural diagrams. The goal is to make EDSL more expressive and easier to use for creating professional system architecture diagrams.

## Analysis of Current Limitations

Based on the provided diagrams, current EDSL lacks:

1. **Nested Grouping**: Ability to create hierarchical groups with visual boundaries
2. **Advanced Layout Algorithms**: Support for different layout strategies (layered, flow-based, hierarchical)
3. **Rich Styling**: Component styling with fills, borders, shadows, and patterns
4. **Connection Types**: Various arrow styles and routing strategies
5. **Annotations**: Support for labels, notes, and descriptive text
6. **Diagram Types**: Built-in support for common architectural patterns

## Proposed Improvements

### 1. Enhanced Layout System

```edsl
layout {
  type: layered | flow | grid | circular | tree
  direction: horizontal | vertical | radial
  spacing: { x: 50, y: 100 }
  alignment: center | start | end | justify
}
```

### 2. Nested Groups with Visual Boundaries

```edsl
group "DFW Proxy Layer" {
  style {
    background: pattern("diagonal-lines", "#f0f0f0")
    border: dashed(2, "#666")
    padding: 20
    rounded: 10
  }
  
  group "Core Components" {
    layout: grid(2, 3)
    
    component "Proxy Manager" { type: service }
    component "JWT Auth Validator" { type: security }
    component "Rate Limiter" { type: control }
  }
}
```

### 3. Component Types and Styles

```edsl
// Define reusable component types
componentType service {
  shape: rectangle
  style {
    fill: "#e8f5e9"
    border: solid(2, "#4caf50")
    rounded: 8
  }
}

componentType security {
  shape: rectangle
  style {
    fill: "#ffebee"
    border: solid(2, "#f44336")
    rounded: 8
  }
}

componentType database {
  shape: cylinder
  style {
    fill: "#e3f2fd"
    border: solid(2, "#2196f3")
  }
}
```

### 4. Advanced Connection Syntax

```edsl
connection {
  from: "Client Application"
  to: "DFW Proxy Layer"
  style {
    type: arrow | line | dashed | dotted
    label: "HTTPS/H2"
    labelPosition: 0.5  // Position along the line
    routing: straight | orthogonal | curved | auto
    color: "#333"
    width: 2
  }
}

// Bulk connections with pattern
connections {
  from: "Proxy Manager"
  to: ["API Server 1", "API Server 2"]
  style {
    type: arrow
    label: "Load Balance"
    routing: orthogonal
  }
}
```

### 5. Flow Diagram Support

```edsl
flow {
  start: "User"
  
  step "User" -> "CloudFront" {
    label: "request"
  }
  
  decision "CloudFront" {
    case "static" -> "S3/Asset Bucket" {
      label: "Fn(req)"
    }
    case "dynamic" -> "Lambda Function" {
      label: "Fn(res)"
    }
  }
  
  end: "User"
}
```

### 6. Annotations and Labels

```edsl
annotation {
  target: "Lambda Function"
  text: "User code with Common Layer and Env"
  position: bottom
  style {
    fontSize: 12
    color: "#666"
    background: "#fffbf0"
    padding: 5
  }
}

label {
  text: "Load Balance"
  attachTo: connection("Proxy Manager", "API Server 1")
  offset: { x: 0, y: -10 }
}
```

### 7. Diagram Templates

```edsl
template microservices {
  layers {
    "Client Tier" {
      components: ["Web App", "Mobile App"]
    }
    "Gateway Tier" {
      components: ["API Gateway", "Load Balancer"]
    }
    "Service Tier" {
      components: ["Service A", "Service B", "Service C"]
      layout: horizontal
    }
    "Data Tier" {
      components: ["Database", "Cache", "Message Queue"]
    }
  }
  
  connections {
    pattern: "each-to-next-layer"
  }
}
```

### 8. Positioning and Alignment

```edsl
// Absolute positioning
component "Database" {
  position: { x: 500, y: 300 }
}

// Relative positioning
component "Cache" {
  position: relative("Database", { x: 150, y: 0 })
}

// Alignment helpers
align {
  components: ["Service A", "Service B", "Service C"]
  direction: horizontal
  spacing: 50
}
```

## Example: System Architecture Diagram

```edsl
diagram "System Architecture (Pingora)" {
  layout {
    type: layered
    direction: horizontal
    spacing: { x: 100, y: 50 }
  }
  
  // Define styles
  componentType proxy {
    style {
      fill: "#fff3e0"
      border: solid(2, "#ff9800")
      rounded: 8
    }
  }
  
  // Client side
  layer "Client Side" {
    component "Client Application" {
      type: client
    }
  }
  
  // Proxy layer
  layer "DFW Proxy Layer" {
    style {
      background: pattern("diagonal-lines")
      border: dashed(2, "#666")
      padding: 30
    }
    
    component "Data Firewall" {
      position: top
      style { fill: "#c8e6c9" }
    }
    
    group "Core Components" {
      layout: flow
      
      component "Proxy Manager" { type: proxy }
      component "JWT Auth Validator" { type: security }
      component "Rate Limiter" { type: control }
      component "Circuit Breaker" { type: control }
      component "Security Headers" { type: security }
    }
    
    group "Plugin System" {
      layout: vertical
      
      component "JSON-RPC Plugin" { type: plugin }
      component "gRPC Plugin" { type: plugin }
      component "Statsig Plugin" { type: plugin }
    }
  }
  
  // Backend services
  layer "Backend Services" {
    component "API Server 1" { type: service }
    component "API Server 2" { type: service }
    component "gRPC Server" { type: service }
  }
  
  // Connections
  connection {
    from: "Client Application"
    to: "Data Firewall"
    label: "HTTPS/H2"
  }
  
  connections {
    from: "Proxy Manager"
    to: ["API Server 1", "API Server 2"]
    label: "Load Balance"
    style { routing: orthogonal }
  }
  
  connection {
    from: "Core Components"
    to: "gRPC Server"
    label: "H2/H2C"
  }
}
```

## Implementation Priorities

1. **Phase 1**: Basic grouping and improved layout algorithms
2. **Phase 2**: Component types and styling system
3. **Phase 3**: Advanced connections and routing
4. **Phase 4**: Templates and diagram types
5. **Phase 5**: Annotations and advanced features

## Benefits

1. **Reduced Verbosity**: Less code needed for complex diagrams
2. **Better Abstractions**: Higher-level concepts like layers and flows
3. **Reusability**: Component types and templates
4. **Professional Output**: Better visual quality with styling options
5. **Maintainability**: Structured approach to diagram creation

## Backward Compatibility

- Maintain support for current EDSL syntax
- Provide migration tools for existing diagrams
- Allow mixing of old and new syntax during transition period