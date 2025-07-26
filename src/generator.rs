// src/generator.rs
use crate::ast::{ArrowType, ArrowheadType, FillStyle, StrokeStyle};
use crate::error::{GeneratorError, Result};
use crate::igr::{ContainerData, EdgeData, IntermediateGraph, NodeData};
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcalidrawElementSkeleton {
    pub r#type: String,
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    #[serde(rename = "strokeColor")]
    pub stroke_color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "fillStyle")]
    pub fill_style: String,
    #[serde(rename = "strokeWidth")]
    pub stroke_width: f64,
    #[serde(rename = "strokeStyle")]
    pub stroke_style: String,
    pub roughness: u8,
    pub opacity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: f64,
    #[serde(rename = "fontFamily")]
    pub font_family: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_binding: Option<ElementBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_binding: Option<ElementBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_arrowhead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_arrowhead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<[f64; 2]>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementBinding {
    #[serde(rename = "elementId")]
    pub element_id: String,
    pub focus: f64,
    pub gap: f64,
}

pub struct ExcalidrawGenerator;

impl ExcalidrawGenerator {
    pub fn generate(igr: &IntermediateGraph) -> Result<Vec<ExcalidrawElementSkeleton>> {
        let mut elements = Vec::new();
        let mut node_id_map = std::collections::HashMap::new();

        // Generate container elements first (background rectangles)
        for container in &igr.containers {
            if let Some(container_element) = Self::generate_container(container)? {
                elements.push(container_element);
            }
        }

        // Generate node elements
        for (_, node_data) in igr.graph.node_references() {
            let element_id = format!("node_{}", Uuid::new_v4());
            let element = Self::generate_node(node_data, &element_id)?;
            node_id_map.insert(node_data.id.clone(), element_id.clone());
            elements.push(element);
        }

        // Generate edge elements
        for edge_ref in igr.graph.edge_references() {
            let source_node = &igr.graph[edge_ref.source()];
            let target_node = &igr.graph[edge_ref.target()];
            let edge_data = edge_ref.weight();

            let source_element_id = node_id_map.get(&source_node.id).ok_or_else(|| {
                GeneratorError::GenerationFailed(format!(
                    "Source node {} not found in node map",
                    source_node.id
                ))
            })?;
            let target_element_id = node_id_map.get(&target_node.id).ok_or_else(|| {
                GeneratorError::GenerationFailed(format!(
                    "Target node {} not found in node map",
                    target_node.id
                ))
            })?;

            let element = Self::generate_edge(
                edge_data,
                source_node,
                target_node,
                source_element_id,
                target_element_id,
            )?;
            elements.push(element);
        }

        Ok(elements)
    }

