// src/generator.rs
use crate::ast::{ArrowType, ArrowheadType, FillStyle, GroupType, StrokeStyle};
use crate::error::{GeneratorError, Result};
use crate::igr::{ContainerData, EdgeData, GroupData, IntermediateGraph, NodeData};
use crate::routing::EdgeRouter;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// String constants to avoid repeated allocations
const EXCALIDRAW_TYPE: &str = "excalidraw";
const EXCALIDRAW_SOURCE: &str = "https://excalidraw-dsl.com";
const DEFAULT_BACKGROUND_COLOR: &str = "#ffffff";
const DEFAULT_STROKE_COLOR: &str = "#000000";
const DEFAULT_FILL_STYLE: &str = "solid";
const DEFAULT_STROKE_STYLE: &str = "solid";
const TEXT_ALIGN_CENTER: &str = "center";
const TEXT_ALIGN_LEFT: &str = "left";
const VERTICAL_ALIGN_MIDDLE: &str = "middle";
const VERTICAL_ALIGN_TOP: &str = "top";
const ELEMENT_TYPE_RECTANGLE: &str = "rectangle";
const ELEMENT_TYPE_ELLIPSE: &str = "ellipse";
const ELEMENT_TYPE_DIAMOND: &str = "diamond";
const ELEMENT_TYPE_ARROW: &str = "arrow";
const ELEMENT_TYPE_TEXT: &str = "text";

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcalidrawFile {
    pub r#type: String,
    pub version: u32,
    pub source: String,
    pub elements: Vec<ExcalidrawElementSkeleton>,
    #[serde(rename = "appState")]
    pub app_state: AppState,
    pub files: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    #[serde(rename = "gridSize")]
    pub grid_size: Option<u32>,
    #[serde(rename = "viewBackgroundColor")]
    pub view_background_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcalidrawElementSkeleton {
    pub r#type: String,
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub angle: i32,
    #[serde(rename = "strokeColor")]
    pub stroke_color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "fillStyle")]
    pub fill_style: String,
    #[serde(rename = "strokeWidth")]
    pub stroke_width: i32,
    #[serde(rename = "strokeStyle")]
    pub stroke_style: String,
    pub roughness: u8,
    pub opacity: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: i32,
    #[serde(rename = "fontFamily")]
    pub font_family: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "startBinding")]
    pub start_binding: Option<ElementBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "endBinding")]
    pub end_binding: Option<ElementBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "startArrowhead")]
    pub start_arrowhead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "endArrowhead")]
    pub end_arrowhead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<[i32; 2]>>,
    pub seed: i32,
    pub version: i32,
    #[serde(rename = "versionNonce")]
    pub version_nonce: i32,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
    #[serde(rename = "groupIds")]
    pub group_ids: Vec<String>,
    #[serde(rename = "frameId")]
    pub frame_id: Option<String>,
    pub roundness: Option<serde_json::Value>,
    #[serde(rename = "boundElements")]
    pub bound_elements: Vec<serde_json::Value>,
    pub updated: u64,
    pub link: Option<String>,
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "containerId")]
    pub container_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "verticalAlign")]
    pub vertical_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isContainer")]
    pub is_container: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementBinding {
    #[serde(rename = "elementId")]
    pub element_id: String,
    pub focus: i32,
    pub gap: i32,
}

/// Generator for converting intermediate graph representation to Excalidraw format
///
/// The ExcalidrawGenerator is responsible for the final step in the EDSL compilation
/// pipeline - converting the laid-out intermediate graph into Excalidraw JSON format
/// that can be imported into the Excalidraw application.
///
/// # Design
///
/// The generator performs several key transformations:
/// - Converts node positions to Excalidraw coordinates
/// - Maps EDSL styling attributes to Excalidraw properties
/// - Generates text elements for node labels
/// - Creates arrow elements for edges with proper bindings
/// - Handles containers and groups as frame elements
///
/// # Performance
///
/// The generator uses string constants to minimize allocations and employs
/// efficient data structures for element generation.
pub struct ExcalidrawGenerator;

