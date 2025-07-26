// src/ast.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Maximum allowed sketchiness value
pub const MAX_SKETCHINESS: u8 = 4;

/// Valid range for stroke width
pub const MIN_STROKE_WIDTH: f64 = 0.1;
pub const MAX_STROKE_WIDTH: f64 = 20.0;

/// Supported theme values
pub const VALID_THEMES: &[&str] = &["light", "dark"];

/// Supported layout algorithms
pub const VALID_LAYOUTS: &[&str] = &["dagre", "force", "manual", "elk"];

/// Supported font families
pub const VALID_FONTS: &[&str] = &["Virgil", "Helvetica", "Cascadia"];

/// Global configuration settings for the EDSL document
///
/// Controls overall rendering and layout behavior. All fields are optional
/// and will use sensible defaults if not specified.
///
/// # Examples
///
/// ```rust
/// use excalidraw_dsl::ast::GlobalConfig;
///
/// let config = GlobalConfig::builder()
///     .theme("dark")
///     .layout("dagre")
///     .sketchiness(2).unwrap()
///     .build();
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Theme for the document ("light" or "dark")
    pub theme: Option<String>,
    /// Layout algorithm to use ("dagre", "force", "manual")
    pub layout: Option<String>,
    /// Default font family ("Virgil", "Helvetica", "Cascadia")
    pub font: Option<String>,
    /// Hand-drawn style intensity (0-4, where 0 is precise and 4 is very sketchy)
    pub sketchiness: Option<u8>,
    /// Default stroke width in pixels (0.1-20.0)
    pub stroke_width: Option<f64>,
    /// Background color for the document
    pub background_color: Option<String>,
}

impl GlobalConfig {
    /// Create a new builder for GlobalConfig
    pub fn builder() -> GlobalConfigBuilder {
        GlobalConfigBuilder::new()
    }

    /// Validate the configuration values
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate theme
        if let Some(ref theme) = self.theme {
            if !VALID_THEMES.contains(&theme.as_str()) {
                return Err(crate::error::EDSLError::Validation {
                    message: format!(
                        "Invalid theme '{}', must be one of: {}",
                        theme,
                        VALID_THEMES.join(", ")
                    ),
                });
            }
        }

        // Validate layout
        if let Some(ref layout) = self.layout {
            if !VALID_LAYOUTS.contains(&layout.as_str()) {
                return Err(crate::error::EDSLError::Validation {
                    message: format!(
                        "Invalid layout '{}', must be one of: {}",
                        layout,
                        VALID_LAYOUTS.join(", ")
                    ),
                });
            }
        }

        // Validate font
        if let Some(ref font) = self.font {
            if !VALID_FONTS.contains(&font.as_str()) {
                return Err(crate::error::EDSLError::Validation {
                    message: format!(
                        "Invalid font '{}', must be one of: {}",
                        font,
                        VALID_FONTS.join(", ")
                    ),
                });
            }
        }

        // Validate sketchiness range
        if let Some(sketchiness) = self.sketchiness {
            if sketchiness > MAX_SKETCHINESS {
                return Err(crate::error::EDSLError::Validation {
                    message: format!(
                        "Sketchiness must be between 0-{MAX_SKETCHINESS}, got {sketchiness}"
                    ),
                });
            }
        }

        // Validate stroke width
        if let Some(width) = self.stroke_width {
            if !(MIN_STROKE_WIDTH..=MAX_STROKE_WIDTH).contains(&width) {
                return Err(crate::error::EDSLError::Validation {
                    message: format!(
                        "Stroke width must be between {MIN_STROKE_WIDTH}-{MAX_STROKE_WIDTH}, got {width}"
                    ),
                });
            }
        }

        Ok(())
    }
}

/// Builder for creating GlobalConfig instances
#[derive(Debug, Default)]
pub struct GlobalConfigBuilder {
    theme: Option<String>,
    layout: Option<String>,
    font: Option<String>,
    sketchiness: Option<u8>,
    stroke_width: Option<f64>,
    background_color: Option<String>,
}

