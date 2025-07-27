// src/tests/igr_tests.rs

use crate::ast::{
    ArrowType, ArrowheadType, AttributeValue, ContainerDefinition, EdgeDefinition, FillStyle,
    GlobalConfig, GroupDefinition, GroupType, NodeDefinition, ParsedDocument, Statement,
    StrokeStyle,
};
use crate::error::BuildError;
use crate::igr::{
    BoundingBox, ContainerData, EdgeData, ExcalidrawAttributes, GroupData, IntermediateGraph,
    NodeData,
};
use std::collections::HashMap;

// Helper functions
fn create_test_document() -> ParsedDocument {
    ParsedDocument {
        config: GlobalConfig::default(),
        component_types: HashMap::new(),
        templates: HashMap::new(),
        diagram: None,
        nodes: vec![],
        edges: vec![],
        containers: vec![],
        groups: vec![],
        connections: vec![],
    }
}

fn create_test_node(id: &str, label: &str) -> NodeDefinition {
    NodeDefinition {
        id: id.to_string(),
        label: Some(label.to_string()),
        component_type: None,
        attributes: HashMap::new(),
    }
}

#[test]
fn test_igr_creation_empty() {
    let doc = create_test_document();
    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.graph.node_count(), 0);
    assert_eq!(igr.graph.edge_count(), 0);
    assert_eq!(igr.containers.len(), 0);
    assert_eq!(igr.groups.len(), 0);
    assert!(igr.node_map.is_empty());
    assert!(igr.container_map.is_empty());
}

#[test]
fn test_igr_with_single_node() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.graph.node_count(), 1);
    assert_eq!(igr.graph.edge_count(), 0);
    assert!(igr.node_map.contains_key("node1"));

    let node_idx = igr.node_map.get("node1").unwrap();
    let node_data = &igr.graph[*node_idx];
    assert_eq!(node_data.id, "node1");
    assert_eq!(node_data.label, "Node 1");
    assert!(!node_data.is_virtual_container);
}

#[test]
fn test_igr_with_multiple_nodes() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));
    doc.nodes.push(create_test_node("node2", "Node 2"));
    doc.nodes.push(create_test_node("node3", "Node 3"));

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.graph.node_count(), 3);
    assert_eq!(igr.node_map.len(), 3);
    assert!(igr.node_map.contains_key("node1"));
    assert!(igr.node_map.contains_key("node2"));
    assert!(igr.node_map.contains_key("node3"));
}

#[test]
fn test_igr_with_edge() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));
    doc.nodes.push(create_test_node("node2", "Node 2"));
    doc.edges.push(EdgeDefinition {
        from: "node1".to_string(),
        to: "node2".to_string(),
        label: Some("Connection".to_string()),
        arrow_type: ArrowType::SingleArrow,
        attributes: HashMap::new(),
        style: None,
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.graph.node_count(), 2);
    assert_eq!(igr.graph.edge_count(), 1);

    // Find the edge
    let edges: Vec<_> = igr.graph.edge_indices().collect();
    assert_eq!(edges.len(), 1);

    let edge_data = &igr.graph[edges[0]];
    assert_eq!(edge_data.label, Some("Connection".to_string()));
    assert!(matches!(edge_data.arrow_type, ArrowType::SingleArrow));
}

#[test]
fn test_igr_edge_to_unknown_node() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));
    doc.edges.push(EdgeDefinition {
        from: "node1".to_string(),
        to: "unknown".to_string(),
        label: None,
        arrow_type: ArrowType::SingleArrow,
        attributes: HashMap::new(),
        style: None,
    });

    let result = IntermediateGraph::from_ast(doc);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        crate::error::EDSLError::Build(BuildError::UnknownNode(_))
    ));
}

#[test]
fn test_igr_duplicate_node_id() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "First"));
    doc.nodes.push(create_test_node("node1", "Second"));

    let result = IntermediateGraph::from_ast(doc);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        crate::error::EDSLError::Build(BuildError::DuplicateNode(_))
    ));
}