    fn generate_node(node_data: &NodeData, element_id: &str) -> Result<ExcalidrawElementSkeleton> {
        let shape_type = match node_data.attributes.shape.as_deref() {
            Some("rectangle") | None => "rectangle",
            Some("ellipse") => "ellipse",
            Some("diamond") => "diamond",
            Some("cylinder") => "ellipse", // Approximate with ellipse for now
            Some("text") => "text",
            shape => {
                return Err(GeneratorError::InvalidElementType(
                    shape.unwrap_or("unknown").to_string(),
                )
                .into())
            }
        };

        // Validate coordinates
        if !node_data.x.is_finite() || !node_data.y.is_finite() {
            return Err(GeneratorError::InvalidCoordinate {
                x: node_data.x,
                y: node_data.y,
            }
            .into());
        }

        Ok(ExcalidrawElementSkeleton {
            r#type: shape_type.to_string(),
            id: element_id.to_string(),
            x: node_data.x - node_data.width / 2.0,
            y: node_data.y - node_data.height / 2.0,
            width: node_data.width,
            height: node_data.height,
            angle: 0.0,
            stroke_color: node_data
                .attributes
                .stroke_color
                .clone()
                .unwrap_or_else(|| "#000000".to_string()),
            background_color: node_data
                .attributes
                .background_color
                .clone()
                .unwrap_or_else(|| "transparent".to_string()),
            fill_style: Self::convert_fill_style(&node_data.attributes.fill_style),
            stroke_width: node_data.attributes.stroke_width.unwrap_or(2.0),
            stroke_style: Self::convert_stroke_style(&node_data.attributes.stroke_style),
            roughness: node_data.attributes.roughness.unwrap_or(1),
            opacity: 100.0,
            text: if node_data.label.is_empty() {
                None
            } else {
                Some(node_data.label.clone())
            },
            font_size: node_data.attributes.font_size.unwrap_or(20.0),
            font_family: Self::convert_font_family(&node_data.attributes.font),
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
        source_element_id: &str,
        target_element_id: &str,
    ) -> Result<ExcalidrawElementSkeleton> {
        // Calculate connection points
        let start_point = Self::calculate_connection_point(source_node, target_node, true);
        let end_point = Self::calculate_connection_point(target_node, source_node, false);

        let element_type = edge_data.arrow_type.to_excalidraw_type();

        // Validate coordinates
        if !start_point.0.is_finite()
            || !start_point.1.is_finite()
            || !end_point.0.is_finite()
            || !end_point.1.is_finite()
        {
            return Err(GeneratorError::InvalidCoordinate {
                x: start_point.0,
                y: start_point.1,
            }
            .into());
        }

        Ok(ExcalidrawElementSkeleton {
            r#type: element_type.to_string(),
            id: format!("edge_{}", Uuid::new_v4()),
            x: start_point.0,
            y: start_point.1,
            width: end_point.0 - start_point.0,
            height: end_point.1 - start_point.1,
            angle: 0.0,
            stroke_color: edge_data
                .attributes
                .stroke_color
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
            font_family: 1, // Virgil
            start_binding: Some(ElementBinding {
                element_id: source_element_id.to_string(),
                focus: 0.0,
                gap: 0.0,
            }),
            end_binding: Some(ElementBinding {
                element_id: target_element_id.to_string(),
                focus: 0.0,
                gap: 0.0,
            }),
            start_arrowhead: Self::convert_arrowhead(&edge_data.attributes.start_arrowhead),
            end_arrowhead: Self::convert_arrowhead(&edge_data.attributes.end_arrowhead).or_else(
                || match edge_data.arrow_type {
                    ArrowType::SingleArrow => Some("triangle".to_string()),
                    ArrowType::DoubleArrow => Some("triangle".to_string()),
                    _ => None,
                },
            ),
            points: Some(vec![
                [0.0, 0.0],
                [end_point.0 - start_point.0, end_point.1 - start_point.1],
            ]),
        })
    }

    fn generate_container(container: &ContainerData) -> Result<Option<ExcalidrawElementSkeleton>> {
        let bounds = match &container.bounds {
            Some(bounds) => bounds,
            None => return Ok(None), // Container without bounds
        };

        Ok(Some(ExcalidrawElementSkeleton {
            r#type: "rectangle".to_string(),
            id: format!("container_{}", Uuid::new_v4()),
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
            angle: 0.0,
            stroke_color: container
                .attributes
                .stroke_color
                .clone()
                .unwrap_or_else(|| "#868e96".to_string()),
            background_color: container
                .attributes
                .background_color
                .clone()
                .unwrap_or_else(|| "#f8f9fa".to_string()),
            fill_style: Self::convert_fill_style(&container.attributes.fill_style),
            stroke_width: container.attributes.stroke_width.unwrap_or(1.0),
            stroke_style: Self::convert_stroke_style(&container.attributes.stroke_style),
            roughness: container.attributes.roughness.unwrap_or(0),
            opacity: 50.0, // Semi-transparent background
            text: container.label.clone(),
            font_size: container.attributes.font_size.unwrap_or(16.0),
            font_family: Self::convert_font_family(&container.attributes.font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
        }))
    }

    fn convert_fill_style(fill_style: &Option<FillStyle>) -> String {
        match fill_style {
            Some(fill) => fill.to_excalidraw_style().to_string(),
            None => "solid".to_string(),
        }
    }

    fn convert_stroke_style(stroke_style: &Option<StrokeStyle>) -> String {
        match stroke_style {
            Some(style) => style.to_excalidraw_style().to_string(),
            None => "solid".to_string(),
        }
    }

    fn convert_arrowhead(arrowhead: &Option<ArrowheadType>) -> Option<String> {
        arrowhead
            .as_ref()
            .and_then(|a| a.to_excalidraw_type())
            .map(|s| s.to_string())
    }

    fn convert_font_family(font: &Option<String>) -> u8 {
        match font.as_deref() {
            Some("Virgil") | None => 1,
            Some("Helvetica") => 2,
            Some("Cascadia") => 3,
            _ => 1, // Default to Virgil
        }
    }

    fn calculate_connection_point(
        from_node: &NodeData,
        to_node: &NodeData,
        _is_start: bool,
    ) -> (f64, f64) {
        // Simple edge connection calculation
        // Calculate the direction from center to center
        let center_x = from_node.x;
        let center_y = from_node.y;

        let target_center_x = to_node.x;
        let target_center_y = to_node.y;

        let dx = target_center_x - center_x;
        let dy = target_center_y - center_y;
        let length = (dx * dx + dy * dy).sqrt();

        if length == 0.0 {
            return (center_x, center_y);
        }

        // Find intersection with node boundary (simplified rectangular intersection)
        let norm_dx = dx / length;
        let norm_dy = dy / length;

        let half_width = from_node.width / 2.0;
        let half_height = from_node.height / 2.0;

        // Calculate intersection with rectangle boundary
        let t_x = if norm_dx != 0.0 {
            half_width / norm_dx.abs()
        } else {
            f64::INFINITY
        };
        let t_y = if norm_dy != 0.0 {
            half_height / norm_dy.abs()
        } else {
            f64::INFINITY
        };

        let t = t_x.min(t_y);

        let edge_x = center_x + t * norm_dx;
        let edge_y = center_y + t * norm_dy;

        (edge_x, edge_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::igr::{ExcalidrawAttributes, IntermediateGraph};
    use std::collections::HashMap;

    #[test]
    fn test_generate_simple_node() {
        let node_data = NodeData {
            id: "test".to_string(),
            label: "Test Node".to_string(),
            attributes: ExcalidrawAttributes::default(),
            x: 100.0,
            y: 100.0,
            width: 120.0,
            height: 60.0,
        };

        let result = ExcalidrawGenerator::generate_node(&node_data, "test_id").unwrap();

        assert_eq!(result.r#type, "rectangle");
        assert_eq!(result.text, Some("Test Node".to_string()));
        assert_eq!(result.x, 40.0); // 100 - 60 (half width)
        assert_eq!(result.y, 70.0); // 100 - 30 (half height)
        assert_eq!(result.width, 120.0);
        assert_eq!(result.height, 60.0);
    }

    #[test]
    fn test_generate_from_igr() {
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
                label: Some("Edge".to_string()),
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
            }],
            containers: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let elements = ExcalidrawGenerator::generate(&igr).unwrap();

        // Should have 2 nodes + 1 edge = 3 elements
        assert_eq!(elements.len(), 3);

        // Check node elements
        let node_elements: Vec<_> = elements
            .iter()
            .filter(|e| e.r#type == "rectangle")
            .collect();
        assert_eq!(node_elements.len(), 2);

        // Check edge elements
        let edge_elements: Vec<_> = elements.iter().filter(|e| e.r#type == "arrow").collect();
        assert_eq!(edge_elements.len(), 1);

        let edge = &edge_elements[0];
        assert_eq!(edge.text, Some("Edge".to_string()));
        assert!(edge.start_binding.is_some());
        assert!(edge.end_binding.is_some());
        assert_eq!(edge.end_arrowhead, Some("triangle".to_string()));
    }
}
