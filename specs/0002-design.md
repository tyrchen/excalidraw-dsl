# ExcaliDraw-DSL (EDSL) System Design Document

## Overview

This document presents the detailed system design for ExcaliDraw-DSL (EDSL), a domain-specific language for generating Excalidraw diagrams. The system combines declarative syntax simplicity with native Excalidraw visual expressiveness, built on a robust Rust-based compiler architecture with optional LLM-powered layout optimization.

## 1. System Architecture

### 1.1 High-Level Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐
│   EDSL      │───▶│   Parser     │───▶│     IGR     │───▶│ Layout Engine│
│   Source    │    │   (pest)     │    │ (petgraph)  │    │  (@antv)     │
└─────────────┘    └──────────────┘    └─────────────┘    └──────────────┘
                                                                   │
┌─────────────┐    ┌──────────────┐    ┌─────────────┐           │
│ Excalidraw  │◀───│  Generator   │◀───│ Positioned  │◀──────────┘
│    JSON     │    │              │    │     IGR     │
└─────────────┘    └──────────────┘    └─────────────┘
                                                  ▲
                                                  │
                                     ┌──────────────┐
                                     │ LLM Layout   │
                                     │ Optimizer    │
                                     │ (Optional)   │
                                     └──────────────┘
```

### 1.2 Core Components

1. **Parser**: EDSL text → Intermediate Graph Representation (IGR)
2. **IGR**: Unified graph data structure with metadata
3. **Layout Engine**: IGR → Positioned IGR with coordinates
4. **LLM Layout Optimizer**: Semantic layout refinement (optional)
5. **Generator**: Positioned IGR → ExcalidrawElementSkeleton
6. **Excalidraw Integration**: Skeleton → Final Excalidraw JSON

### 1.3 Technology Stack

- **Language**: Rust (for performance, safety, and WASM compatibility)
- **Parser**: pest (PEG-based, maintainable grammar)
- **Graph Structure**: petgraph (mature, feature-rich)
- **Layout Algorithms**: @antv/layout-rust (unified multi-algorithm library)
- **LLM Integration**: HTTP API client (GPT-4, Claude, etc.)
- **Output Format**: ExcalidrawElementSkeleton → Excalidraw JSON

## 2. EDSL Language Design

### 2.1 Design Philosophy

1. **Declarative Simplicity**: Focus on "what" not "how"
2. **Excalidraw-Native**: First-class support for Excalidraw visual properties
3. **Progressive Complexity**: Easy basics, powerful advanced features
4. **Container-First**: Strong grouping and hierarchical organization

### 2.2 Syntax Specification

#### 2.2.1 Global Configuration (YAML Frontmatter)

```yaml
---
theme: dark
layout: dagre              # dagre, force, elk
font: Virgil               # Excalidraw fonts
sketchiness: 2             # 0: Off, 1: Low, 2: High
strokeWidth: 2
backgroundColor: "#ffffff"
---
```

#### 2.2.2 Node Definitions

```edsl
# Basic node
web_server

# Node with label
web_server[Web Server]

# Node with full styling
db_server[Database] {
  shape: cylinder;
  fill: hachure;
  fillWeight: 2;
  strokeColor: '#868e96';
  roughness: 2;
  backgroundColor: '#f8f9fa';
}
```

#### 2.2.3 Edge Definitions

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

#### 2.2.4 Container Definitions

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
backend -> database;
```

#### 2.2.5 Shape Vocabulary

- `rectangle` (default)
- `ellipse`
- `diamond`
- `cylinder`
- `arrow`
- `line`
- `text`

#### 2.2.6 Visual Properties

**Stroke Properties:**
- `strokeColor`: Color hex code
- `strokeWidth`: Line thickness
- `strokeStyle`: solid, dotted, dashed

**Fill Properties:**
- `backgroundColor`: Fill color
- `fill`: none, solid, hachure, cross-hatch
- `fillWeight`: Hachure density (1-5)

**Excalidraw-Specific:**
- `roughness`: 0 (precise) to 2 (very rough)
- `sketchiness`: Hand-drawn appearance level
- `font`: Virgil, Cascadia, etc.

**Layout Hints:**
- `width`, `height`: Explicit sizing
- `x`, `y`: Manual positioning (overrides layout)

### 2.3 Grammar Definition (pest)

