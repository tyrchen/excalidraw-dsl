// src/template.rs
use crate::ast::*;
use crate::error::Result;
use std::collections::HashMap;

/// Template processor for expanding templates into concrete nodes and edges
pub struct TemplateProcessor {
    templates: HashMap<String, TemplateDefinition>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Add a template to the processor
    pub fn add_template(&mut self, template: TemplateDefinition) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Process a parsed document and expand templates
    pub fn process_document(&self, mut doc: ParsedDocument) -> Result<ParsedDocument> {
        // Add all templates from the document
        for name in doc.templates.keys() {
            if !self.templates.contains_key(name) {
                // Add template to local registry if not already present
            }
        }

        // If there's a diagram that uses a template, expand it
        if let Some(diagram) = &doc.diagram {
            if let Some(template_name) = &diagram.template {
                if let Some(template) = doc.templates.get(template_name) {
                    let (nodes, edges) = self.expand_template(template, diagram)?;
                    doc.nodes.extend(nodes);
                    doc.edges.extend(edges);
                }
            }
        }

        Ok(doc)
    }

    /// Expand a template into nodes and edges
    fn expand_template(
        &self,
        template: &TemplateDefinition,
        _diagram: &DiagramDefinition,
    ) -> Result<(Vec<NodeDefinition>, Vec<EdgeDefinition>)> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Create nodes from template layers
        let mut layer_positions = HashMap::new();
        let mut all_components = Vec::new();

        for (layer_idx, layer) in template.layers.iter().enumerate() {
            let mut layer_components = Vec::new();

            // Create nodes for each component in the layer
            for component_name in layer.components.iter() {
                let node_id = format!(
                    "{}_{}",
                    layer.name.to_lowercase().replace(" ", "_"),
                    component_name.to_lowercase().replace(" ", "_")
                );

                let node = NodeDefinition {
                    id: node_id.clone(),
                    label: Some(component_name.clone()),
                    component_type: None,
                    attributes: HashMap::new(),
                };

                nodes.push(node);
                layer_components.push(node_id.clone());
                all_components.push(node_id);
            }

            layer_positions.insert(layer_idx, layer_components);
        }

        // Create connections based on the template pattern
        if let Some(pattern) = &template.connections {
            match pattern {
                ConnectionPattern::EachToNextLayer => {
                    edges.extend(self.create_layer_to_layer_connections(&layer_positions)?);
                }
                ConnectionPattern::Star(center_component) => {
                    edges.extend(self.create_star_connections(&all_components, center_component)?);
                }
                ConnectionPattern::Mesh => {
                    edges.extend(self.create_mesh_connections(&all_components)?);
                }
                ConnectionPattern::Custom(connections) => {
                    edges.extend(self.create_custom_connections(connections)?);
                }
            }
        }

        Ok((nodes, edges))
    }

    /// Create connections from each layer to the next layer
    fn create_layer_to_layer_connections(
        &self,
        layer_positions: &HashMap<usize, Vec<String>>,
    ) -> Result<Vec<EdgeDefinition>> {
        let mut edges = Vec::new();
        let layer_count = layer_positions.len();

        for layer_idx in 0..layer_count - 1 {
            if let (Some(current_layer), Some(next_layer)) = (
                layer_positions.get(&layer_idx),
                layer_positions.get(&(layer_idx + 1)),
            ) {
                // Connect each node in current layer to each node in next layer
                for from_node in current_layer {
                    for to_node in next_layer {
                        let edge = EdgeDefinition {
                            from: from_node.clone(),
                            to: to_node.clone(),
                            label: None,
                            arrow_type: ArrowType::SingleArrow,
                            attributes: HashMap::new(),
                            style: None,
                        };
                        edges.push(edge);
                    }
                }
            }
        }

        Ok(edges)
    }

    /// Create star pattern connections
    fn create_star_connections(
        &self,
        all_components: &[String],
        center_component: &str,
    ) -> Result<Vec<EdgeDefinition>> {
        let mut edges = Vec::new();

        // Find the center component
        let center_id = all_components
            .iter()
            .find(|id| id.contains(&center_component.to_lowercase().replace(" ", "_")))
            .cloned()
            .unwrap_or_else(|| all_components.first().unwrap().clone());

        // Connect center to all other components
        for component_id in all_components {
            if component_id != &center_id {
                let edge = EdgeDefinition {
                    from: center_id.clone(),
                    to: component_id.clone(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                };
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    /// Create mesh (all-to-all) connections
    fn create_mesh_connections(&self, all_components: &[String]) -> Result<Vec<EdgeDefinition>> {
        let mut edges = Vec::new();

        for (i, from_component) in all_components.iter().enumerate() {
            for (j, to_component) in all_components.iter().enumerate() {
                if i != j {
                    let edge = EdgeDefinition {
                        from: from_component.clone(),
                        to: to_component.clone(),
                        label: None,
                        arrow_type: ArrowType::SingleArrow,
                        attributes: HashMap::new(),
                        style: None,
                    };
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    /// Create custom connections
    fn create_custom_connections(
        &self,
        connections: &[(String, String)],
    ) -> Result<Vec<EdgeDefinition>> {
        let mut edges = Vec::new();

        for (from, to) in connections {
            let edge = EdgeDefinition {
                from: from.clone(),
                to: to.clone(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            };
            edges.push(edge);
        }

        Ok(edges)
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}