impl ExcalidrawGenerator {
    /// Get the render order for containers (parent containers first, then children)
    fn get_container_render_order(containers: &[ContainerData]) -> Vec<usize> {
        let mut order = Vec::new();
        let mut visited = vec![false; containers.len()];

        // Find root containers (those without parents)
        for i in 0..containers.len() {
            if containers[i].parent_container.is_none() && !visited[i] {
                Self::visit_container_tree(i, containers, &mut visited, &mut order);
            }
        }

        order
    }

    fn visit_container_tree(
        idx: usize,
        containers: &[ContainerData],
        visited: &mut Vec<bool>,
        order: &mut Vec<usize>,
    ) {
        if visited[idx] {
            return;
        }

        visited[idx] = true;
        order.push(idx);

        // Visit nested containers
        for &nested_idx in &containers[idx].nested_containers {
            Self::visit_container_tree(nested_idx, containers, visited, order);
        }
    }

    /// Get the render order for groups (parent groups first, then children)
    fn get_group_render_order(groups: &[GroupData]) -> Vec<usize> {
        let mut order = Vec::new();
        let mut visited = vec![false; groups.len()];

        // Find root groups (those without parents)
        for i in 0..groups.len() {
            if groups[i].parent_group.is_none()
                && groups[i].parent_container.is_none()
                && !visited[i]
            {
                Self::visit_group_tree(i, groups, &mut visited, &mut order);
            }
        }

        order
    }

    fn visit_group_tree(
        idx: usize,
        groups: &[GroupData],
        visited: &mut Vec<bool>,
        order: &mut Vec<usize>,
    ) {
        if visited[idx] {
            return;
        }

        visited[idx] = true;
        order.push(idx);

        // Visit nested groups
        for &nested_idx in &groups[idx].nested_groups {
            Self::visit_group_tree(nested_idx, groups, visited, order);
        }
    }

    /// Generate a complete Excalidraw file from an intermediate graph
    ///
    /// This is the main entry point for the generator. It creates a complete
    /// Excalidraw file structure including metadata, app state, and all elements.
    ///
    /// # Arguments
    /// * `igr` - The intermediate graph representation containing positioned nodes and edges
    ///
    /// # Returns
    /// * `Ok(ExcalidrawFile)` - Complete Excalidraw file ready for serialization
    /// * `Err(GeneratorError)` - If element generation fails
    ///
    /// # Examples
    /// ```rust
    /// use excalidraw_dsl::generator::ExcalidrawGenerator;
    /// use excalidraw_dsl::igr::IntermediateGraph;
    ///
    /// let igr = IntermediateGraph::new();
    /// let file = ExcalidrawGenerator::generate_file(&igr).unwrap();
    /// ```
    pub fn generate_file(igr: &IntermediateGraph) -> Result<ExcalidrawFile> {
        let elements = Self::generate(igr)?;

        Ok(ExcalidrawFile {
            r#type: EXCALIDRAW_TYPE.to_string(),
            version: 2,
            source: EXCALIDRAW_SOURCE.to_string(),
            elements,
            app_state: AppState {
                grid_size: None,
                view_background_color: DEFAULT_BACKGROUND_COLOR.to_string(),
            },
            files: serde_json::json!({}),
        })
    }

