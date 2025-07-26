// src/igr.rs
use crate::ast::*;
use crate::error::{BuildError, Result};
use petgraph::graph::{DiGraph, NodeIndex};
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
            containers: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn from_ast(document: ParsedDocument) -> Result<Self> {
        let mut igr = IntermediateGraph::new();
        igr.global_config = document.config;

        // First pass: Build all nodes from top-level and containers
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
                }
            }
        }

        // Build nodes
        for node_def in all_nodes {
            if igr.node_map.contains_key(&node_def.id) {
                return Err(BuildError::DuplicateNode(node_def.id).into());
            }

            let node_data = NodeData::from_definition(node_def)?;
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
    pub fn from_definition(def: NodeDefinition) -> Result<Self> {
        let attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;
        let label = def.label.unwrap_or_else(|| def.id.clone());

        // Estimate initial dimensions based on label
        let estimated_width = attributes
            .width
            .unwrap_or_else(|| (label.len() as f64 * 8.0 + 40.0).max(80.0));
        let estimated_height = attributes.height.unwrap_or(60.0);

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
        let attributes = ExcalidrawAttributes::from_hashmap(&def.attributes)?;

        Ok(EdgeData {
            label: def.label,
            arrow_type: def.arrow_type,
            attributes,
        })
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
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![EdgeDefinition {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
            }],
            containers: vec![],
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
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "a".to_string(), // Duplicate!
                    label: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![],
            containers: vec![],
        };

        let result = IntermediateGraph::from_ast(document);
        assert!(matches!(
            result,
            Err(crate::error::EDSLError::Build(BuildError::DuplicateNode(_)))
        ));
    }
}
