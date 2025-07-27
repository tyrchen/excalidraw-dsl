use excalidraw_dsl::{EDSLCompiler, Result};
use serde_json::Value;

/// Helper function to compile EDSL and return JSON
fn compile_to_json(edsl: &str) -> Result<Value> {
    let mut compiler = EDSLCompiler::new();
    let json_output = compiler.compile(edsl)?;
    Ok(serde_json::from_str(&json_output)?)
}

#[test]
fn test_connection_routing_types() {
    let edsl = r#"
a[Node A]
b[Node B]
c[Node C]
d[Node D]

connection {
    from: "a";
    to: "b";
    style {
        type: arrow;
        routing: orthogonal;
    }
}

connection {
    from: "c";
    to: "d";
    style {
        type: arrow;
        routing: curved;
    }
}
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Find edges and check their points
    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();
    assert_eq!(edges.len(), 2);

    // Check that orthogonal routing has more than 2 points
    let orthogonal_edge = edges
        .iter()
        .find(|e| e["text"] == "Orthogonal Route" || e["points"].as_array().unwrap().len() > 2);
    assert!(orthogonal_edge.is_some());

    // Check that curved routing has 3 points
    let curved_edge = edges
        .iter()
        .find(|e| e["text"] == "Curved Route" || e["points"].as_array().unwrap().len() == 3);
    assert!(curved_edge.is_some());
}

#[test]
fn test_connections_with_multiple_targets() {
    let edsl = r#"
hub[Hub]
node1[Node 1]
node2[Node 2]
node3[Node 3]

connections {
    from: "hub";
    to: ["node1", "node2", "node3"];
    style {
        type: arrow;
        routing: auto;
    }
}
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Should create 3 edges
    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();
    assert_eq!(edges.len(), 3);

    // Each edge should have points based on auto routing
    for edge in edges {
        let points = edge["points"].as_array().unwrap();
        assert!(points.len() >= 2); // At least start and end points
    }
}

#[test]
fn test_routing_with_straight_line() {
    let edsl = r#"
start[Start]
end[End]

connection {
    from: "start";
    to: "end";
    style {
        type: arrow;
        routing: straight;
    }
}
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();
    assert_eq!(edges.len(), 1);

    // Straight routing should have exactly 2 points
    let points = edges[0]["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
}

#[test]
fn test_default_routing() {
    let edsl = r#"
a[A]
b[B]

# Regular edge without routing specification
a -> b
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();
    assert_eq!(edges.len(), 1);

    // Default routing uses auto which might produce 2 or more points
    let points = edges[0]["points"].as_array().unwrap();
    assert!(points.len() >= 2); // At least start and end points
}