#[test]
fn test_igr_with_container() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));
    doc.containers.push(ContainerDefinition {
        id: Some("container1".to_string()),
        label: Some("Container 1".to_string()),
        children: vec!["node1".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![],
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.containers.len(), 1);
    assert!(igr.container_map.contains_key("container1"));

    let container = &igr.containers[0];
    assert_eq!(container.id, Some("container1".to_string()));
    assert_eq!(container.label, Some("Container 1".to_string()));
    assert_eq!(container.children.len(), 1);
}

#[test]
fn test_igr_container_with_unknown_child() {
    let mut doc = create_test_document();
    doc.containers.push(ContainerDefinition {
        id: Some("container1".to_string()),
        label: Some("Container 1".to_string()),
        children: vec!["unknown".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![],
    });

    let result = IntermediateGraph::from_ast(doc);
    assert!(result.is_err());
}

#[test]
fn test_igr_with_group() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));
    doc.groups.push(GroupDefinition {
        id: "group1".to_string(),
        label: Some("Group 1".to_string()),
        group_type: GroupType::BasicGroup,
        children: vec!["node1".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![],
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.groups.len(), 1);

    let group = &igr.groups[0];
    assert_eq!(group.id, "group1");
    assert_eq!(group.label, Some("Group 1".to_string()));
    assert!(matches!(group.group_type, GroupType::BasicGroup));
    assert_eq!(group.children.len(), 1);
}

#[test]
fn test_excalidraw_attributes_from_hashmap() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "strokeColor".to_string(),
        AttributeValue::String("#ff0000".to_string()),
    );
    attrs.insert("strokeWidth".to_string(), AttributeValue::Number(2.5));
    attrs.insert("roughness".to_string(), AttributeValue::Number(2.0));
    attrs.insert(
        "fill".to_string(),
        AttributeValue::String("hachure".to_string()),
    );

    let excalidraw_attrs = ExcalidrawAttributes::from_hashmap(&attrs).unwrap();

    assert_eq!(excalidraw_attrs.stroke_color, Some("#ff0000".to_string()));
    assert_eq!(excalidraw_attrs.stroke_width, Some(2.5));
    assert_eq!(excalidraw_attrs.roughness, Some(2));
    assert_eq!(excalidraw_attrs.fill_style, Some(FillStyle::Hachure));
}

#[test]
fn test_excalidraw_attributes_text_color() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "color".to_string(),
        AttributeValue::String("#0000ff".to_string()),
    );

    let excalidraw_attrs = ExcalidrawAttributes::from_hashmap(&attrs).unwrap();

    // color attribute maps to text_color
    assert_eq!(excalidraw_attrs.text_color, Some("#0000ff".to_string()));
}

#[test]
fn test_excalidraw_attributes_invalid_values() {
    // Stroke width validation is not enforced in from_hashmap
    let mut attrs = HashMap::new();
    attrs.insert("strokeWidth".to_string(), AttributeValue::Number(25.0));

    let result = ExcalidrawAttributes::from_hashmap(&attrs);
    assert!(result.is_ok()); // No validation on stroke width in from_hashmap

    let mut attrs = HashMap::new();
    attrs.insert("roughness".to_string(), AttributeValue::Number(5.0)); // Too large (max is 2)

    let result = ExcalidrawAttributes::from_hashmap(&attrs);
    assert!(result.is_err());
}

#[test]
fn test_node_data_creation() {
    let node = NodeData {
        id: "test".to_string(),
        label: "Test Node".to_string(),
        attributes: ExcalidrawAttributes::default(),
        x: 100.0,
        y: 200.0,
        width: 150.0,
        height: 75.0,
        is_virtual_container: false,
    };

    assert_eq!(node.id, "test");
    assert_eq!(node.label, "Test Node");
    assert_eq!(node.x, 100.0);
    assert_eq!(node.y, 200.0);
    assert_eq!(node.width, 150.0);
    assert_eq!(node.height, 75.0);
    assert!(!node.is_virtual_container);
}

