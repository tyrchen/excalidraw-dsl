// src/igr.rs
use crate::ast::*;
use crate::error::{BuildError, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct IntermediateGraph {
    pub graph: DiGraph<NodeData, EdgeData>,
    pub global_config: GlobalConfig,
    pub component_types: HashMap<String, ComponentTypeDefinition>,
    pub containers: Vec<ContainerData>,
    pub groups: Vec<GroupData>,
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
pub struct GroupData {
    pub id: String,
    pub label: Option<String>,
    pub group_type: GroupType,
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
    pub rounded: Option<f64>,

    // Arrow properties
    pub start_arrowhead: Option<ArrowheadType>,
    pub end_arrowhead: Option<ArrowheadType>,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for ExcalidrawAttributes {
    fn default() -> Self {
        Self {
            shape: None,
            width: None,
            height: None,
            stroke_color: None,
            stroke_width: None,
            stroke_style: None,
            background_color: None,
            fill_style: None,
            fill_weight: None,
            roughness: None,
            font: None,
            font_size: None,
            rounded: None,
            start_arrowhead: None,
            end_arrowhead: None,
        }
    }
}

impl IntermediateGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            global_config: GlobalConfig::default(),
            component_types: HashMap::new(),
            containers: Vec::new(),
            groups: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn from_ast(document: ParsedDocument) -> Result<Self> {
        let mut igr = IntermediateGraph::new();
        igr.global_config = document.config;
        igr.component_types = document.component_types;

        // First pass: Build all nodes from top-level, containers, and groups
        let mut all_nodes = document.nodes;
        let mut all_edges = document.edges;

        // Extract nodes and edges from containers
        for container in &document.containers {
            for statement in &container.internal_statements {
                match statement {
                    Statement::Node(node) => all_nodes.push(node.clone()),
                    Statement::Edge(edge) => all_edges.push(edge.clone()),
                    Statement::Container(_) => {
                        // Nested containers - for now just ignore
                        // TODO: Handle nested containers
                    }
                    Statement::Group(_) => {
                        // Nested groups - for now just ignore
                        // TODO: Handle nested groups
                    }
                    Statement::Connection(_) => {
                        // Connections are handled separately
                    }
                }
            }
        }

        // Extract nodes and edges from groups
        for group in &document.groups {
            for statement in &group.internal_statements {
                match statement {
                    Statement::Node(node) => all_nodes.push(node.clone()),
                    Statement::Edge(edge) => all_edges.push(edge.clone()),
                    Statement::Container(_) => {
                        // Nested containers - for now just ignore
                    }
                    Statement::Group(_) => {
                        // Nested groups - for now just ignore
                    }
                    Statement::Connection(_) => {
                        // Connections are handled separately
                    }
                }
            }
        }

        // Build nodes
        for node_def in all_nodes {
            if igr.node_map.contains_key(&node_def.id) {
                return Err(BuildError::DuplicateNode(node_def.id).into());
            }

            let node_data = NodeData::from_definition(node_def, &igr.component_types)?;
            let node_idx = igr.graph.add_node(node_data.clone());
            igr.node_map.insert(node_data.id.clone(), node_idx);
        }

        // Build edges
        for edge_def in all_edges {
            let from_idx = igr
                .node_map
                .get(&edge_def.from)
                .ok_or_else(|| BuildError::UnknownNode(edge_def.from.clone()))?;
            let to_idx = igr
                .node_map
                .get(&edge_def.to)
                .ok_or_else(|| BuildError::UnknownNode(edge_def.to.clone()))?;

            let edge_data = EdgeData::from_definition(edge_def)?;
            igr.graph.add_edge(*from_idx, *to_idx, edge_data);
        }

        // Build containers
        for container_def in document.containers {
            let container_data = ContainerData::from_definition(container_def, &igr.node_map)?;
            igr.containers.push(container_data);
        }

        // Process connections (convert to edges)
        for connection in document.connections {
            // Convert each connection to edges
            let edges = EdgeData::from_connection(connection, &igr.node_map)?;
            for (from_idx, to_idx, edge_data) in edges {
                igr.graph.add_edge(from_idx, to_idx, edge_data);
            }
        }

        // Build groups
        for group_def in document.groups {
            let group_data = GroupData::from_definition(group_def, &igr.node_map)?;
            igr.groups.push(group_data);
        }

        Ok(igr)
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<(NodeIndex, &NodeData)> {
        self.node_map.get(id).map(|&idx| (idx, &self.graph[idx]))
    }

    pub fn get_node_mut_by_id(&mut self, id: &str) -> Option<(NodeIndex, &mut NodeData)> {
        self.node_map
            .get(id)
            .copied()
            .map(move |idx| (idx, &mut self.graph[idx]))
    }
}

impl NodeData {
    pub fn from_definition(def: NodeDefinition, component_types: &HashMap<String, ComponentTypeDefinition>) -> Result<Self> {
        let mut attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;
        
        // Apply component type styling if specified
        if let Some(type_name) = &def.component_type {
            if let Some(comp_type) = component_types.get(type_name) {
                // Apply component type styles
                if let Some(shape) = &comp_type.shape {
                    attributes.shape = Some(shape.clone());
                }
                
                // Apply style from component type (with node-specific overrides)
                if comp_type.style.fill.is_some() && attributes.background_color.is_none() {
                    attributes.background_color = comp_type.style.fill.clone();
                }
                if comp_type.style.stroke_color.is_some() && attributes.stroke_color.is_none() {
                    attributes.stroke_color = comp_type.style.stroke_color.clone();
                }
                if comp_type.style.stroke_width.is_some() && attributes.stroke_width.is_none() {
                    attributes.stroke_width = comp_type.style.stroke_width;
                }
                if comp_type.style.stroke_style.is_some() && attributes.stroke_style.is_none() {
                    attributes.stroke_style = comp_type.style.stroke_style.clone();
                }
                if comp_type.style.fill_style.is_some() && attributes.fill_style.is_none() {
                    attributes.fill_style = comp_type.style.fill_style.clone();
                }
                if comp_type.style.roughness.is_some() && attributes.roughness.is_none() {
                    attributes.roughness = comp_type.style.roughness;
                }
                if comp_type.style.font_size.is_some() && attributes.font_size.is_none() {
                    attributes.font_size = comp_type.style.font_size;
                }
                if comp_type.style.font.is_some() && attributes.font.is_none() {
                    attributes.font = comp_type.style.font.clone();
                }
                if comp_type.style.rounded.is_some() && attributes.rounded.is_none() {
                    attributes.rounded = comp_type.style.rounded;
                }
            } else {
                return Err(BuildError::UnknownComponentType(type_name.clone()).into());
            }
        }
        
        let label = def.label.unwrap_or_else(|| def.id.clone());

        // Estimate initial dimensions based on label with better text metrics
        let font_size = attributes.font_size.unwrap_or(20.0);
        let font_family = match attributes.font.as_deref() {
            Some("Virgil") => 1,
            Some("Helvetica") => 2,
            Some("Cascadia") | Some("Code") => 3,
            None => 3, // Default to Cascadia
            _ => 3,
        };
        
        // Calculate text dimensions using improved logic for better accuracy
        let char_width_multiplier = match font_family {
            1 => 0.65,  // Virgil - slightly wider
            2 => 0.55,  // Helvetica - slightly wider  
            3 => 0.6,   // Cascadia - wider for better readability
            _ => 0.6,
        };
        
        // Improved character width calculation with better handling for common characters
        let effective_length = label.chars().map(|c| {
            match c {
                // Narrow characters
                'i' | 'l' | '.' | '!' | '|' | '\'' | '`' | 'I' | 'j' | 'f' | 't' => 0.4,
                // Wide characters
                'w' | 'm' | 'W' | 'M' | '@' | '%' | '#' => 1.4,
                // Uppercase letters (generally wider)
                'A'..='Z' => 1.15,
                // Space (reduced to save space)
                ' ' => 0.35,
                // Numbers and common punctuation
                '0'..='9' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '=' | '+' => 0.9,
                // Default for most lowercase and other characters
                _ => 1.0,
            }
        }).sum::<f64>();
        
        let text_width = effective_length * font_size * char_width_multiplier;
        let text_height = font_size * 1.3; // Slightly more height for better appearance
        
        // Increased padding for better text visibility and node appearance
        let padding_x = 75.0; // Even more horizontal padding to prevent text overflow
        let padding_y = 25.0; // More vertical padding
        
        let estimated_width = attributes
            .width
            .unwrap_or_else(|| (text_width + padding_x).max(100.0)); // Increased minimum width
        let estimated_height = attributes
            .height
            .unwrap_or_else(|| (text_height + padding_y).max(70.0));  // Increased minimum height

        Ok(NodeData {
            id: def.id,
            label,
            attributes,
            x: 0.0,
            y: 0.0,
            width: estimated_width,
            height: estimated_height,
        })
    }
}

impl EdgeData {
    pub fn from_definition(def: EdgeDefinition) -> Result<Self> {
        let mut attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;
        
        // Apply advanced edge styling if present
        if let Some(style) = &def.style {
            // Map edge type to stroke style
            if let Some(edge_type) = &style.edge_type {
                match edge_type {
                    EdgeType::Dashed => attributes.stroke_style = Some(StrokeStyle::Dashed),
                    EdgeType::Dotted => attributes.stroke_style = Some(StrokeStyle::Dotted),
                    _ => attributes.stroke_style = Some(StrokeStyle::Solid),
                }
            }
            
            // Apply other style properties
            if let Some(color) = &style.color {
                attributes.stroke_color = Some(color.clone());
            }
            if let Some(width) = &style.width {
                attributes.stroke_width = Some(*width);
            }
            if let Some(stroke_style) = &style.stroke_style {
                attributes.stroke_style = Some(stroke_style.clone());
            }
        }

        Ok(EdgeData {
            label: def.label.or(def.style.as_ref().and_then(|s| s.label.clone())),
            arrow_type: def.arrow_type,
            attributes,
        })
    }
    
    pub fn from_connection(def: ConnectionDefinition, node_map: &HashMap<String, NodeIndex>) -> Result<Vec<(NodeIndex, NodeIndex, EdgeData)>> {
        let mut edges = Vec::new();
        
        // Get the source node index
        let from_idx = node_map.get(&def.from)
            .copied()
            .ok_or_else(|| BuildError::UnknownNode(def.from.clone()))?;
        
        // Create an edge for each target
        for to_id in &def.to {
            let to_idx = node_map.get(to_id)
                .copied()
                .ok_or_else(|| BuildError::UnknownNode(to_id.clone()))?;
            
            let mut attributes = ExcalidrawAttributes::default();
            
            // Apply connection style
            let style = &def.style;
            
            // Map edge type to arrow type and stroke style
            let arrow_type = if let Some(edge_type) = &style.edge_type {
                match edge_type {
                    EdgeType::Arrow => ArrowType::SingleArrow,
                    EdgeType::Line => ArrowType::Line,
                    _ => ArrowType::SingleArrow,
                }
            } else {
                ArrowType::SingleArrow
            };
            
            // Apply stroke style
            if let Some(edge_type) = &style.edge_type {
                match edge_type {
                    EdgeType::Dashed => attributes.stroke_style = Some(StrokeStyle::Dashed),
                    EdgeType::Dotted => attributes.stroke_style = Some(StrokeStyle::Dotted),
                    _ => attributes.stroke_style = Some(StrokeStyle::Solid),
                }
            }
            
            // Apply other style properties
            if let Some(color) = &style.color {
                attributes.stroke_color = Some(color.clone());
            }
            if let Some(width) = &style.width {
                attributes.stroke_width = Some(*width);
            }
            if let Some(stroke_style) = &style.stroke_style {
                attributes.stroke_style = Some(stroke_style.clone());
            }
            
            let edge_data = EdgeData {
                label: style.label.clone(),
                arrow_type,
                attributes,
            };
            
            edges.push((from_idx, to_idx, edge_data));
        }
        
        Ok(edges)
    }
}

impl ContainerData {
    pub fn from_definition(
        def: ContainerDefinition,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<Self> {
        let attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;

        // Resolve child node indices
        let mut children = Vec::new();
        for child_id in &def.children {
            if let Some(&node_idx) = node_map.get(child_id) {
                children.push(node_idx);
            } else {
                return Err(BuildError::UnknownNode(child_id.clone()).into());
            }
        }

        if children.is_empty() {
            return Err(BuildError::EmptyContainer(
                def.id
                    .clone()
                    .unwrap_or_else(|| def.label.clone().unwrap_or_else(|| "unnamed".to_string())),
            )
            .into());
        }

        Ok(ContainerData {
            id: def.id,
            label: def.label,
            children,
            attributes,
            bounds: None,
        })
    }
}

impl GroupData {
    pub fn from_definition(
        def: GroupDefinition,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<Self> {
        let attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;

        // Resolve child node indices
        let mut children = Vec::new();
        for child_id in &def.children {
            if let Some(&node_idx) = node_map.get(child_id) {
                children.push(node_idx);
            } else {
                return Err(BuildError::UnknownNode(child_id.clone()).into());
            }
        }

        if children.is_empty() {
            return Err(BuildError::EmptyContainer(
                def.id.clone()
            )
            .into());
        }

        Ok(GroupData {
            id: def.id,
            label: def.label,
            group_type: def.group_type,
            children,
            attributes,
            bounds: None,
        })
    }
}

impl ExcalidrawAttributes {
    pub fn from_hashmap(attrs: &HashMap<String, AttributeValue>) -> Result<Self> {
        let mut excalidraw_attrs = ExcalidrawAttributes::default();

        for (key, value) in attrs {
            match key.as_str() {
                "shape" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.shape = Some(s.to_string());
                    }
                }
                "width" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.width = Some(n);
                    }
                }
                "height" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.height = Some(n);
                    }
                }
                "strokeColor" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.stroke_color = Some(s.to_string());
                    }
                }
                "strokeWidth" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.stroke_width = Some(n);
                    }
                }
                "strokeStyle" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.stroke_style = StrokeStyle::from_str(s);
                    }
                }
                "backgroundColor" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.background_color = Some(s.to_string());
                    }
                }
                "fill" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.fill_style = FillStyle::from_str(s);
                    }
                }
                "fillWeight" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.fill_weight = Some(n as u8);
                    }
                }
                "roughness" => {
                    if let Some(n) = value.as_number() {
                        let roughness = n as u8;
                        if roughness > 2 {
                            return Err(BuildError::InvalidAttribute {
                                attribute: "roughness".to_string(),
                                value: n.to_string(),
                            }
                            .into());
                        }
                        excalidraw_attrs.roughness = Some(roughness);
                    }
                }
                "font" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.font = Some(s.to_string());
                    }
                }
                "fontSize" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.font_size = Some(n);
                    }
                }
                "startArrowhead" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.start_arrowhead = ArrowheadType::from_str(s);
                    }
                }
                "endArrowhead" => {
                    if let Some(s) = value.as_string() {
                        excalidraw_attrs.end_arrowhead = ArrowheadType::from_str(s);
                    }
                }
                "rounded" => {
                    if let Some(n) = value.as_number() {
                        excalidraw_attrs.rounded = Some(n);
                    }
                }
                _ => {
                    // Unknown attribute - could log a warning here
                }
            }
        }

        Ok(excalidraw_attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_igr_from_simple_ast() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![EdgeDefinition {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            }],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();

        assert_eq!(igr.graph.node_count(), 2);
        assert_eq!(igr.graph.edge_count(), 1);
        assert_eq!(igr.node_map.len(), 2);
    }

    #[test]
    fn test_duplicate_node_error() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: None,
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "a".to_string(), // Duplicate!
                    label: None,
                    component_type: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let result = IntermediateGraph::from_ast(document);
        assert!(matches!(
            result,
            Err(crate::error::EDSLError::Build(BuildError::DuplicateNode(_)))
        ));
    }
}