```pest
// edsl.pest - Complete grammar specification
WHITESPACE = _{ " " | "\t" | NEWLINE }
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* }

file = { SOI ~ config? ~ statement* ~ EOI }

// YAML frontmatter configuration
config = { "---" ~ yaml_content ~ "---" }
yaml_content = { (!("---") ~ ANY)* }

// Main statements
statement = { container_def | node_def | edge_def }

// Node definitions
node_def = { id ~ label? ~ style_block? }
label = { "[" ~ label_text ~ "]" }
label_text = @{ (!"[" ~ !"]" ~ ANY)+ }

// Edge definitions  
edge_def = { edge_chain | single_edge }
edge_chain = { id ~ (arrow ~ id)+ ~ edge_label? ~ style_block? }
single_edge = { id ~ arrow ~ id ~ edge_label? ~ style_block? }
edge_label = { ":" ~ string_literal }
arrow = { "->" | "--" | "<->" | "~>" }

// Container definitions
container_def = { 
  "container" ~ string_literal? ~ ("as" ~ id)? ~ "{" ~ 
  container_style? ~ 
  statement* ~ 
  "}" 
}
container_style = { "style:" ~ style_block }

// Style blocks
style_block = { "{" ~ attribute* ~ "}" }
attribute = { property_name ~ ":" ~ property_value ~ ";" }
property_name = @{ (ASCII_ALPHANUMERIC | "_")+ }
property_value = { string_literal | number | color }

// Primitives
id = @{ (ASCII_ALPHANUMERIC | "_" | ".")+ }
string_literal = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
number = @{ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
color = @{ "#" ~ ASCII_HEX_DIGIT{6} }
```

## 3. Compiler Pipeline Design

### 3.1 Parser Implementation

#### 3.1.1 Parser Module Structure

```rust
// src/parser/mod.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "edsl.pest"]
pub struct EDSLParser;

pub struct ParsedDocument {
    pub config: GlobalConfig,
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<EdgeDefinition>, 
    pub containers: Vec<ContainerDefinition>,
}

pub fn parse_edsl(input: &str) -> Result<ParsedDocument, ParseError> {
    let pairs = EDSLParser::parse(Rule::file, input)?;
    // Transform pest pairs into structured data
    Ok(build_document(pairs))
}
```

#### 3.1.2 AST Data Structures

```rust
// src/ast/mod.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub font: Option<String>,
    pub sketchiness: Option<u8>,
    pub stroke_width: Option<f64>,
    pub background_color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NodeDefinition {
    pub id: String,
    pub label: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct EdgeDefinition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub arrow_type: ArrowType,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct ContainerDefinition {
    pub id: Option<String>,
    pub label: Option<String>,
    pub children: Vec<String>, // Node IDs
    pub attributes: HashMap<String, AttributeValue>,
    pub internal_statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Color(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum ArrowType {
    SingleArrow,    // ->
    Line,           // --
    DoubleArrow,    // <->
    WavyArrow,      // ~>
}
```

### 3.2 Intermediate Graph Representation (IGR)

#### 3.2.1 Core IGR Structure

```rust
// src/igr/mod.rs
use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use std::collections::HashMap;

pub struct IntermediateGraph {
    pub graph: DiGraph<NodeData, EdgeData>,
    pub global_config: GlobalConfig,
    pub containers: Vec<ContainerData>,
    pub node_map: HashMap<String, NodeIndex>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    pub attributes: ExcalidrawAttributes,
    // Layout will populate these
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub label: Option<String>,
    pub arrow_type: ArrowType,
    pub attributes: ExcalidrawAttributes,
}

#[derive(Debug, Clone)]
pub struct ContainerData {
    pub id: Option<String>,
    pub label: Option<String>,
    pub children: Vec<NodeIndex>,
    pub attributes: ExcalidrawAttributes,
    pub bounds: Option<BoundingBox>,
}

#[derive(Debug, Clone)]
pub struct ExcalidrawAttributes {
    // Shape properties
    pub shape: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    
    // Stroke properties
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub stroke_style: Option<StrokeStyle>,
    
    // Fill properties
    pub background_color: Option<String>,
    pub fill_style: Option<FillStyle>,
    pub fill_weight: Option<u8>,
    
    // Excalidraw-specific
    pub roughness: Option<u8>,
    pub font: Option<String>,
    pub font_size: Option<f64>,
    
    // Arrow properties
    pub start_arrowhead: Option<ArrowheadType>,
    pub end_arrowhead: Option<ArrowheadType>,
}

#[derive(Debug, Clone)]
pub enum StrokeStyle {
    Solid,
    Dotted, 
    Dashed,
}

#[derive(Debug, Clone)]
pub enum FillStyle {
    None,
    Solid,
    Hachure,
    CrossHatch,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

#### 3.2.2 IGR Builder

```rust
// src/igr/builder.rs
impl IntermediateGraph {
    pub fn from_ast(document: ParsedDocument) -> Result<Self, BuildError> {
        let mut igr = IntermediateGraph::new();
        
        // Build nodes
        for node_def in document.nodes {
            let node_data = NodeData::from_definition(node_def);
            let node_idx = igr.graph.add_node(node_data.clone());
            igr.node_map.insert(node_data.id.clone(), node_idx);
        }
        
        // Build edges
        for edge_def in document.edges {
            let from_idx = igr.node_map.get(&edge_def.from)
                .ok_or(BuildError::UnknownNode(edge_def.from))?;
            let to_idx = igr.node_map.get(&edge_def.to)
                .ok_or(BuildError::UnknownNode(edge_def.to))?;
            
            let edge_data = EdgeData::from_definition(edge_def);
            igr.graph.add_edge(*from_idx, *to_idx, edge_data);
        }
        
        // Build containers
        for container_def in document.containers {
            let container_data = ContainerData::from_definition(
                container_def, &igr.node_map
            )?;
            igr.containers.push(container_data);
        }
        
        igr.global_config = document.config;
        Ok(igr)
    }
}
```

## 4. Layout Engine Design

### 4.1 Multi-Algorithm Layout Engine

#### 4.1.1 Layout Engine Interface

```rust
// src/layout/mod.rs
pub trait LayoutEngine {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<(), LayoutError>;
    fn name(&self) -> &'static str;
}

