// src/fluent.rs
//! Fluent API for building Excalidraw diagrams programmatically

use crate::ast::{
    ArrowType, AttributeValue, ContainerDefinition, EdgeDefinition, GlobalConfig, GroupDefinition,
    GroupType, NodeDefinition, ParsedDocument, Statement,
};
use crate::{EDSLCompiler, Result};
use std::collections::HashMap;

/// Fluent builder for creating Excalidraw diagrams
pub struct DiagramBuilder {
    config: GlobalConfig,
    nodes: Vec<NodeDefinition>,
    edges: Vec<EdgeDefinition>,
    containers: Vec<ContainerDefinition>,
    groups: Vec<GroupDefinition>,
    #[allow(dead_code)]
    current_container: Option<String>,
}

impl Default for DiagramBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramBuilder {
    /// Create a new diagram builder
    pub fn new() -> Self {
        Self {
            config: GlobalConfig::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            containers: Vec::new(),
            groups: Vec::new(),
            current_container: None,
        }
    }

    /// Set the layout algorithm
    pub fn with_layout(mut self, layout: &str) -> Self {
        self.config.layout = Some(layout.to_string());
        self
    }

    /// Set the theme
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.config.theme = Some(theme.to_string());
        self
    }

    /// Add a node to the diagram
    pub fn node(self, id: &str) -> NodeBuilder {
        NodeBuilder::new(self, id.to_string())
    }

    /// Add an edge between two nodes
    pub fn edge(self, from: &str, to: &str) -> EdgeBuilder {
        EdgeBuilder::new(self, from.to_string(), to.to_string())
    }

    /// Start a container definition
    pub fn container(self, name: &str) -> ContainerBuilder {
        ContainerBuilder::new(self, name.to_string())
    }

    /// Create a group of nodes
    pub fn group(self, label: &str) -> GroupBuilder {
        GroupBuilder::new(self, label.to_string())
    }

    /// Build the diagram and compile to Excalidraw JSON
    pub fn build(self) -> Result<String> {
        let document = ParsedDocument {
            config: self.config,
            nodes: self.nodes,
            edges: self.edges,
            containers: self.containers,
            groups: self.groups,
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            connections: Vec::new(),
        };

        let _compiler = EDSLCompiler::new();
        let mut igr = crate::igr::IntermediateGraph::from_ast(document)?;
        let layout_manager = crate::layout::LayoutManager::new();
        layout_manager.layout(&mut igr)?;

        let file = crate::generator::ExcalidrawGenerator::generate_file(&igr)?;
        serde_json::to_string_pretty(&file).map_err(crate::error::EDSLError::Json)
    }

    /// Build and return the ParsedDocument (for testing or further processing)
    pub fn build_ast(self) -> ParsedDocument {
        ParsedDocument {
            config: self.config,
            nodes: self.nodes,
            edges: self.edges,
            containers: self.containers,
            groups: self.groups,
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            connections: Vec::new(),
        }
    }
}

/// Builder for nodes
pub struct NodeBuilder {
    parent: DiagramBuilder,
    node: NodeDefinition,
}

impl NodeBuilder {
    fn new(parent: DiagramBuilder, id: String) -> Self {
        Self {
            parent,
            node: NodeDefinition {
                id,
                label: None,
                component_type: None,
                attributes: HashMap::new(),
            },
        }
    }

    /// Set the node label
    pub fn label(mut self, label: &str) -> Self {
        self.node.label = Some(label.to_string());
        self
    }

    /// Set the shape
    pub fn shape(mut self, shape: &str) -> Self {
        self.node.attributes.insert(
            "shape".to_string(),
            AttributeValue::String(shape.to_string()),
        );
        self
    }

    /// Set the color
    pub fn color(mut self, color: &str) -> Self {
        self.node.attributes.insert(
            "strokeColor".to_string(),
            AttributeValue::String(color.to_string()),
        );
        self
    }

