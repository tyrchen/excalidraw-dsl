use excalidraw_dsl::{EDSLCompiler, Result};
use serde_json::Value;

/// Helper function to compile EDSL with ELK layout and return JSON
fn compile_with_elk(edsl: &str, algorithm: &str) -> Result<Value> {
    let mut compiler = EDSLCompiler::new();

    // Create EDSL with ELK layout configuration
    let full_edsl = format!(
        r#"---
layout: elk
elk_algorithm: {algorithm}
---

{edsl}"#
    );

    let json_output = compiler.compile(&full_edsl)?;
    Ok(serde_json::from_str(&json_output)?)
}

#[test]
fn test_elk_layered_layout() {
    let edsl = r#"
start[Start]
process1[Process 1]
process2[Process 2]
decision[Decision]
end[End]

start -> process1
process1 -> decision
decision -> process2: Yes
decision -> end: No
process2 -> end
"#;

    let result = compile_with_elk(edsl, "layered");
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 5 nodes + 5 text elements + 5 edges = 15 elements
    assert_eq!(elements.len(), 15);

    // Verify layered layout properties
    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" || e["type"] == "diamond")
        .collect();

    // In layered layout, nodes should be arranged in layers
    // Start should be leftmost, End should be rightmost
    let start_node = nodes
        .iter()
        .find(|n| {
            elements
                .iter()
                .any(|e| e["type"] == "text" && e["text"] == "Start" && e["containerId"] == n["id"])
        })
        .expect("Should find start node");

    let end_node = nodes
        .iter()
        .find(|n| {
            elements
                .iter()
                .any(|e| e["type"] == "text" && e["text"] == "End" && e["containerId"] == n["id"])
        })
        .expect("Should find end node");

    // Verify start is to the left of end
    assert!(start_node["x"].as_f64().unwrap() < end_node["x"].as_f64().unwrap());
}

#[test]
fn test_elk_stress_layout() {
    let edsl = r#"
a[A]
b[B]
c[C]
d[D]
e[E]

a -> b
a -> c
b -> d
c -> d
d -> e
"#;

    let result = compile_with_elk(edsl, "stress");
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 5 nodes + 5 text elements + 5 edges = 15 elements
    assert_eq!(elements.len(), 15);

    // Stress layout should minimize edge lengths
    // Verify all nodes have positions
    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle")
        .collect();

    for node in nodes {
        assert!(node["x"].is_number());
        assert!(node["y"].is_number());
        // Node positions are top-left corner, so they can be negative if width/height > 2*position
        // Just check they're reasonable values
        let x = node["x"].as_f64().unwrap();
        let y = node["y"].as_f64().unwrap();
        assert!(
            x > -1000.0 && x < 1000.0,
            "x position {x} is out of reasonable range"
        );
        assert!(
            y > -1000.0 && y < 1000.0,
            "y position {y} is out of reasonable range"
        );
    }
}

#[test]
fn test_elk_force_layout() {
    let edsl = r#"
center[Center]
node1[Node 1]
node2[Node 2]
node3[Node 3]
node4[Node 4]

center -> node1
center -> node2
center -> node3
center -> node4
"#;

    let result = compile_with_elk(edsl, "force");
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 5 nodes + 5 text elements + 4 edges = 14 elements
    assert_eq!(elements.len(), 14);

    // Force layout should spread nodes around the center
    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle")
        .collect();

    // Find center node
    let center_node = nodes
        .iter()
        .find(|n| {
            elements.iter().any(|e| {
                e["type"] == "text" && e["text"] == "Center" && e["containerId"] == n["id"]
            })
        })
        .expect("Should find center node");

    let _center_x = center_node["x"].as_f64().unwrap();
    let _center_y = center_node["y"].as_f64().unwrap();

    // Other nodes should be distributed around center
    let peripheral_nodes: Vec<&Value> = nodes
        .iter()
        .filter(|n| n["id"] != center_node["id"])
        .copied()
        .collect();

    assert_eq!(peripheral_nodes.len(), 4);

    // Check that nodes are distributed (not all in same position)
    let mut x_positions: Vec<f64> = peripheral_nodes
        .iter()
        .map(|n| n["x"].as_f64().unwrap())
        .collect();
    x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let x_spread = x_positions.last().unwrap() - x_positions.first().unwrap();

    // Also check y spread to ensure nodes are distributed in 2D
    let mut y_positions: Vec<f64> = peripheral_nodes
        .iter()
        .map(|n| n["y"].as_f64().unwrap())
        .collect();
    y_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let y_spread = y_positions.last().unwrap() - y_positions.first().unwrap();

    let total_spread = x_spread.max(y_spread);
    assert!(
        total_spread > 50.0,
        "Nodes should be spread out, but x_spread={x_spread}, y_spread={y_spread}"
    ); // Nodes should be spread out in at least one dimension
}