pub struct LayoutManager {
    engines: HashMap<String, Box<dyn LayoutEngine>>,
}

impl LayoutManager {
    pub fn new() -> Self {
        let mut manager = LayoutManager {
            engines: HashMap::new(),
        };
        
        // Register available layout engines
        manager.register("dagre", Box::new(DagreLayout::new()));
        manager.register("force", Box::new(ForceLayout::new()));
        manager.register("elk", Box::new(ElkLayout::new()));
        
        manager
    }
    
    pub fn layout(&self, igr: &mut IntermediateGraph) -> Result<(), LayoutError> {
        let layout_name = igr.global_config.layout
            .as_deref()
            .unwrap_or("dagre");
            
        let engine = self.engines.get(layout_name)
            .ok_or(LayoutError::UnknownEngine(layout_name.to_string()))?;
            
        engine.layout(igr)
    }
}
```

#### 4.1.2 Dagre Layout Implementation

```rust
// src/layout/dagre.rs
use antv_layout::{DagreLayout as AntVDagre, LayoutOptions};

pub struct DagreLayout {
    options: DagreLayoutOptions,
}

#[derive(Debug, Clone)]
pub struct DagreLayoutOptions {
    pub node_sep: f64,
    pub edge_sep: f64, 
    pub rank_sep: f64,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl LayoutEngine for DagreLayout {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<(), LayoutError> {
        // Convert IGR to AntV layout format
        let layout_graph = self.convert_to_antv_format(igr)?;
        
        // Apply Dagre layout
        let mut dagre_engine = AntVDagre::new();
        dagre_engine.set_options(LayoutOptions {
            node_sep: self.options.node_sep,
            edge_sep: self.options.edge_sep,
            rank_sep: self.options.rank_sep,
            rankdir: self.convert_direction(&self.options.direction),
        });
        
        let positioned_graph = dagre_engine.layout(layout_graph)?;
        
        // Update IGR with calculated positions
        self.update_igr_positions(igr, positioned_graph)?;
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "dagre"
    }
}

impl DagreLayout {
    fn convert_to_antv_format(&self, igr: &IntermediateGraph) -> Result<AntVGraph, LayoutError> {
        let mut antv_graph = AntVGraph::new();
        
        // Add nodes with initial dimensions
        for (node_idx, node_data) in igr.graph.node_references() {
            let (width, height) = self.calculate_node_dimensions(node_data);
            antv_graph.add_node(AntVNode {
                id: node_data.id.clone(),
                width,
                height,
                x: 0.0, // Will be calculated
                y: 0.0, // Will be calculated
            });
        }
        
        // Add edges
        for edge in igr.graph.edge_references() {
            let source = &igr.graph[edge.source()].id;
            let target = &igr.graph[edge.target()].id;
            antv_graph.add_edge(AntVEdge {
                source: source.clone(),
                target: target.clone(),
            });
        }
        
        Ok(antv_graph)
    }
    
    fn calculate_node_dimensions(&self, node_data: &NodeData) -> (f64, f64) {
        // Calculate based on text content and attributes
        let base_width = node_data.attributes.width.unwrap_or_else(|| {
            self.estimate_text_width(&node_data.label)
        });
        let base_height = node_data.attributes.height.unwrap_or(60.0);
        
        (base_width, base_height)
    }
    