    pub fn generate(igr: &IntermediateGraph) -> Result<Vec<ExcalidrawElementSkeleton>> {
        let mut elements = Vec::new();
        let mut node_id_map = std::collections::HashMap::new();
        let mut node_element_indices = std::collections::HashMap::new();

        // Generate group elements first (visual grouping rectangles) in depth-first order
        let group_order = Self::get_group_render_order(&igr.groups);
        for &group_idx in &group_order {
            let group = &igr.groups[group_idx];
            if let Some(mut group_element) = Self::generate_group(group)? {
                let group_id = group_element.id.clone();

                // Generate text element for group if it has a label
                if let Some(label) = &group.label {
                    if !label.is_empty() {
                        if let Some(bounds) = &group.bounds {
                            let text_element = Self::generate_container_text_element(
                                label,
                                bounds.x + 10.0, // 10px padding from left edge
                                bounds.y + 10.0, // 10px padding from top edge
                                &group_id,
                                group.attributes.font_size.unwrap_or(16.0),
                                &group.attributes.font,
                            )?;

                            // Add reference to text element in the group's boundElements
                            group_element.bound_elements.push(serde_json::json!({
                                "id": text_element.id.clone(),
                                "type": ELEMENT_TYPE_TEXT
                            }));

                            elements.push(group_element);
                            elements.push(text_element);
                        } else {
                            elements.push(group_element);
                        }
                    } else {
                        elements.push(group_element);
                    }
                } else {
                    elements.push(group_element);
                }
            }
        }

        // Generate container elements in depth-first order (parent containers first)
        let container_order = Self::get_container_render_order(&igr.containers);
        for &container_idx in &container_order {
            let container = &igr.containers[container_idx];
            if let Some(mut container_element) = Self::generate_container(container)? {
                let container_id = container_element.id.clone();

                // Generate text element for container if it has a label
                if let Some(label) = &container.label {
                    if !label.is_empty() {
                        if let Some(bounds) = &container.bounds {
                            let text_element = Self::generate_container_text_element(
                                label,
                                bounds.x + 10.0, // 10px padding from left edge
                                bounds.y + 10.0, // 10px padding from top edge
                                &container_id,
                                container.attributes.font_size.unwrap_or(16.0),
                                &container.attributes.font,
                            )?;

                            // Add reference to text element in the container's boundElements
                            container_element.bound_elements.push(serde_json::json!({
                                "id": text_element.id.clone(),
                                "type": ELEMENT_TYPE_TEXT
                            }));

                            elements.push(container_element);
                            elements.push(text_element);
                        } else {
                            elements.push(container_element);
                        }
                    } else {
                        elements.push(container_element);
                    }
                } else {
                    elements.push(container_element);
                }
            }
        }

        // Generate node elements
        for (_, node_data) in igr.graph.node_references() {
            let element_id = format!("node_{}", Uuid::new_v4());
            let mut element = Self::generate_node(node_data, &element_id)?;
            node_id_map.insert(node_data.id.clone(), element_id.clone());

            // Remove text from shape element (it will be a separate element)
            let label = element.text.take();

            // Track the actual index where this node element is pushed
            let node_index = elements.len();
            node_element_indices.insert(element_id.clone(), node_index);

            // Generate separate text element for node label
            if let Some(label) = label {
                if !label.is_empty() {
                    let text_element = Self::generate_text_element(
                        &label,
                        node_data.x,
                        node_data.y,
                        &element_id,
                        node_data.attributes.font_size.unwrap_or(20.0),
                        &node_data.attributes.font,
                    )?;

                    // Add reference to text element in the shape's boundElements
                    element.bound_elements.push(serde_json::json!({
                        "id": text_element.id.clone(),
                        "type": ELEMENT_TYPE_TEXT
                    }));

                    elements.push(element);
                    elements.push(text_element);
                } else {
                    elements.push(element);
                }
            } else {
                elements.push(element);
            }
        }

        // Generate edge elements and update node boundElements
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

            let edge_element = Self::generate_edge(
                edge_data,
                source_node,
                target_node,
                source_element_id,
                target_element_id,
            )?;

            let edge_id = edge_element.id.clone();

            // Update source node's boundElements to include this edge
            if let Some(&source_index) = node_element_indices.get(source_element_id) {
                elements[source_index]
                    .bound_elements
                    .push(serde_json::json!({
                        "id": edge_id.clone(),
                        "type": ELEMENT_TYPE_ARROW
                    }));
            }

            // Update target node's boundElements to include this edge
            if let Some(&target_index) = node_element_indices.get(target_element_id) {
                elements[target_index]
                    .bound_elements
                    .push(serde_json::json!({
                        "id": edge_id.clone(),
                        "type": ELEMENT_TYPE_ARROW
                    }));
            }

            elements.push(edge_element);
        }