#[test]
fn test_elk_tree_layout() {
    let edsl = r#"
root[Root]
child1[Child 1]
child2[Child 2]
leaf1[Leaf 1]
leaf2[Leaf 2]
leaf3[Leaf 3]

root -> child1
root -> child2
child1 -> leaf1
child1 -> leaf2
child2 -> leaf3
"#;

    let result = compile_with_elk(edsl, "tree");
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 6 nodes + 6 text elements + 5 edges = 17 elements
    assert_eq!(elements.len(), 17);

    // Tree layout should arrange nodes hierarchically
    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle")
        .collect();

    // Find root node
    let root_node = nodes
        .iter()
        .find(|n| {
            elements
                .iter()
                .any(|e| e["type"] == "text" && e["text"] == "Root" && e["containerId"] == n["id"])
        })
        .expect("Should find root node");

    // Find leaf nodes
    let leaf_nodes: Vec<&Value> = nodes
        .iter()
        .filter(|n| {
            elements.iter().any(|e| {
                e["type"] == "text"
                    && e["text"].as_str().unwrap().starts_with("Leaf")
                    && e["containerId"] == n["id"]
            })
        })
        .copied()
        .collect();

    assert_eq!(leaf_nodes.len(), 3);

    // In tree layout, check that the tree has a hierarchical structure
    // Since the tree algorithm can position nodes in various ways, let's check
    // that root is to the left of leaves (smaller x coordinate)
    let root_x = root_node["x"].as_f64().unwrap();
    for leaf in leaf_nodes {
        let leaf_x = leaf["x"].as_f64().unwrap();
        // Root should be to the left of leaves in a left-to-right tree
        assert!(
            root_x < leaf_x,
            "Root x={root_x} should be to the left of leaf x={leaf_x}"
        );
    }
}

#[test]
fn test_elk_layout_with_containers() {
    let edsl = r#"
container "System" {
    api[API]
    service[Service]

    api -> service
}

user[User]
database[Database]

user -> api
service -> database
"#;

    let result = compile_with_elk(edsl, "layered");
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Should have nodes, edges, text elements, and container
    assert!(!elements.is_empty());

    // Verify container exists
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    assert!(!containers.is_empty(), "Should have at least one container");
}

#[test]
fn test_elk_layout_options() {
    let edsl = r#"
---
layout: elk
elk_algorithm: layered
node_spacing: 100
layer_spacing: 150
---

a[A]
b[B]
c[C]

a -> b
b -> c
"#;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl);
    assert!(result.is_ok());

    let json: Value = serde_json::from_str(&result.unwrap()).unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 3 nodes + 3 text elements + 2 edges = 8 elements
    assert_eq!(elements.len(), 8);

    // With larger spacing, nodes should be further apart
    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle")
        .collect();

    assert_eq!(nodes.len(), 3);

    // Check that spacing is respected (approximate check)
    let mut x_positions: Vec<f64> = nodes.iter().map(|n| n["x"].as_f64().unwrap()).collect();
    x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if x_positions[0] != x_positions[1] && x_positions[1] != x_positions[2] {
        // If nodes are in different layers, check layer spacing
        let spacing1 = (x_positions[1] - x_positions[0]).abs();
        let spacing2 = (x_positions[2] - x_positions[1]).abs();

        // With layer_spacing of 150, actual spacing should be at least 100
        assert!(spacing1 > 100.0 || spacing2 > 100.0);
    }
}

#[test]
fn test_elk_layout_with_edge_routing() {
    let edsl = r#"
---
layout: elk
elk_algorithm: layered
edge_routing: orthogonal
---

start[Start]
middle[Middle]
end[End]

start -> middle
middle -> end
start -> end
"#;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl);
    assert!(result.is_ok());

    // Just verify it compiles successfully with edge routing option
    let json: Value = serde_json::from_str(&result.unwrap()).unwrap();
    let elements = json["elements"].as_array().unwrap();

    // 3 nodes + 3 text elements + 3 edges = 9 elements
    assert_eq!(elements.len(), 9);
}