impl GlobalConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme<S: Into<String>>(mut self, theme: S) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn layout<S: Into<String>>(mut self, layout: S) -> Self {
        self.layout = Some(layout.into());
        self
    }

    pub fn font<S: Into<String>>(mut self, font: S) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn sketchiness(mut self, sketchiness: u8) -> crate::error::Result<Self> {
        if sketchiness > MAX_SKETCHINESS {
            return Err(crate::error::EDSLError::Validation {
                message: format!(
                    "Sketchiness must be between 0-{MAX_SKETCHINESS}, got {sketchiness}"
                ),
            });
        }
        self.sketchiness = Some(sketchiness);
        Ok(self)
    }

    pub fn stroke_width(mut self, width: f64) -> crate::error::Result<Self> {
        if !(MIN_STROKE_WIDTH..=MAX_STROKE_WIDTH).contains(&width) {
            return Err(crate::error::EDSLError::Validation {
                message: format!(
                    "Stroke width must be between {MIN_STROKE_WIDTH}-{MAX_STROKE_WIDTH}, got {width}"
                ),
            });
        }
        self.stroke_width = Some(width);
        Ok(self)
    }

    pub fn background_color<S: Into<String>>(mut self, color: S) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn build(self) -> GlobalConfig {
        GlobalConfig {
            theme: self.theme,
            layout: self.layout,
            font: self.font,
            sketchiness: self.sketchiness,
            stroke_width: self.stroke_width,
            background_color: self.background_color,
        }
    }
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
pub struct EdgeChainDefinition {
    pub nodes: Vec<String>,
    pub label: Option<String>,
    pub arrow_type: ArrowType,
    pub attributes: HashMap<String, AttributeValue>,
    pub style: Option<EdgeStyleDefinition>,
}

impl EdgeChainDefinition {
    /// Expand edge chain into individual edge definitions
    pub fn expand(&self) -> Vec<EdgeDefinition> {
        let mut edges = Vec::new();

        for i in 0..self.nodes.len().saturating_sub(1) {
            let edge_label = if i == 0 {
                self.label.clone() // Only apply label to first edge
            } else {
                None
            };

            edges.push(EdgeDefinition {
                from: self.nodes[i].clone(),
                to: self.nodes[i + 1].clone(),
                label: edge_label,
                arrow_type: self.arrow_type,
                attributes: self.attributes.clone(),
                style: self.style.clone(),
            });
        }

        edges
    }
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

impl TryFrom<AttributeValue> for String {
    type Error = crate::error::EDSLError;

    fn try_from(value: AttributeValue) -> crate::error::Result<Self> {
        match value {
            AttributeValue::String(s) => Ok(s),
            _ => Err(crate::error::EDSLError::Validation {
                message: format!("Expected string value, got {value:?}"),
            }),
        }
    }
}

impl TryFrom<AttributeValue> for f64 {
    type Error = crate::error::EDSLError;

    fn try_from(value: AttributeValue) -> crate::error::Result<Self> {
        match value {
            AttributeValue::Number(n) => Ok(n),
            _ => Err(crate::error::EDSLError::Validation {
                message: format!("Expected number value, got {value:?}"),
            }),
        }
    }
}

impl TryFrom<AttributeValue> for bool {
    type Error = crate::error::EDSLError;

    fn try_from(value: AttributeValue) -> crate::error::Result<Self> {
        match value {
            AttributeValue::Boolean(b) => Ok(b),
            _ => Err(crate::error::EDSLError::Validation {
                message: format!("Expected boolean value, got {value:?}"),
            }),
        }
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{s}"),
            AttributeValue::Number(n) => write!(f, "{n}"),
            AttributeValue::Color(c) => write!(f, "{c}"),
            AttributeValue::Boolean(b) => write!(f, "{b}"),
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
