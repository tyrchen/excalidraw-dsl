// tests/integration_tests.rs
use excalidraw_dsl::{EDSLCompiler, Result};
use serde_json::Value;

/// Helper function to compile EDSL and return JSON
fn compile_to_json(edsl: &str) -> Result<Value> {
    let compiler = EDSLCompiler::new();
    let json_output = compiler.compile(edsl)?;
    Ok(serde_json::from_str(&json_output)?)
}

/// Helper to count elements by type
fn count_elements_by_type(elements: &Value, element_type: &str) -> usize {
    elements
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == element_type)
        .count()
}

#[test]
fn test_simple_nodes_and_edge() {
    let edsl = r##"
a[Node A]
b[Node B]
a -> b
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 3);
    assert_eq!(count_elements_by_type(&result, "rectangle"), 2);
    assert_eq!(count_elements_by_type(&result, "arrow"), 1);
}

#[test]
fn test_nodes_with_styles() {
    let edsl = r##"
---
layout: dagre
---

start[Start] {
  strokeColor: "#22c55e";
  backgroundColor: "#dcfce7";
}

end[End] {
  strokeColor: "#ef4444";
  backgroundColor: "#fee2e2";
}

start -> end
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 3);

    // Check first node has correct colors
    let start_node = &elements[0];
    assert_eq!(start_node["strokeColor"], "#22c55e");
    assert_eq!(start_node["backgroundColor"], "#dcfce7");

    // Check second node has correct colors
    let end_node = &elements[1];
    assert_eq!(end_node["strokeColor"], "#ef4444");
    assert_eq!(end_node["backgroundColor"], "#fee2e2");
}

#[test]
fn test_edge_labels_with_braces() {
    let edsl = r##"
question[Question]
yes[Yes]
no[No]

question -> yes{Yes Path}
question -> no{No Path}
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 5); // 3 nodes + 2 edges
    assert_eq!(count_elements_by_type(&result, "rectangle"), 3);
    assert_eq!(count_elements_by_type(&result, "arrow"), 2);
}

#[test]
fn test_edge_labels_with_colon() {
    let edsl = r##"
source[Source]
target[Target]

source -> target: Label Text
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 3); // 2 nodes + 1 edge
}

#[test]
fn test_complex_flow_diagram() {
    let edsl = r##"
---
layout: dagre
---

# Process flow
input[User Input] {
  backgroundColor: "#dbeafe";
}

validate[Validate] {
  backgroundColor: "#fef3c7";
}

process[Process] {
  backgroundColor: "#f3e8ff";
}

success[Success] {
  backgroundColor: "#dcfce7";
}

error[Error] {
  backgroundColor: "#fee2e2";
}

# Connections
input -> validate
validate -> process{Valid}
validate -> error{Invalid}
process -> success
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 9); // 5 nodes + 4 edges
    assert_eq!(count_elements_by_type(&result, "rectangle"), 5);
    assert_eq!(count_elements_by_type(&result, "arrow"), 4);
}

#[test]
fn test_all_arrow_types() {
    let edsl = r##"
a[A]
b[B]
c[C]
d[D]

a -> b
b -- c
c <-> d
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 7); // 4 nodes + 3 edges

    // Check arrow types
    let arrows: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();

    // The line type (--) generates type "line", not "arrow"
    let lines: Vec<&Value> = elements.iter().filter(|e| e["type"] == "line").collect();

    assert_eq!(arrows.len(), 2); // Single arrow and double arrow
    assert_eq!(lines.len(), 1); // Line type

    // First arrow (a -> b) should have end arrowhead
    assert!(arrows[0]["end_arrowhead"].is_string());

    // Line (b -- c) should not have arrowheads
    assert!(lines[0]["end_arrowhead"].is_null());

    // Second arrow (c <-> d) should have both arrowheads
    assert!(arrows[1]["end_arrowhead"].is_string());
    assert!(arrows[1]["start_arrowhead"].is_string());
}

#[test]
fn test_node_without_label() {
    let edsl = r##"
node1
node2[With Label]

node1 -> node2
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 3);

    // First node should have id as text
    assert_eq!(elements[0]["text"], "node1");

    // Second node should have label as text
    assert_eq!(elements[1]["text"], "With Label");
}