    /// Set the background color
    pub fn background(mut self, color: &str) -> Self {
        self.node.attributes.insert(
            "backgroundColor".to_string(),
            AttributeValue::String(color.to_string()),
        );
        self
    }

    /// Finish building this node and return to the diagram builder
    pub fn done(mut self) -> DiagramBuilder {
        self.parent.nodes.push(self.node);
        self.parent
    }
}

/// Builder for edges
pub struct EdgeBuilder {
    parent: DiagramBuilder,
    edge: EdgeDefinition,
}

impl EdgeBuilder {
    fn new(parent: DiagramBuilder, from: String, to: String) -> Self {
        Self {
            parent,
            edge: EdgeDefinition {
                from,
                to,
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
        }
    }

    /// Set the edge label
    pub fn label(mut self, label: &str) -> Self {
        self.edge.label = Some(label.to_string());
        self
    }

    /// Set the arrow type
    pub fn arrow_type(mut self, arrow_type: ArrowType) -> Self {
        self.edge.arrow_type = arrow_type;
        self
    }

    /// Set the edge style (dashed, dotted, etc.)
    pub fn style(mut self, style: &str) -> Self {
        self.edge.attributes.insert(
            "strokeStyle".to_string(),
            AttributeValue::String(style.to_string()),
        );
        self
    }

    /// Finish building this edge and return to the diagram builder
    pub fn done(mut self) -> DiagramBuilder {
        self.parent.edges.push(self.edge);
        self.parent
    }
}

/// Builder for containers
pub struct ContainerBuilder {
    parent: DiagramBuilder,
    container: ContainerDefinition,
}

impl ContainerBuilder {
    fn new(parent: DiagramBuilder, name: String) -> Self {
        Self {
            parent,
            container: ContainerDefinition {
                id: Some(name.clone()),
                label: Some(name),
                children: Vec::new(),
                attributes: HashMap::new(),
                internal_statements: Vec::new(),
            },
        }
    }

    /// Add a node to this container
    pub fn with_node(mut self, id: &str, label: Option<&str>) -> Self {
        let node = NodeDefinition {
            id: id.to_string(),
            label: label.map(|s| s.to_string()),
            component_type: None,
            attributes: HashMap::new(),
        };
        self.container
            .internal_statements
            .push(Statement::Node(node));
        self.container.children.push(id.to_string());
        self
    }

    /// Set container color
    pub fn color(mut self, color: &str) -> Self {
        self.container.attributes.insert(
            "strokeColor".to_string(),
            AttributeValue::String(color.to_string()),
        );
        self
    }

    /// Finish building this container and return to the diagram builder
    pub fn done(mut self) -> DiagramBuilder {
        self.parent.containers.push(self.container);
        self.parent
    }
}

/// Builder for groups
pub struct GroupBuilder {
    parent: DiagramBuilder,
    group: GroupDefinition,
}

impl GroupBuilder {
    fn new(parent: DiagramBuilder, label: String) -> Self {
        Self {
            parent,
            group: GroupDefinition {
                id: label.clone(),
                label: Some(label),
                group_type: GroupType::BasicGroup,
                children: Vec::new(),
                attributes: HashMap::new(),
                internal_statements: Vec::new(),
            },
        }
    }

    /// Add members to this group
    pub fn with_members(mut self, members: Vec<&str>) -> Self {
        self.group.children = members.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Finish building this group and return to the diagram builder
    pub fn done(mut self) -> DiagramBuilder {
        self.parent.groups.push(self.group);
        self.parent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluent_api_basic() {
        let diagram = DiagramBuilder::new()
            .with_layout("dagre")
            .with_theme("light")
            .node("user")
            .label("User")
            .shape("rectangle")
            .done()
            .node("api")
            .label("API")
            .shape("rectangle")
            .done()
            .edge("user", "api")
            .label("HTTP Request")
            .done()
            .build_ast();

        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.edges.len(), 1);
        assert_eq!(diagram.config.layout, Some("dagre".to_string()));
    }
}