    fn estimate_text_width(&self, text: &str) -> f64 {
        // Simple text width estimation
        // In production, this should use proper font metrics
        text.len() as f64 * 8.0 + 40.0 // padding
    }
}
```

### 4.2 LLM Layout Optimization

#### 4.2.1 LLM Optimizer Interface

```rust
// src/layout/llm_optimizer.rs
use serde::{Deserialize, Serialize};

pub struct LLMLayoutOptimizer {
    client: LLMClient,
    enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct LayoutOptimizationRequest {
    pub edsl_source: String,
    pub current_layout: Vec<NodePosition>,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodePosition {
    pub id: String,
    pub x: f64,
    pub y: f64, 
    pub width: f64,
    pub height: f64,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct LayoutAdjustment {
    pub id: String,
    pub x_move: Option<f64>,
    pub y_move: Option<f64>,
    pub reason: Option<String>,
}

impl LLMLayoutOptimizer {
    pub fn new(api_key: String) -> Self {
        Self {
            client: LLMClient::new(api_key),
            enabled: true,
        }
    }
    
    pub fn optimize_layout(
        &self, 
        igr: &mut IntermediateGraph,
        original_edsl: &str
    ) -> Result<Vec<LayoutAdjustment>, OptimizerError> {
        if !self.enabled {
            return Ok(vec![]);
        }
        
        let request = self.prepare_request(igr, original_edsl);
        let prompt = self.build_optimization_prompt(request);
        
        let response = self.client.query(&prompt)?;
        let adjustments = self.parse_adjustments(&response)?;
        
        self.apply_adjustments(igr, &adjustments)?;
        
        Ok(adjustments)
    }
    
    fn build_optimization_prompt(&self, request: LayoutOptimizationRequest) -> String {
        format!(r#"
You are a professional diagram layout optimizer. Your task is to analyze the provided EDSL source code and current layout positions, then suggest improvements to enhance semantic clarity and visual appeal.

## Original EDSL Source:
```edsl
{}
```

## Current Layout (nodes with positions):
```json
{}
```

## Your Task:
Based on the EDSL semantics, evaluate if the current layout can be improved. Consider:
1. Semantic positioning (e.g., databases typically at bottom, users at top)
2. Visual balance and symmetry
3. Logical flow direction
4. Container organization

**IMPORTANT**: Respond ONLY with a JSON array of adjustment objects. Each object must have:
- `id`: node identifier
- `x_move`: horizontal movement (optional)
- `y_move`: vertical movement (optional)
- `reason`: brief explanation (optional)

Example: [{"id": "database", "y_move": 50, "reason": "move database to bottom layer"}]

If no improvements needed, return empty array: []
        "#, 
        request.edsl_source,
        serde_json::to_string_pretty(&request.current_layout).unwrap()
        )
    }
    
    fn parse_adjustments(&self, response: &str) -> Result<Vec<LayoutAdjustment>, OptimizerError> {
        // Extract JSON from response (handle potential markdown formatting)
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];
        
        serde_json::from_str(json_str)
            .map_err(|e| OptimizerError::InvalidResponse(e.to_string()))
    }
    
    fn apply_adjustments(
        &self, 
        igr: &mut IntermediateGraph, 
        adjustments: &[LayoutAdjustment]
    ) -> Result<(), OptimizerError> {
        for adjustment in adjustments {
            if let Some(node_idx) = igr.node_map.get(&adjustment.id) {
                let node = &mut igr.graph[*node_idx];
                
                if let Some(x_move) = adjustment.x_move {
                    node.x += x_move;
                }
                if let Some(y_move) = adjustment.y_move {
                    node.y += y_move;
                }
            }
        }
        Ok(())
    }
}
```

#### 4.2.2 LLM Client Implementation

```rust
// src/layout/llm_client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct LLMClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }
    
    pub async fn query(&self, prompt: &str) -> Result<String, LLMError> {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a diagram layout optimization expert.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.1,
            max_tokens: 1000,
        };
        
        let response = self.client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
            
        let chat_response: ChatResponse = response.json().await?;
        
        Ok(chat_response.choices[0].message.content.clone())
    }
}
```

## 5. Code Generation

### 5.1 ExcalidrawElementSkeleton Generator

```rust
// src/generator/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcalidrawElementSkeleton {
    pub r#type: String,
    pub id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub stroke_color: String,
    pub background_color: String,
    pub fill_style: String,
    pub stroke_width: f64,
    pub stroke_style: String,
    pub roughness: u8,
    pub opacity: f64,
    pub text: Option<String>,
    pub font_size: f64,
    pub font_family: u8,
    pub start_binding: Option<ElementBinding>,
    pub end_binding: Option<ElementBinding>,
    pub start_arrowhead: Option<String>,
    pub end_arrowhead: Option<String>,
    pub points: Option<Vec<[f64; 2]>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementBinding {
    pub element_id: String,
    pub focus: f64,
    pub gap: f64,
}

pub struct ExcalidrawGenerator;

impl ExcalidrawGenerator {
    pub fn generate(igr: &IntermediateGraph) -> Result<Vec<ExcalidrawElementSkeleton>, GeneratorError> {
        let mut elements = Vec::new();
        
        // Generate container elements first (background)
        for container in &igr.containers {
            if let Some(container_element) = Self::generate_container(container)? {
                elements.push(container_element);
            }
        }
        
        // Generate node elements
        for (node_idx, node_data) in igr.graph.node_references() {
            let element = Self::generate_node(node_data, &format!("node_{}", node_idx.index()))?;
            elements.push(element);
        }
        
        // Generate edge elements
        for (edge_idx, edge_ref) in igr.graph.edge_references().enumerate() {
            let source_node = &igr.graph[edge_ref.source()];
            let target_node = &igr.graph[edge_ref.target()];
            let edge_data = edge_ref.weight();
            
            let element = Self::generate_edge(
                edge_data, 
                source_node, 
                target_node,
                &format!("edge_{}", edge_idx)
            )?;
            elements.push(element);
        }
        
        Ok(elements)
    }
    