#[test]
fn test_yaml_frontmatter() {
    let edsl = r##"
---
layout: dagre
direction: LR
---

a[A]
b[B]
a -> b
"##;

    // Should compile without errors
    let result = compile_to_json(edsl);
    assert!(result.is_ok());
}

#[test]
fn test_comments_ignored() {
    let edsl = r##"
# This is a comment
a[Node A] # Inline comment
# Another comment
b[Node B]

# Edge comment
a -> b
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 3); // Comments should be ignored
}

#[test]
fn test_multiline_edsl() {
    let edsl = r##"
start[Start]
middle[Middle]
end[End]

start -> middle
middle -> end
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 5); // 3 nodes + 2 edges
}

#[test]
fn test_empty_edsl() {
    let edsl = "";

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 0);
}

#[test]
fn test_only_comments() {
    let edsl = r##"
# Just comments
# Nothing else
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();

    assert_eq!(elements.len(), 0);
}

// Error cases

#[test]
fn test_undefined_node_reference_error() {
    let edsl = r##"
a[Node A]
a -> b  # b is not defined
"##;

    let result = compile_to_json(edsl);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unknown node referenced"));
}

#[test]
fn test_invalid_color_syntax_error() {
    let edsl = r##"
node[Node] {
  backgroundColor: #ffffff;  # Missing quotes
}
"##;

    let result = compile_to_json(edsl);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("Parse error"));
}

#[test]
fn test_cyclic_graph_error() {
    let edsl = r##"
a[A]
b[B]
c[C]

a -> b
b -> c
c -> a  # Creates a cycle
"##;

    let result = compile_to_json(edsl);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("Layout error"));
}

#[test]
fn test_excalidraw_element_properties() {
    let edsl = r##"
node[Test Node]
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();
    let node = &elements[0];

    // Check required Excalidraw properties
    assert!(node["id"].is_string());
    assert!(node["type"].is_string());
    assert!(node["x"].is_number());
    assert!(node["y"].is_number());
    assert!(node["width"].is_number());
    assert!(node["height"].is_number());
    assert!(node["angle"].is_number());
    assert!(node["strokeColor"].is_string());
    assert!(node["backgroundColor"].is_string());
    assert!(node["fillStyle"].is_string());
    assert!(node["strokeWidth"].is_number());
    assert!(node["strokeStyle"].is_string());
    assert!(node["roughness"].is_number());
    assert!(node["opacity"].is_number());
    assert!(node["text"].is_string());
    assert!(node["fontSize"].is_number());
    assert!(node["fontFamily"].is_number());
}

#[test]
fn test_edge_bindings() {
    let edsl = r##"
a[A]
b[B]
a -> b
"##;

    let result = compile_to_json(edsl).unwrap();
    let elements = result.as_array().unwrap();
    let edge = &elements[2]; // Third element should be the edge

    // Check edge has proper bindings
    assert!(edge["start_binding"].is_object());
    assert!(edge["end_binding"].is_object());

    let start_binding = &edge["start_binding"];
    assert!(start_binding["elementId"].is_string());
    assert!(start_binding["gap"].is_number());
    assert!(start_binding["focus"].is_number());

    let end_binding = &edge["end_binding"];
    assert!(end_binding["elementId"].is_string());
    assert!(end_binding["gap"].is_number());
    assert!(end_binding["focus"].is_number());
}

// Performance tests

#[test]
fn test_large_graph_performance() {
    use std::time::Instant;

    // Generate a large graph with 100 nodes
    let mut edsl = String::new();
    for i in 0..100 {
        edsl.push_str(&format!("node{}[Node {}]\n", i, i));
    }

    // Connect them in a chain
    for i in 0..99 {
        edsl.push_str(&format!("node{} -> node{}\n", i, i + 1));
    }

    let start = Instant::now();
    let result = compile_to_json(&edsl);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
        duration.as_secs() < 5,
        "Large graph took too long to compile: {:?}",
        duration
    );

    let elements = result.unwrap();
    let arr = elements.as_array().unwrap();
    assert_eq!(arr.len(), 199); // 100 nodes + 99 edges
}