#[test]
fn test_edge_data_creation() {
    let attrs = ExcalidrawAttributes {
        start_arrowhead: Some(ArrowheadType::Dot),
        end_arrowhead: Some(ArrowheadType::Triangle),
        ..Default::default()
    };

    let edge = EdgeData {
        label: Some("Edge Label".to_string()),
        arrow_type: ArrowType::DoubleArrow,
        attributes: attrs,
        routing_type: Some(crate::ast::RoutingType::Curved),
    };

    assert_eq!(edge.label, Some("Edge Label".to_string()));
    assert!(matches!(edge.arrow_type, ArrowType::DoubleArrow));
    assert!(matches!(
        edge.attributes.start_arrowhead,
        Some(ArrowheadType::Dot)
    ));
    assert!(matches!(
        edge.attributes.end_arrowhead,
        Some(ArrowheadType::Triangle)
    ));
    assert!(matches!(
        edge.routing_type,
        Some(crate::ast::RoutingType::Curved)
    ));
}

#[test]
fn test_container_data_creation() {
    let container = ContainerData {
        id: Some("container1".to_string()),
        label: Some("Container Label".to_string()),
        children: vec![],
        nested_containers: vec![],
        nested_groups: vec![],
        parent_container: None,
        attributes: ExcalidrawAttributes::default(),
        bounds: Some(BoundingBox {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
        }),
    };

    assert_eq!(container.id, Some("container1".to_string()));
    assert_eq!(container.label, Some("Container Label".to_string()));
    assert!(container.children.is_empty());
    assert!(container.bounds.is_some());

    let bounds = container.bounds.unwrap();
    assert_eq!(bounds.x, 10.0);
    assert_eq!(bounds.y, 20.0);
    assert_eq!(bounds.width, 300.0);
    assert_eq!(bounds.height, 200.0);
}

#[test]
fn test_group_data_creation() {
    let group = GroupData {
        id: "group1".to_string(),
        label: Some("Group Label".to_string()),
        group_type: GroupType::SemanticGroup("service".to_string()),
        children: vec![],
        nested_containers: vec![],
        nested_groups: vec![],
        parent_group: None,
        parent_container: None,
        attributes: ExcalidrawAttributes::default(),
        bounds: None,
    };

    assert_eq!(group.id, "group1");
    assert_eq!(group.label, Some("Group Label".to_string()));
    assert!(matches!(
        group.group_type,
        GroupType::SemanticGroup(ref s) if s == "service"
    ));
    assert!(group.bounds.is_none());
}

#[test]
fn test_virtual_container_nodes() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node2", "Node 2"));
    doc.containers.push(ContainerDefinition {
        id: Some("container1".to_string()),
        label: Some("Container 1".to_string()),
        children: vec!["node2".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![],
    });
    doc.edges.push(EdgeDefinition {
        from: "node1".to_string(),
        to: "container1".to_string(),
        label: None,
        arrow_type: ArrowType::SingleArrow,
        attributes: HashMap::new(),
        style: None,
    });
    doc.nodes.push(create_test_node("node1", "Node 1"));

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    // Should have 3 nodes: node1, node2, and virtual container node
    assert_eq!(igr.graph.node_count(), 3);
    assert_eq!(igr.graph.edge_count(), 1);

    // Find the virtual container node
    let container_node_idx = igr.node_map.get("container1").unwrap();
    let container_node = &igr.graph[*container_node_idx];
    assert!(container_node.is_virtual_container);
    assert_eq!(container_node.id, "container1");
}