    fn generate_node(node_data: &NodeData, element_id: &str) -> Result<ExcalidrawElementSkeleton, GeneratorError> {
        let shape_type = match node_data.attributes.shape.as_deref() {
            Some("rectangle") | None => "rectangle",
            Some("ellipse") => "ellipse", 
            Some("diamond") => "diamond",
            Some("cylinder") => "ellipse", // Approximate with ellipse
            Some("text") => "text",
            _ => "rectangle",
        };
        
        Ok(ExcalidrawElementSkeleton {
            r#type: shape_type.to_string(),
            id: Some(element_id.to_string()),
            x: node_data.x,
            y: node_data.y,
            width: node_data.width,
            height: node_data.height,
            angle: 0.0,
            stroke_color: node_data.attributes.stroke_color
                .clone()
                .unwrap_or_else(|| "#000000".to_string()),
            background_color: node_data.attributes.background_color
                .clone()
                .unwrap_or_else(|| "#ffffff".to_string()),
            fill_style: Self::convert_fill_style(&node_data.attributes.fill_style),
            stroke_width: node_data.attributes.stroke_width.unwrap_or(2.0),
            stroke_style: Self::convert_stroke_style(&node_data.attributes.stroke_style),
            roughness: node_data.attributes.roughness.unwrap_or(1),
            opacity: 100.0,
            text: if node_data.label.is_empty() { None } else { Some(node_data.label.clone()) },
            font_size: node_data.attributes.font_size.unwrap_or(20.0),
            font_family: 1, // Virgil
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
        })
    }
    
    fn generate_edge(
        edge_data: &EdgeData,
        source_node: &NodeData,
        target_node: &NodeData,
        element_id: &str
    ) -> Result<ExcalidrawElementSkeleton, GeneratorError> {
        let arrow_type = match edge_data.arrow_type {
            ArrowType::SingleArrow => "arrow",
            ArrowType::Line => "line",
            ArrowType::DoubleArrow => "arrow", // Handle in arrowheads
            ArrowType::WavyArrow => "arrow", // Custom styling
        };
        
        // Calculate connection points
        let start_point = Self::calculate_connection_point(source_node, target_node, true);
        let end_point = Self::calculate_connection_point(target_node, source_node, false);
        
        Ok(ExcalidrawElementSkeleton {
            r#type: arrow_type.to_string(),
            id: Some(element_id.to_string()),
            x: start_point.0,
            y: start_point.1,
            width: end_point.0 - start_point.0,
            height: end_point.1 - start_point.1,
            angle: 0.0,
            stroke_color: edge_data.attributes.stroke_color
                .clone()
                .unwrap_or_else(|| "#000000".to_string()),
            background_color: "transparent".to_string(),
            fill_style: "solid".to_string(),
            stroke_width: edge_data.attributes.stroke_width.unwrap_or(2.0),
            stroke_style: Self::convert_stroke_style(&edge_data.attributes.stroke_style),
            roughness: edge_data.attributes.roughness.unwrap_or(1),
            opacity: 100.0,
            text: edge_data.label.clone(),
            font_size: 16.0,
            font_family: 1,
            start_binding: Some(ElementBinding {
                element_id: format!("node_{}", source_node.id),
                focus: 0.0,
                gap: 0.0,
            }),
            end_binding: Some(ElementBinding {
                element_id: format!("node_{}", target_node.id), 
                focus: 0.0,
                gap: 0.0,
            }),
            start_arrowhead: Self::convert_arrowhead(&edge_data.attributes.start_arrowhead),
            end_arrowhead: Self::convert_arrowhead(&edge_data.attributes.end_arrowhead)
                .or_else(|| match edge_data.arrow_type {
                    ArrowType::SingleArrow => Some("triangle".to_string()),
                    ArrowType::DoubleArrow => Some("triangle".to_string()),
                    _ => None,
                }),
            points: Some(vec![
                [0.0, 0.0],
                [end_point.0 - start_point.0, end_point.1 - start_point.1]
            ]),
        })
    }
    