        Ok(elements)
    }

    fn generate_node(node_data: &NodeData, element_id: &str) -> Result<ExcalidrawElementSkeleton> {
        let shape_type = match node_data.attributes.shape.as_deref() {
            Some("rectangle") | None => ELEMENT_TYPE_RECTANGLE,
            Some("ellipse") => ELEMENT_TYPE_ELLIPSE,
            Some("diamond") => ELEMENT_TYPE_DIAMOND,
            Some("cylinder") => ELEMENT_TYPE_ELLIPSE, // Approximate with ellipse for now
            Some("text") => ELEMENT_TYPE_TEXT,
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
            x: (node_data.x - node_data.width / 2.0).round() as i32,
            y: (node_data.y - node_data.height / 2.0).round() as i32,
            width: node_data.width.round() as i32,
            height: node_data.height.round() as i32,
            angle: 0,
            stroke_color: node_data
                .attributes
                .stroke_color
                .clone()
                .unwrap_or_else(|| DEFAULT_STROKE_COLOR.to_string()),
            background_color: node_data
                .attributes
                .background_color
                .clone()
                .unwrap_or_else(|| "transparent".to_string()),
            fill_style: Self::convert_fill_style(&node_data.attributes.fill_style),
            stroke_width: node_data.attributes.stroke_width.unwrap_or(2.0).round() as i32,
            stroke_style: Self::convert_stroke_style(&node_data.attributes.stroke_style),
            roughness: node_data.attributes.roughness.unwrap_or(0),
            opacity: 100,
            text: if node_data.label.is_empty() {
                None
            } else {
                Some(node_data.label.clone())
            },
            font_size: node_data.attributes.font_size.unwrap_or(20.0).round() as i32,
            font_family: Self::convert_font_family(&node_data.attributes.font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: if shape_type == ELEMENT_TYPE_RECTANGLE {
                if let Some(rounded) = node_data.attributes.rounded {
                    // Convert rounded value to Excalidraw format
                    // Excalidraw uses a radius value for rounded corners
                    Some(serde_json::json!({"type": 3, "value": rounded}))
                } else {
                    Some(serde_json::json!({"type": 3}))
                }
            } else if shape_type == ELEMENT_TYPE_ELLIPSE {
                Some(serde_json::json!({"type": 2}))
            } else {
                None
            },
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: None,
            text_align: None,
            vertical_align: None,
            is_container: None,
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
            x: start_point.0.round() as i32,
            y: start_point.1.round() as i32,
            width: (end_point.0 - start_point.0).round() as i32,
            height: (end_point.1 - start_point.1).round() as i32,
            angle: 0,
            stroke_color: edge_data
                .attributes
                .stroke_color
                .clone()
                .unwrap_or_else(|| DEFAULT_STROKE_COLOR.to_string()),
            background_color: "transparent".to_string(),
            fill_style: DEFAULT_FILL_STYLE.to_string(),
            stroke_width: edge_data.attributes.stroke_width.unwrap_or(2.0).round() as i32,
            stroke_style: Self::convert_stroke_style(&edge_data.attributes.stroke_style),
            roughness: edge_data.attributes.roughness.unwrap_or(0),
            opacity: 100,
            text: edge_data.label.clone(),
            font_size: 16,
            font_family: 3, // Cascadia (Code font)
            start_binding: Some(ElementBinding {
                element_id: source_element_id.to_string(),
                focus: 0,
                gap: 1,
            }),
            end_binding: Some(ElementBinding {
                element_id: target_element_id.to_string(),
                focus: 0,
                gap: 1,
            }),
            start_arrowhead: Self::convert_arrowhead(&edge_data.attributes.start_arrowhead)
                .or_else(|| match edge_data.arrow_type {
                    ArrowType::DoubleArrow => Some(ELEMENT_TYPE_ARROW.to_string()),
                    _ => None,
                }),
            end_arrowhead: Self::convert_arrowhead(&edge_data.attributes.end_arrowhead).or_else(
                || match edge_data.arrow_type {
                    ArrowType::SingleArrow => Some(ELEMENT_TYPE_ARROW.to_string()),
                    ArrowType::DoubleArrow => Some(ELEMENT_TYPE_ARROW.to_string()),
                    _ => None,
                },
            ),
            points: Some(EdgeRouter::route_edge(
                start_point,
                end_point,
                source_node,
                target_node,
                edge_data.routing_type,
            )),
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: Some(serde_json::json!({"type": 2})),
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: None,
            text_align: None,
            vertical_align: None,
            is_container: None,
        })
    }

    fn generate_group(group: &GroupData) -> Result<Option<ExcalidrawElementSkeleton>> {
        let bounds = match &group.bounds {
            Some(bounds) => bounds,
            None => return Ok(None), // Group without bounds
        };

        // Different visual styles for different group types
        let (stroke_color, background_color, stroke_style, stroke_width) = match &group.group_type {
            GroupType::FlowGroup => (
                group
                    .attributes
                    .stroke_color
                    .clone()
                    .unwrap_or_else(|| "#3b82f6".to_string()),
                group
                    .attributes
                    .background_color
                    .clone()
                    .unwrap_or_else(|| "#dbeafe".to_string()),
                group.attributes.stroke_style.unwrap_or(StrokeStyle::Dashed),
                group.attributes.stroke_width.unwrap_or(2.0),
            ),
            GroupType::BasicGroup => (
                group
                    .attributes
                    .stroke_color
                    .clone()
                    .unwrap_or_else(|| "#6b7280".to_string()),
                group
                    .attributes
                    .background_color
                    .clone()
                    .unwrap_or_else(|| "#f3f4f6".to_string()),
                group.attributes.stroke_style.unwrap_or(StrokeStyle::Solid),
                group.attributes.stroke_width.unwrap_or(1.0),
            ),
            GroupType::SemanticGroup(group_type) => {
                // Different colors for different semantic types
                let (default_stroke, default_bg) = match group_type.as_str() {
                    "service" => ("#8b5cf6", "#f3e8ff"),
                    "layer" => ("#f59e0b", "#fef3c7"),
                    "component" => ("#10b981", "#d1fae5"),
                    "subsystem" => ("#ef4444", "#fee2e2"),
                    "zone" => ("#06b6d4", "#cffafe"),
                    "cluster" => ("#ec4899", "#fce7f3"),
                    _ => ("#6b7280", "#f3f4f6"),
                };
                (
                    group
                        .attributes
                        .stroke_color
                        .clone()
                        .unwrap_or_else(|| default_stroke.to_string()),
                    group
                        .attributes
                        .background_color
                        .clone()
                        .unwrap_or_else(|| default_bg.to_string()),
                    group.attributes.stroke_style.unwrap_or(StrokeStyle::Solid),
                    group.attributes.stroke_width.unwrap_or(2.0),
                )
            }
        };

        Ok(Some(ExcalidrawElementSkeleton {
            r#type: ELEMENT_TYPE_RECTANGLE.to_string(),
            id: format!("group_{}", Uuid::new_v4()),
            x: bounds.x.round() as i32,
            y: bounds.y.round() as i32,
            width: bounds.width.round() as i32,
            height: bounds.height.round() as i32,
            angle: 0,
            stroke_color,
            background_color,
            fill_style: Self::convert_fill_style(&group.attributes.fill_style),
            stroke_width: stroke_width.round() as i32,
            stroke_style: Self::convert_stroke_style(&Some(stroke_style)),
            roughness: group.attributes.roughness.unwrap_or(0),
            opacity: 30, // Semi-transparent background for groups
            text: None,  // Text will be a separate element
            font_size: group.attributes.font_size.unwrap_or(18.0).round() as i32,
            font_family: Self::convert_font_family(&group.attributes.font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: Some(serde_json::json!({"type": 3})),
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: None,
            text_align: None,
            vertical_align: None,
            is_container: Some(true),
        }))
    }

    fn generate_container(container: &ContainerData) -> Result<Option<ExcalidrawElementSkeleton>> {
        let bounds = match &container.bounds {
            Some(bounds) => bounds,
            None => return Ok(None), // Container without bounds
        };

        Ok(Some(ExcalidrawElementSkeleton {
            r#type: ELEMENT_TYPE_RECTANGLE.to_string(),
            id: format!("container_{}", Uuid::new_v4()),
            x: bounds.x.round() as i32,
            y: bounds.y.round() as i32,
            width: bounds.width.round() as i32,
            height: bounds.height.round() as i32,
            angle: 0,
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
            stroke_width: container.attributes.stroke_width.unwrap_or(1.0).round() as i32,
            stroke_style: Self::convert_stroke_style(&container.attributes.stroke_style),
            roughness: container.attributes.roughness.unwrap_or(0),
            opacity: 50, // Semi-transparent background
            text: None,  // Text will be a separate element
            font_size: container.attributes.font_size.unwrap_or(16.0).round() as i32,
            font_family: Self::convert_font_family(&container.attributes.font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: Some(serde_json::json!({"type": 3})),
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: None,
            text_align: None,
            vertical_align: None,
            is_container: Some(true),
        }))
    }

    fn convert_fill_style(fill_style: &Option<FillStyle>) -> String {
        match fill_style {
            Some(fill) => fill.to_excalidraw_style().to_string(),
            None => DEFAULT_FILL_STYLE.to_string(),
        }
    }

    fn convert_stroke_style(stroke_style: &Option<StrokeStyle>) -> String {
        match stroke_style {
            Some(style) => style.to_excalidraw_style().to_string(),
            None => DEFAULT_FILL_STYLE.to_string(),
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
            Some("Virgil") => 1,
            Some("Helvetica") => 2,
            Some("Cascadia") => 3,
            Some("Code") => 3, // Alias for Cascadia
            None => 3,         // Default to Cascadia (Code font)
            _ => 3,            // Default to Cascadia
        }
    }

    fn calculate_text_dimensions(text: &str, font_size: f64, font_family: u8) -> (i32, i32) {
        // Improved text width calculation matching the IGR logic for consistency
        let char_width_multiplier = match font_family {
            1 => 0.65, // Virgil - slightly wider
            2 => 0.55, // Helvetica - slightly wider
            3 => 0.6,  // Cascadia - wider for better readability
            _ => 0.6,
        };

        // Improved character width calculation with better handling for common characters
        let effective_length = text
            .chars()
            .map(|c| {
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
            })
            .sum::<f64>();

        let text_width = (effective_length * font_size * char_width_multiplier).round() as i32;
        let text_height = (font_size * 1.3).round() as i32; // Slightly more height for better appearance

        (text_width, text_height)
    }

    fn generate_container_text_element(
        text: &str,
        x: f64,
        y: f64,
        container_id: &str,
        font_size: f64,
        font: &Option<String>,
    ) -> Result<ExcalidrawElementSkeleton> {
        let font_family = Self::convert_font_family(font);
        let (text_width, text_height) =
            Self::calculate_text_dimensions(text, font_size, font_family);

        // Position text at top-left (no centering)
        let text_x = x.round() as i32;
        let text_y = y.round() as i32;

        Ok(ExcalidrawElementSkeleton {
            r#type: ELEMENT_TYPE_TEXT.to_string(),
            id: format!("text_{}", Uuid::new_v4()),
            x: text_x,
            y: text_y,
            width: text_width,
            height: text_height,
            angle: 0,
            stroke_color: DEFAULT_STROKE_COLOR.to_string(),
            background_color: "transparent".to_string(),
            fill_style: DEFAULT_FILL_STYLE.to_string(),
            stroke_width: 0,
            stroke_style: DEFAULT_STROKE_STYLE.to_string(),
            roughness: 0,
            opacity: 100,
            text: Some(text.to_string()),
            font_size: font_size.round() as i32,
            font_family: Self::convert_font_family(font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: None,
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: Some(container_id.to_string()),
            text_align: Some(TEXT_ALIGN_LEFT.to_string()),
            vertical_align: Some(VERTICAL_ALIGN_TOP.to_string()),
            is_container: None,
        })
    }

    fn generate_text_element(
        text: &str,
        x: f64,
        y: f64,
        container_id: &str,
        font_size: f64,
        font: &Option<String>,
    ) -> Result<ExcalidrawElementSkeleton> {
        let font_family = Self::convert_font_family(font);
        let (text_width, text_height) =
            Self::calculate_text_dimensions(text, font_size, font_family);

        // Center the text relative to the given position
        let text_x = (x - text_width as f64 / 2.0).round() as i32;
        let text_y = (y - text_height as f64 / 2.0).round() as i32;

        Ok(ExcalidrawElementSkeleton {
            r#type: ELEMENT_TYPE_TEXT.to_string(),
            id: format!("text_{}", Uuid::new_v4()),
            x: text_x,
            y: text_y,
            width: text_width,
            height: text_height,
            angle: 0,
            stroke_color: DEFAULT_STROKE_COLOR.to_string(),
            background_color: "transparent".to_string(),
            fill_style: DEFAULT_FILL_STYLE.to_string(),
            stroke_width: 0,
            stroke_style: DEFAULT_STROKE_STYLE.to_string(),
            roughness: 0,
            opacity: 100,
            text: Some(text.to_string()),
            font_size: font_size.round() as i32,
            font_family: Self::convert_font_family(font),
            start_binding: None,
            end_binding: None,
            start_arrowhead: None,
            end_arrowhead: None,
            points: None,
            seed: rand::random::<i32>().abs(),
            version: 1,
            version_nonce: rand::random::<i32>().abs(),
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            roundness: None,
            bound_elements: vec![],
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64,
            link: None,
            locked: false,
            container_id: Some(container_id.to_string()),
            text_align: Some(TEXT_ALIGN_CENTER.to_string()),
            vertical_align: Some(VERTICAL_ALIGN_MIDDLE.to_string()),
            is_container: None,
        })
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

        assert_eq!(result.r#type, ELEMENT_TYPE_RECTANGLE);
        assert_eq!(result.text, Some("Test Node".to_string()));
        assert_eq!(result.x, 40); // 100 - 60 (half width)
        assert_eq!(result.y, 70); // 100 - 30 (half height)
        assert_eq!(result.width, 120);
        assert_eq!(result.height, 60);
    }

    #[test]
    fn test_generate_from_igr() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
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
                label: Some("Edge".to_string()),
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            }],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let mut igr = IntermediateGraph::from_ast(document).unwrap();

        // Simulate layout being done
        igr.graph.node_weights_mut().for_each(|node| {
            node.x = 100.0;
            node.y = 100.0;
        });

        let elements = ExcalidrawGenerator::generate(&igr).unwrap();

        // Should have 2 nodes + 2 text elements + 1 edge = 5 elements
        assert_eq!(elements.len(), 5);

        // Check node elements
        let node_elements: Vec<_> = elements
            .iter()
            .filter(|e| e.r#type == ELEMENT_TYPE_RECTANGLE)
            .collect();
        assert_eq!(node_elements.len(), 2);

        // Check edge elements
        let edge_elements: Vec<_> = elements
            .iter()
            .filter(|e| e.r#type == ELEMENT_TYPE_ARROW)
            .collect();
        assert_eq!(edge_elements.len(), 1);

        let edge = &edge_elements[0];
        assert_eq!(edge.text, Some("Edge".to_string()));
        assert!(edge.start_binding.is_some());
        assert!(edge.end_binding.is_some());
        assert_eq!(edge.end_arrowhead, Some(ELEMENT_TYPE_ARROW.to_string()));
    }
}
