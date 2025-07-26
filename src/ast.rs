// src/ast.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<EdgeDefinition>,
    pub containers: Vec<ContainerDefinition>,
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
pub enum Statement {
    Node(NodeDefinition),
    Edge(EdgeDefinition),
    Container(ContainerDefinition),
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

impl ArrowType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "->" => Some(ArrowType::SingleArrow),
            "--" => Some(ArrowType::Line),
            "<->" => Some(ArrowType::DoubleArrow),
            "~>" => Some(ArrowType::WavyArrow),
            _ => None,
        }
    }

    pub fn to_excalidraw_type(&self) -> &'static str {
        match self {
            ArrowType::SingleArrow => "arrow",
            ArrowType::Line => "line",
            ArrowType::DoubleArrow => "arrow",
            ArrowType::WavyArrow => "arrow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrokeStyle {
    Solid,
    Dotted,
    Dashed,
}

impl StrokeStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "solid" => Some(StrokeStyle::Solid),
            "dotted" => Some(StrokeStyle::Dotted),
            "dashed" => Some(StrokeStyle::Dashed),
            _ => None,
        }
    }

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

impl FillStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(FillStyle::None),
            "solid" => Some(FillStyle::Solid),
            "hachure" => Some(FillStyle::Hachure),
            "cross-hatch" => Some(FillStyle::CrossHatch),
            _ => None,
        }
    }

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

impl ArrowheadType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(ArrowheadType::None),
            "triangle" => Some(ArrowheadType::Triangle),
            "dot" => Some(ArrowheadType::Dot),
            "diamond" => Some(ArrowheadType::Diamond),
            _ => None,
        }
    }

    pub fn to_excalidraw_type(&self) -> Option<&'static str> {
        match self {
            ArrowheadType::None => None,
            ArrowheadType::Triangle => Some("triangle"),
            ArrowheadType::Dot => Some("dot"),
            ArrowheadType::Diamond => Some("diamond"),
        }
    }
}
