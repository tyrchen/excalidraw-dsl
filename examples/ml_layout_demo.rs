//! ML Layout Demo - Demonstrates ML-enhanced layout functionality
//!
//! Run with: cargo run --example ml_layout_demo --features ml-layout

use excalidraw_dsl::{ast::*, error::Result, igr::IntermediateGraph, layout::LayoutManager};
use petgraph::visit::IntoNodeReferences;
use std::collections::HashMap;

fn main() -> Result<()> {
    env_logger::init();

    println!("=== ML Layout Demo ===\n");

    // Create different types of graphs to test ML layout selection
    let graphs = vec![
        ("Simple Linear", create_simple_linear_graph()),
        ("Complex Network", create_complex_network()),
        ("Hierarchical", create_hierarchical_graph()),
        ("Dense Graph", create_dense_graph()),
    ];

    let layout_manager = LayoutManager::new();

    for (name, mut igr) in graphs {
        println!("Testing ML layout on: {name}");

        // Set layout to ML
        igr.global_config.layout = Some("ml".to_string());

        // Apply ML-enhanced layout
        match layout_manager.layout(&mut igr) {
            Ok(_) => {
                println!("  ✓ ML layout succeeded");
                print_graph_info(&igr);
            }
            Err(e) => {
                println!("  ✗ ML layout failed: {e}");
                // Try with standard layout as fallback
                igr.global_config.layout = Some("dagre".to_string());
                layout_manager.layout(&mut igr)?;
                println!("  ✓ Fallback to dagre succeeded");
            }
        }
        println!();
    }

    Ok(())
}

fn create_simple_linear_graph() -> IntermediateGraph {
    let document = ParsedDocument {
        config: GlobalConfig::default(),
        component_types: HashMap::new(),
        templates: HashMap::new(),
        diagram: None,
        nodes: vec![
            NodeDefinition {
                id: "start".to_string(),
                label: Some("Start".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "middle".to_string(),
                label: Some("Process".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "end".to_string(),
                label: Some("End".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
        ],
        edges: vec![
            EdgeDefinition {
                from: "start".to_string(),
                to: "middle".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "middle".to_string(),
                to: "end".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
        ],
        containers: vec![],
        groups: vec![],
        connections: vec![],
    };

    IntermediateGraph::from_ast(document).unwrap()
}

fn create_complex_network() -> IntermediateGraph {
    let document = ParsedDocument {
        config: GlobalConfig::default(),
        component_types: HashMap::new(),
        templates: HashMap::new(),
        diagram: None,
        nodes: vec![
            NodeDefinition {
                id: "hub".to_string(),
                label: Some("Central Hub".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "node1".to_string(),
                label: Some("Node 1".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "node2".to_string(),
                label: Some("Node 2".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "node3".to_string(),
                label: Some("Node 3".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "node4".to_string(),
                label: Some("Node 4".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
        ],
        edges: vec![
            EdgeDefinition {
                from: "hub".to_string(),
                to: "node1".to_string(),
                label: None,
                arrow_type: ArrowType::DoubleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "hub".to_string(),
                to: "node2".to_string(),
                label: None,
                arrow_type: ArrowType::DoubleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "hub".to_string(),
                to: "node3".to_string(),
                label: None,
                arrow_type: ArrowType::DoubleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "hub".to_string(),
                to: "node4".to_string(),
                label: None,
                arrow_type: ArrowType::DoubleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "node1".to_string(),
                to: "node2".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "node3".to_string(),
                to: "node4".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
        ],
        containers: vec![],
        groups: vec![],
        connections: vec![],
    };

    IntermediateGraph::from_ast(document).unwrap()
}

fn create_hierarchical_graph() -> IntermediateGraph {
    let document = ParsedDocument {
        config: GlobalConfig::default(),
        component_types: HashMap::new(),
        templates: HashMap::new(),
        diagram: None,
        nodes: vec![
            NodeDefinition {
                id: "ceo".to_string(),
                label: Some("CEO".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "cto".to_string(),
                label: Some("CTO".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "cfo".to_string(),
                label: Some("CFO".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "dev1".to_string(),
                label: Some("Dev 1".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "dev2".to_string(),
                label: Some("Dev 2".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
            NodeDefinition {
                id: "acc1".to_string(),
                label: Some("Accountant".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            },
        ],
        edges: vec![
            EdgeDefinition {
                from: "ceo".to_string(),
                to: "cto".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "ceo".to_string(),
                to: "cfo".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "cto".to_string(),
                to: "dev1".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "cto".to_string(),
                to: "dev2".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
            EdgeDefinition {
                from: "cfo".to_string(),
                to: "acc1".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            },
        ],
        containers: vec![
            ContainerDefinition {
                id: Some("tech".to_string()),
                label: Some("Tech Department".to_string()),
                children: vec!["cto".to_string(), "dev1".to_string(), "dev2".to_string()],
                attributes: HashMap::new(),
                internal_statements: vec![],
            },
            ContainerDefinition {
                id: Some("finance".to_string()),
                label: Some("Finance Department".to_string()),
                children: vec!["cfo".to_string(), "acc1".to_string()],
                attributes: HashMap::new(),
                internal_statements: vec![],
            },
        ],
        groups: vec![],
        connections: vec![],
    };

    IntermediateGraph::from_ast(document).unwrap()
}

fn create_dense_graph() -> IntermediateGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create a dense graph with many interconnections
    for i in 0..6 {
        nodes.push(NodeDefinition {
            id: format!("n{i}"),
            label: Some(format!("Node {i}")),
            component_type: None,
            attributes: HashMap::new(),
        });
    }

    // Create many edges (almost fully connected)
    for i in 0..5 {
        for j in (i + 1)..6 {
            if i * j % 3 != 0 {
                // Skip some edges to avoid complete graph
                edges.push(EdgeDefinition {
                    from: format!("n{i}"),
                    to: format!("n{j}"),
                    label: None,
                    arrow_type: if i * j % 2 == 0 {
                        ArrowType::DoubleArrow
                    } else {
                        ArrowType::SingleArrow
                    },
                    attributes: HashMap::new(),
                    style: None,
                });
            }
        }
    }

    let document = ParsedDocument {
        config: GlobalConfig::default(),
        component_types: HashMap::new(),
        templates: HashMap::new(),
        diagram: None,
        nodes,
        edges,
        containers: vec![],
        groups: vec![],
        connections: vec![],
    };

    IntermediateGraph::from_ast(document).unwrap()
}

fn print_graph_info(igr: &IntermediateGraph) {
    println!("  - Nodes: {}", igr.graph.node_count());
    println!("  - Edges: {}", igr.graph.edge_count());
    println!("  - Containers: {}", igr.containers.len());
    println!("  - Groups: {}", igr.groups.len());

    // Sample a few node positions
    let mut positions = Vec::new();
    for (i, (_, node)) in igr.graph.node_references().enumerate() {
        if i < 3 {
            positions.push(format!("{}: ({:.1}, {:.1})", node.id, node.x, node.y));
        }
    }
    println!("  - Sample positions: {}", positions.join(", "));
}
