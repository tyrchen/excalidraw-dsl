use excalidraw_dsl::EDSLCompiler;
use serde_json::Value;

#[test]
fn test_text_color_end_to_end() {
    let edsl_content = r##"
---
layout: elk
direction: TB
---

# Test text color end-to-end

dark_node[Dark Background] {
  backgroundColor: "#1e293b";
  strokeColor: "#0f172a";
  fontSize: 20;
  color: "#ffffff";
}

light_node[Light Background] {
  backgroundColor: "#fef3c7";
  strokeColor: "#f59e0b";
  fontSize: 20;
  color: "#7c2d12";
}

default_node[Default Text Color] {
  backgroundColor: "#7dd3fc";
  fontSize: 20;
}

container "Test Container" as test_container {
  style: {
    backgroundColor: "#000000";
    strokeColor: "#333333";
  }

  contained_node[Container Node] {
    backgroundColor: "#1a1a1a";
    fontSize: 18;
    color: "#00ff00";
  }
}

dark_node -> light_node: "Connection";
light_node -> default_node;
"##;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl_content);
    assert!(result.is_ok(), "Compilation should succeed");

    let json_output = result.unwrap();
    let parsed: Value = serde_json::from_str(&json_output).expect("Should parse as JSON");

    // Verify it's a valid Excalidraw file
    assert_eq!(parsed["type"], "excalidraw");
    assert_eq!(parsed["version"], 2);

    let elements = parsed["elements"]
        .as_array()
        .expect("Should have elements array");
    assert!(!elements.is_empty());

    // Find and verify text elements have correct colors
    let text_elements: Vec<&Value> = elements.iter().filter(|e| e["type"] == "text").collect();

    // Find dark node text element
    let dark_text = text_elements
        .iter()
        .find(|e| e["text"] == "Dark Background")
        .expect("Should find dark node text");
    assert_eq!(
        dark_text["strokeColor"], "#ffffff",
        "Dark node text should be white"
    );

    // Find light node text element
    let light_text = text_elements
        .iter()
        .find(|e| e["text"] == "Light Background")
        .expect("Should find light node text");
    assert_eq!(
        light_text["strokeColor"], "#7c2d12",
        "Light node text should be dark brown"
    );

    // Find default node text element
    let default_text = text_elements
        .iter()
        .find(|e| e["text"] == "Default Text Color")
        .expect("Should find default node text");
    assert_eq!(
        default_text["strokeColor"], "#000000",
        "Default node text should be black"
    );

    // Find container node text element
    let container_node_text = text_elements
        .iter()
        .find(|e| e["text"] == "Container Node")
        .expect("Should find container node text");
    assert_eq!(
        container_node_text["strokeColor"], "#00ff00",
        "Container node text should be green"
    );
}

#[test]
fn test_container_text_color_inheritance() {
    let edsl_content = r##"
---
layout: elk
---

container "Colored Container" as colored_container {
  style: {
    backgroundColor: "#1e293b";
    strokeColor: "#0f172a";
    color: "#ffffff";
  }

  node1[Node 1] {
    backgroundColor: "#2d3748";
  }
}

container "Default Container" as default_container {
  style: {
    backgroundColor: "#f0f0f0";
  }

  node2[Node 2] {
    backgroundColor: "#e0e0e0";
  }
}
"##;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl_content);
    assert!(result.is_ok(), "Compilation should succeed");

    let json_output = result.unwrap();
    let parsed: Value = serde_json::from_str(&json_output).expect("Should parse as JSON");

    let elements = parsed["elements"]
        .as_array()
        .expect("Should have elements array");

    // Find container label text elements
    let text_elements: Vec<&Value> = elements.iter().filter(|e| e["type"] == "text").collect();

    // Verify colored container label has white text
    let colored_container_text = text_elements
        .iter()
        .find(|e| e["text"] == "Colored Container")
        .expect("Should find colored container text");
    assert_eq!(
        colored_container_text["strokeColor"], "#ffffff",
        "Colored container text should be white"
    );

    // Verify default container label has black text
    let default_container_text = text_elements
        .iter()
        .find(|e| e["text"] == "Default Container")
        .expect("Should find default container text");
    assert_eq!(
        default_container_text["strokeColor"], "#000000",
        "Default container text should be black"
    );
}

#[test]
#[ignore = "Group attributes syntax needs investigation"]
fn test_group_text_color() {
    let edsl_content = r##"
---
layout: elk
---

group colored_group {
  label: "Colored Group";
  type: service;
  backgroundColor: "#1e293b";
  strokeColor: "#0f172a";
  color: "#ffff00";

  node1[Node 1] {
    backgroundColor: "#7dd3fc";
  }
}
"##;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl_content);
    assert!(result.is_ok(), "Compilation should succeed");

    let json_output = result.unwrap();
    let parsed: Value = serde_json::from_str(&json_output).expect("Should parse as JSON");

    let elements = parsed["elements"]
        .as_array()
        .expect("Should have elements array");

    // Find group label text element
    let group_text = elements
        .iter()
        .find(|e| e["type"] == "text" && e["text"] == "Colored Group")
        .expect("Should find group text");

    assert_eq!(
        group_text["strokeColor"], "#ffff00",
        "Group text should be yellow"
    );
}

#[test]
fn test_edge_label_colors_remain_default() {
    let edsl_content = r##"
---
layout: elk
---

node1[Node 1] {
  color: "#ff0000";
}

node2[Node 2] {
  color: "#00ff00";
}

node1 -> node2: "Edge Label";
"##;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl_content);
    assert!(result.is_ok(), "Compilation should succeed");

    let json_output = result.unwrap();
    let parsed: Value = serde_json::from_str(&json_output).expect("Should parse as JSON");

    let elements = parsed["elements"]
        .as_array()
        .expect("Should have elements array");

    // Verify node texts have their specified colors
    let node1_text = elements
        .iter()
        .find(|e| e["type"] == "text" && e["text"] == "Node 1")
        .expect("Should find Node 1 text");
    assert_eq!(node1_text["strokeColor"], "#ff0000");

    let node2_text = elements
        .iter()
        .find(|e| e["type"] == "text" && e["text"] == "Node 2")
        .expect("Should find Node 2 text");
    assert_eq!(node2_text["strokeColor"], "#00ff00");

    // Edge labels should remain black (arrows with text don't use the node color)
    let edge = elements
        .iter()
        .find(|e| e["type"] == "arrow" && e["text"] == "Edge Label")
        .expect("Should find edge with label");
    assert_eq!(
        edge["strokeColor"], "#000000",
        "Edge text should remain black"
    );
}