    fn convert_fill_style(fill_style: &Option<FillStyle>) -> String {
        match fill_style {
            Some(FillStyle::Solid) => "solid",
            Some(FillStyle::Hachure) => "hachure", 
            Some(FillStyle::CrossHatch) => "cross-hatch",
            Some(FillStyle::None) | None => "solid",
        }.to_string()
    }
    
    fn convert_stroke_style(stroke_style: &Option<StrokeStyle>) -> String {
        match stroke_style {
            Some(StrokeStyle::Dotted) => "dotted",
            Some(StrokeStyle::Dashed) => "dashed",
            Some(StrokeStyle::Solid) | None => "solid",
        }.to_string()
    }
    
    fn calculate_connection_point(
        from_node: &NodeData, 
        to_node: &NodeData, 
        is_start: bool
    ) -> (f64, f64) {
        // Simple edge connection calculation
        // In production, this should calculate precise edge intersections
        let center_x = from_node.x + from_node.width / 2.0;
        let center_y = from_node.y + from_node.height / 2.0;
        
        // Calculate direction towards target
        let target_center_x = to_node.x + to_node.width / 2.0;
        let target_center_y = to_node.y + to_node.height / 2.0;
        
        let dx = target_center_x - center_x;
        let dy = target_center_y - center_y;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length == 0.0 {
            return (center_x, center_y);
        }
        
        // Find intersection with node boundary
        let norm_dx = dx / length;
        let norm_dy = dy / length;
        
        let half_width = from_node.width / 2.0;
        let half_height = from_node.height / 2.0;
        
        // Simple rectangular boundary intersection
        let edge_x = if norm_dx.abs() * half_height > norm_dy.abs() * half_width {
            center_x + half_width * norm_dx.signum()
        } else {
            center_x + half_width * norm_dx * half_height / (norm_dy.abs() * half_width).max(0.001)
        };
        
        let edge_y = if norm_dy.abs() * half_width > norm_dx.abs() * half_height {
            center_y + half_height * norm_dy.signum()
        } else {
            center_y + half_height * norm_dy * half_width / (norm_dx.abs() * half_height).max(0.001)
        };
        
        (edge_x, edge_y)
    }
}
```

## 6. Error Handling & Validation

### 6.1 Error Types

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EDSLError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Build error: {0}")]
    Build(#[from] BuildError),
    
    #[error("Layout error: {0}")]
    Layout(#[from] LayoutError),
    
    #[error("Generator error: {0}")]
    Generator(#[from] GeneratorError),
    
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },
    
    #[error("Invalid YAML configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Pest parsing failed: {0}")]
    PestError(#[from] pest::error::Error<crate::parser::Rule>),
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Unknown node referenced: {0}")]
    UnknownNode(String),
    
    #[error("Circular container dependency")]
    CircularDependency,
    
    #[error("Invalid attribute value for {attribute}: {value}")]
    InvalidAttribute { attribute: String, value: String },
}

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Unknown layout engine: {0}")]
    UnknownEngine(String),
    
    #[error("Layout calculation failed: {0}")]
    CalculationFailed(String),
    
    #[error("AntV layout error: {0}")]
    AntVError(String),
}

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    
    #[error("API quota exceeded")]
    QuotaExceeded,
    
    #[error("LLM service unavailable")]
    ServiceUnavailable,
}
```

### 6.2 Validation Framework

```rust
// src/validation/mod.rs
pub struct ValidationEngine;

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl ValidationEngine {
    pub fn validate_igr(igr: &IntermediateGraph) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Validate graph connectivity
        Self::validate_connectivity(igr, &mut result);
        
        // Validate attributes
        Self::validate_attributes(igr, &mut result);
        
        // Validate containers
        Self::validate_containers(igr, &mut result);
        
        result.is_valid = result.errors.is_empty();
        result
    }
    
    fn validate_connectivity(igr: &IntermediateGraph, result: &mut ValidationResult) {
        // Check for isolated nodes (warning)
        for (node_idx, node_data) in igr.graph.node_references() {
            let has_edges = igr.graph.edges(node_idx).next().is_some() ||
                           igr.graph.edges_directed(node_idx, petgraph::Incoming).next().is_some();
            
            if !has_edges {
                result.warnings.push(ValidationWarning {
                    code: "W001".to_string(),
                    message: format!("Node '{}' has no connections", node_data.id),
                    location: None,
                });
            }
        }
    }
    
    fn validate_attributes(igr: &IntermediateGraph, result: &mut ValidationResult) {
        for (_, node_data) in igr.graph.node_references() {
            // Validate color values
            if let Some(color) = &node_data.attributes.stroke_color {
                if !Self::is_valid_color(color) {
                    result.errors.push(ValidationError {
                        code: "E001".to_string(),
                        message: format!("Invalid color value: {}", color),
                        location: None,
                    });
                }
            }
            
            // Validate numeric ranges
            if let Some(roughness) = node_data.attributes.roughness {
                if roughness > 2 {
                    result.errors.push(ValidationError {
                        code: "E002".to_string(),
                        message: format!("Roughness value {} exceeds maximum of 2", roughness),
                        location: None,
                    });
                }
            }
        }
    }
    
    fn is_valid_color(color: &str) -> bool {
        // Simple hex color validation
        color.starts_with('#') && color.len() == 7 && 
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}
```

## 7. CLI Interface & Integration

### 7.1 Main CLI Application

```rust
// src/main.rs
use clap::{Arg, Command};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("edsl")
        .version("0.1.0")
        .about("ExcaliDraw-DSL Compiler")
        .arg(Arg::new("input")
            .help("Input EDSL file")
            .required(true)
            .index(1))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .help("Output file path")
            .takes_value(true))
        .arg(Arg::new("layout")
            .short('l')
            .long("layout")
            .help("Layout algorithm")
            .takes_value(true)
            .possible_values(&["dagre", "force", "elk"]))
        .arg(Arg::new("llm-optimize")
            .long("llm-optimize")
            .help("Enable LLM layout optimization")
            .takes_value(false))
        .arg(Arg::new("api-key")
            .long("api-key")
            .help("LLM API key")
            .takes_value(true)
            .env("EDSL_API_KEY"))
        .arg(Arg::new("validate")
            .long("validate")
            .help("Validate input only")
            .takes_value(false))
        .get_matches();

    let input_path = PathBuf::from(matches.value_of("input").unwrap());
    let output_path = matches.value_of("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut path = input_path.clone();
            path.set_extension("json");
            path
        });

    // Read input file
    let input_content = std::fs::read_to_string(&input_path)?;
    
    // Parse EDSL
    let parsed_doc = crate::parser::parse_edsl(&input_content)?;
    
    // Build IGR
    let mut igr = crate::igr::IntermediateGraph::from_ast(parsed_doc)?;
    
    // Override layout if specified
    if let Some(layout) = matches.value_of("layout") {
        igr.global_config.layout = Some(layout.to_string());
    }
    
    // Validate if requested
    if matches.is_present("validate") {
        let validation_result = crate::validation::ValidationEngine::validate_igr(&igr);
        
        for error in validation_result.errors {
            eprintln!("Error {}: {}", error.code, error.message);
        }
        
        for warning in validation_result.warnings {
            eprintln!("Warning {}: {}", warning.code, warning.message);
        }
        
        if !validation_result.is_valid {
            std::process::exit(1);
        }
        
        println!("Validation passed!");
        return Ok(());
    }
    
    // Apply layout
    let layout_manager = crate::layout::LayoutManager::new();
    layout_manager.layout(&mut igr)?;
    
    // Apply LLM optimization if enabled
    if matches.is_present("llm-optimize") {
        if let Some(api_key) = matches.value_of("api-key") {
            let optimizer = crate::layout::LLMLayoutOptimizer::new(api_key.to_string());
            let adjustments = optimizer.optimize_layout(&mut igr, &input_content)?;
            
            if !adjustments.is_empty() {
                println!("Applied {} LLM optimizations", adjustments.len());
            }
        } else {
            eprintln!("Warning: LLM optimization requested but no API key provided");
        }
    }
    
    // Generate Excalidraw elements
    let elements = crate::generator::ExcalidrawGenerator::generate(&igr)?;
    
    // Write output
    let output_json = serde_json::to_string_pretty(&elements)?;
    std::fs::write(&output_path, output_json)?;
    
    println!("Generated Excalidraw JSON: {}", output_path.display());
    
    Ok(())
}
```

### 7.2 Library API

```rust
// src/lib.rs
pub mod parser;
pub mod igr;
pub mod layout;
pub mod generator;
pub mod validation;
pub mod error;

pub use error::EDSLError;

pub struct EDSLCompiler {
    layout_manager: layout::LayoutManager,
    llm_optimizer: Option<layout::LLMLayoutOptimizer>,
}

impl EDSLCompiler {
    pub fn new() -> Self {
        Self {
            layout_manager: layout::LayoutManager::new(),
            llm_optimizer: None,
        }
    }
    
    pub fn with_llm_optimization(mut self, api_key: String) -> Self {
        self.llm_optimizer = Some(layout::LLMLayoutOptimizer::new(api_key));
        self
    }
    
    pub fn compile(&self, edsl_source: &str) -> Result<String, EDSLError> {
        // Parse
        let parsed_doc = parser::parse_edsl(edsl_source)?;
        
        // Build IGR
        let mut igr = igr::IntermediateGraph::from_ast(parsed_doc)?;
        
        // Layout
        self.layout_manager.layout(&mut igr)?;
        
        // LLM optimization
        if let Some(optimizer) = &self.llm_optimizer {
            optimizer.optimize_layout(&mut igr, edsl_source)?;
        }
        
        // Generate
        let elements = generator::ExcalidrawGenerator::generate(&igr)?;
        
        // Serialize
        Ok(serde_json::to_string_pretty(&elements)?)
    }
    
    pub fn validate(&self, edsl_source: &str) -> Result<validation::ValidationResult, EDSLError> {
        let parsed_doc = parser::parse_edsl(edsl_source)?;
        let igr = igr::IntermediateGraph::from_ast(parsed_doc)?;
        Ok(validation::ValidationEngine::validate_igr(&igr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_compilation() {
        let edsl = r#"
---
layout: dagre
---

user[User] -> api[API Gateway] -> db[Database] {
  shape: cylinder;
}
        "#;
        
        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl);
        
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("rectangle"));
        assert!(json.contains("cylinder"));
    }
    
    #[test]
    fn test_container_compilation() {
        let edsl = r#"
container "Backend" {
  api[API]
  db[Database]
  api -> db;
}
        "#;
        
        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl);
        
        assert!(result.is_ok());
    }
}
```

## 8. Implementation Roadmap

### Phase 1: Core MVP (4-6 weeks)
1. **Week 1-2**: Parser implementation with pest
2. **Week 2-3**: IGR and basic Dagre layout
3. **Week 3-4**: ExcalidrawElementSkeleton generator
4. **Week 4**: CLI interface and basic testing
5. **Week 5-6**: Integration testing and bug fixes

### Phase 2: Enhanced Features (3-4 weeks)
1. **Week 1**: Multiple layout algorithms (force, elk)
2. **Week 2**: Advanced EDSL syntax features
3. **Week 3**: Comprehensive validation system
4. **Week 4**: Error handling and user experience improvements

### Phase 3: LLM Integration (2-3 weeks)
1. **Week 1**: LLM client and optimization framework
2. **Week 2**: Prompt engineering and testing
3. **Week 3**: Integration and performance optimization

### Phase 4: Ecosystem (4-5 weeks)
1. **Week 1-2**: WASM compilation for web use
2. **Week 2-3**: Web playground development
3. **Week 4**: VS Code extension
4. **Week 5**: Documentation and examples

## 9. Testing Strategy

### 9.1 Unit Tests
- Parser: Grammar rule validation
- IGR: Graph construction and manipulation
- Layout: Algorithm correctness
- Generator: Excalidraw format compliance

### 9.2 Integration Tests
- End-to-end compilation pipeline
- LLM optimization accuracy
- Error handling coverage
- Performance benchmarks

### 9.3 Visual Regression Tests
- Excalidraw rendering consistency
- Layout algorithm stability
- LLM optimization quality

## 10. Performance Considerations

### 10.1 Compilation Performance
- Target: <100ms for small diagrams (<20 nodes)
- Target: <1s for medium diagrams (<100 nodes)
- Target: <10s for large diagrams (<1000 nodes)

### 10.2 Memory Usage
- Efficient IGR representation
- Streaming parser for large files
- Lazy evaluation where possible

### 10.3 LLM Optimization
- Caching of optimization results
- Timeout handling (5s max)
- Fallback to deterministic layout

## 11. Security Considerations

### 11.1 Input Validation
- Strict grammar enforcement
- Resource limits (max nodes, containers)
- Sanitization of user content

### 11.2 LLM Integration
- API key security
- Response validation and sanitization
- Rate limiting and quota management

### 11.3 Output Safety
- Safe JSON generation
- XSS prevention in generated content
- File system access controls

## Conclusion

This design provides a comprehensive foundation for implementing ExcaliDraw-DSL. The modular architecture ensures maintainability and extensibility, while the hybrid layout approach with optional LLM optimization offers both reliability and innovation. The phased implementation roadmap allows for iterative development and early user feedback.

The system balances simplicity for basic use cases with powerful features for advanced users, achieving the goal of creating a DSL that is both approachable and expressive for generating high-quality Excalidraw diagrams.