// src/ast.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub font: Option<String>,
    pub sketchiness: Option<u8>,
    pub stroke_width: Option<f64>,
    pub background_color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub config: GlobalConfig,
    pub component_types: HashMap<String, ComponentTypeDefinition>,
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<EdgeDefinition>,
    pub containers: Vec<ContainerDefinition>,
    pub groups: Vec<GroupDefinition>,
    pub connections: Vec<ConnectionDefinition>,
}

#[derive(Debug, Clone)]
pub struct ComponentTypeDefinition {
    pub name: String,
    pub shape: Option<String>,
    pub style: StyleDefinition,
}

#[derive(Debug, Clone)]
pub struct StyleDefinition {
    pub fill: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub stroke_style: Option<StrokeStyle>,
    pub rounded: Option<f64>,
    pub fill_style: Option<FillStyle>,
    pub roughness: Option<u8>,
    pub font_size: Option<f64>,
    pub font: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NodeDefinition {
    pub id: String,
    pub label: Option<String>,
    pub component_type: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone)]
pub struct EdgeDefinition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub arrow_type: ArrowType,
    pub attributes: HashMap<String, AttributeValue>,
    pub style: Option<EdgeStyleDefinition>,
}

#[derive(Debug, Clone)]
pub struct ConnectionDefinition {
    pub from: String,
    pub to: Vec<String>,
    pub style: EdgeStyleDefinition,
}

#[derive(Debug, Clone)]
pub struct EdgeStyleDefinition {
    pub edge_type: Option<EdgeType>,
    pub label: Option<String>,
    pub label_position: Option<f64>,
    pub routing: Option<RoutingType>,
    pub color: Option<String>,
    pub width: Option<f64>,
    pub stroke_style: Option<StrokeStyle>,
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
pub struct GroupDefinition {
    pub id: String,
    pub label: Option<String>,
    pub group_type: GroupType,
    pub children: Vec<String>, // Node IDs
    pub attributes: HashMap<String, AttributeValue>,
    pub internal_statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupType {
    BasicGroup,            // group "name" { ... }
    FlowGroup,             // flow "name" { ... }
    SemanticGroup(String), // service "name" { ... }, layer "name" { ... }, etc.
}

#[derive(Debug, Clone)]
pub enum Statement {
    Node(NodeDefinition),
    Edge(EdgeDefinition),
    Container(ContainerDefinition),
    Group(GroupDefinition),
    Connection(ConnectionDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Color(String),
    Boolean(bool),
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AttributeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowType {
    SingleArrow, // ->
    Line,        // --
    DoubleArrow, // <->
    WavyArrow,   // ~>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Arrow,
    Line,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingType {
    Straight,
    Orthogonal,
    Curved,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeStyle {
    Solid,
    Dashed,
    Dotted,
}

impl FromStr for ArrowType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "->" => Ok(ArrowType::SingleArrow),
            "--" => Ok(ArrowType::Line),
            "<->" => Ok(ArrowType::DoubleArrow),
            "~>" => Ok(ArrowType::WavyArrow),
            _ => Err(()),
        }
    }
}

impl ArrowType {
    pub fn to_excalidraw_type(&self) -> &'static str {
        match self {
            ArrowType::SingleArrow => "arrow",
            ArrowType::Line => "line",
            ArrowType::DoubleArrow => "arrow",
            ArrowType::WavyArrow => "arrow",
        }
    }
}

impl FromStr for StrokeStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "solid" => Ok(StrokeStyle::Solid),
            "dotted" => Ok(StrokeStyle::Dotted),
            "dashed" => Ok(StrokeStyle::Dashed),
            _ => Err(()),
        }
    }
}

impl StrokeStyle {
    pub fn to_excalidraw_style(&self) -> &'static str {
        match self {
            StrokeStyle::Solid => "solid",
            StrokeStyle::Dotted => "dotted",
            StrokeStyle::Dashed => "dashed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillStyle {
    None,
    Solid,
    Hachure,
    CrossHatch,
}

impl FromStr for FillStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(FillStyle::None),
            "solid" => Ok(FillStyle::Solid),
            "hachure" => Ok(FillStyle::Hachure),
            "cross-hatch" => Ok(FillStyle::CrossHatch),
            _ => Err(()),
        }
    }
}

impl FillStyle {
    pub fn to_excalidraw_style(&self) -> &'static str {
        match self {
            FillStyle::None => "none",
            FillStyle::Solid => "solid",
            FillStyle::Hachure => "hachure",
            FillStyle::CrossHatch => "cross-hatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrowheadType {
    None,
    Triangle,
    Dot,
    Diamond,
}

impl FromStr for ArrowheadType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(ArrowheadType::None),
            "triangle" => Ok(ArrowheadType::Triangle),
            "dot" => Ok(ArrowheadType::Dot),
            "diamond" => Ok(ArrowheadType::Diamond),
            _ => Err(()),
        }
    }
}

impl ArrowheadType {
    pub fn to_excalidraw_type(&self) -> Option<&'static str> {
        match self {
            ArrowheadType::None => None,
            ArrowheadType::Triangle => Some("triangle"),
            ArrowheadType::Dot => Some("dot"),
            ArrowheadType::Diamond => Some("diamond"),
        }
    }
}