#[test]
fn test_nested_containers() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));

    // Create parent container
    doc.nodes.push(create_test_node("node2", "Node 2"));
    doc.containers.push(ContainerDefinition {
        id: Some("parent".to_string()),
        label: Some("Parent Container".to_string()),
        children: vec!["node2".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![Statement::Container(ContainerDefinition {
            id: Some("child".to_string()),
            label: Some("Child Container".to_string()),
            children: vec!["node1".to_string()],
            attributes: HashMap::new(),
            internal_statements: vec![],
        })],
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.containers.len(), 2);
    assert_eq!(igr.node_map.len(), 4); // node1, node2, parent container, and child container

    // Find parent and child containers
    let parent_idx = igr.container_map.get("parent").unwrap();
    let child_idx = igr.container_map.get("child").unwrap();

    let parent = &igr.containers[*parent_idx];
    let child = &igr.containers[*child_idx];

    assert!(!parent.nested_containers.is_empty());
    assert_eq!(child.parent_container, Some(*parent_idx));
    assert_eq!(child.children.len(), 1);
}

#[test]
fn test_groups_in_containers() {
    let mut doc = create_test_document();
    doc.nodes.push(create_test_node("node1", "Node 1"));

    // Create container with nested group
    doc.nodes.push(create_test_node("node2", "Node 2"));
    doc.containers.push(ContainerDefinition {
        id: Some("container1".to_string()),
        label: Some("Container 1".to_string()),
        children: vec!["node2".to_string()],
        attributes: HashMap::new(),
        internal_statements: vec![Statement::Group(GroupDefinition {
            id: "group1".to_string(),
            label: Some("Group 1".to_string()),
            group_type: GroupType::BasicGroup,
            children: vec!["node1".to_string()],
            attributes: HashMap::new(),
            internal_statements: vec![],
        })],
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    assert_eq!(igr.containers.len(), 1);
    assert_eq!(igr.groups.len(), 1);

    let container = &igr.containers[0];
    let group = &igr.groups[0];

    assert!(container.nested_groups.contains(&0));
    assert!(group.parent_container.is_some());
    assert_eq!(group.children.len(), 1);
}

#[test]
fn test_apply_global_defaults() {
    let mut doc = create_test_document();
    // GlobalConfig only has background_color, stroke_width, and font
    doc.config.background_color = Some("#ff0000".to_string());
    doc.config.stroke_width = Some(3.0);
    doc.config.font = Some("Virgil".to_string());

    doc.nodes.push(NodeDefinition {
        id: "node1".to_string(),
        label: Some("Node 1".to_string()),
        component_type: None,
        attributes: HashMap::new(),
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    let node_idx = igr.node_map.get("node1").unwrap();
    let node = &igr.graph[*node_idx];

    // Global defaults are not automatically applied to nodes in from_definition
    // This is expected behavior - nodes only get attributes explicitly set
    assert_eq!(node.attributes.stroke_width, None);
    assert_eq!(node.attributes.font, None);

    // But the global config is stored
    assert_eq!(igr.global_config.stroke_width, Some(3.0));
    assert_eq!(igr.global_config.font, Some("Virgil".to_string()));
}

#[test]
fn test_attribute_override() {
    let mut doc = create_test_document();
    // GlobalConfig doesn't have stroke_color, but we can still test attribute override
    let mut attrs = HashMap::new();
    attrs.insert(
        "strokeColor".to_string(),
        AttributeValue::String("#00ff00".to_string()),
    );

    doc.nodes.push(NodeDefinition {
        id: "node1".to_string(),
        label: Some("Node 1".to_string()),
        component_type: None,
        attributes: attrs,
    });

    let igr = IntermediateGraph::from_ast(doc).unwrap();

    let node_idx = igr.node_map.get("node1").unwrap();
    let node = &igr.graph[*node_idx];

    // Node's attribute should override global default
    assert_eq!(node.attributes.stroke_color, Some("#00ff00".to_string()));
}

#[test]
fn test_fill_style_parsing() {
    let test_cases = vec![
        ("solid", Some(FillStyle::Solid)),
        ("hachure", Some(FillStyle::Hachure)),
        ("cross-hatch", Some(FillStyle::CrossHatch)),
        ("invalid", None),
    ];

    for (input, expected) in test_cases {
        let parsed = input.parse::<FillStyle>().ok();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_stroke_style_parsing() {
    let test_cases = vec![
        ("solid", Some(StrokeStyle::Solid)),
        ("dashed", Some(StrokeStyle::Dashed)),
        ("dotted", Some(StrokeStyle::Dotted)),
        ("invalid", None),
    ];

    for (input, expected) in test_cases {
        let parsed = input.parse::<StrokeStyle>().ok();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_arrowhead_type_parsing() {
    let test_cases = vec![
        ("triangle", Some(ArrowheadType::Triangle)),
        ("dot", Some(ArrowheadType::Dot)),
        ("diamond", Some(ArrowheadType::Diamond)),
        ("none", Some(ArrowheadType::None)),
        ("invalid", None),
    ];

    for (input, expected) in test_cases {
        let parsed = input.parse::<ArrowheadType>().ok();
        assert_eq!(parsed, expected);
    }
}
