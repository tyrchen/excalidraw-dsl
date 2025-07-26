#[cfg(test)]
mod integration_tests {
    use crate::EDSLCompiler;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_complete_pipeline_simple() {
        let edsl_content = "
---
layout: dagre
---

# Simple test
node1[Node 1] {
  strokeColor: \"#ff0000\";
  backgroundColor: \"#ffeeee\";
}

node2[Node 2] {
  shape: \"ellipse\";
  strokeColor: \"#00ff00\";
}

node1 -> node2: Connection
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        // Verify basic structure
        assert_eq!(parsed["type"], "excalidraw");
        assert_eq!(parsed["version"], 2);
        assert!(parsed["elements"].is_array());
        
        let elements = parsed["elements"].as_array().unwrap();
        // Should have 2 nodes + 2 text elements + 1 edge = 5 elements
        assert_eq!(elements.len(), 5);
        
        // Check that we have the right element types
        let element_types: Vec<&str> = elements
            .iter()
            .map(|e| e["type"].as_str().unwrap())
            .collect();
        
        assert!(element_types.contains(&"rectangle"));
        assert!(element_types.contains(&"ellipse"));
        assert!(element_types.contains(&"arrow"));
        assert!(element_types.iter().filter(|&&t| t == "text").count() == 2);
    }

    #[test]
    fn test_complete_pipeline_with_styles() {
        let edsl_content = "
---
layout: force
iterations: 100
---

start[Start] {
  shape: \"rectangle\";
  strokeColor: \"#22c55e\";
  backgroundColor: \"#dcfce7\";
  roughness: 0;
  font: \"Code\";
  fontSize: 24;
}

end[End] {
  shape: \"ellipse\";
  strokeColor: \"#ef4444\";
  backgroundColor: \"#fee2e2\";
  fill: \"hachure\";
}

start -> end: Flow
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        let elements = parsed["elements"].as_array().unwrap();
        
        // Find the rectangle element
        let rect_element = elements
            .iter()
            .find(|e| e["type"] == "rectangle")
            .unwrap();
        
        assert_eq!(rect_element["strokeColor"], "#22c55e");
        assert_eq!(rect_element["backgroundColor"], "#dcfce7");
        assert_eq!(rect_element["roughness"], 0);
        assert_eq!(rect_element["fontFamily"], 3); // Cascadia/Code
        assert_eq!(rect_element["fontSize"], 24);
        
        // Find the ellipse element
        let ellipse_element = elements
            .iter()
            .find(|e| e["type"] == "ellipse")
            .unwrap();
        
        assert_eq!(ellipse_element["strokeColor"], "#ef4444");
        assert_eq!(ellipse_element["backgroundColor"], "#fee2e2");
        assert_eq!(ellipse_element["fillStyle"], "hachure");
    }

    #[test]
    fn test_edge_with_label() {
        let edsl_content = "
---
layout: dagre
---

a[Node A]
b[Node B]

a -> b: Test Label
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        let elements = parsed["elements"].as_array().unwrap();
        
        // Find the arrow element
        let arrow_element = elements
            .iter()
            .find(|e| e["type"] == "arrow")
            .unwrap();
        
        assert_eq!(arrow_element["text"], "Test Label");
        assert!(arrow_element["startBinding"].is_object());
        assert!(arrow_element["endBinding"].is_object());
        assert_eq!(arrow_element["endArrowhead"], "arrow");
    }

    #[test]
    fn test_invalid_edsl_syntax() {
        let edsl_content = "node1[Invalid Node] -> {{{{{ invalid syntax";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(matches!(error, crate::error::EDSLError::Parse(_)));
    }

    #[test]
    fn test_unknown_node_reference() {
        let edsl_content = "
---
layout: dagre
---

node1[Node 1]
# Reference unknown node
node1 -> unknown_node: Connection
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(matches!(error, crate::error::EDSLError::Build(_)));
    }

    #[test]
    fn test_cycle_detection_with_dagre() {
        let edsl_content = "
---
layout: dagre
---

a[Node A]
b[Node B]
c[Node C]

a -> b
b -> c
c -> a  # Creates a cycle
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(matches!(error, crate::error::EDSLError::Layout(_)));
        
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("cycle"));
        assert!(error_msg.contains("dagre"));
        assert!(error_msg.contains("force"));
    }

    #[test]
    fn test_force_layout_with_cycles() {
        let edsl_content = "
---
layout: force
iterations: 50
---

a[Node A]
b[Node B]
c[Node C]

a -> b
b -> c
c -> a  # Creates a cycle - should work with force layout
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        let elements = parsed["elements"].as_array().unwrap();
        
        // Should have 3 nodes + 3 text elements + 3 edges = 9 elements
        assert_eq!(elements.len(), 9);
        
        // Verify all elements have valid coordinates
        for element in elements {
            if let Some(x) = element.get("x") {
                assert!(x.is_number());
                let x_val = x.as_i64().unwrap();
                assert!(x_val.abs() < 10000); // Reasonable coordinate range
            }
            if let Some(y) = element.get("y") {
                assert!(y.is_number());
                let y_val = y.as_i64().unwrap();
                assert!(y_val.abs() < 10000); // Reasonable coordinate range
            }
        }
    }

    #[test]
    fn test_file_io_integration() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.edsl");
        let output_path = dir.path().join("test.excalidraw");

        let edsl_content = "
---
layout: dagre
---

test[Test Node] {
  strokeColor: \"#blue\";
  backgroundColor: \"#lightblue\";
}

test2[Test Node 2]
test -> test2: Connection
";

        // Write input file
        fs::write(&input_path, edsl_content).unwrap();

        // Compile using the CLI-like interface
        let compiler = EDSLCompiler::new();
        let input_content = fs::read_to_string(&input_path).unwrap();
        let output_json = compiler.compile(&input_content).unwrap();
        
        // Write output file
        fs::write(&output_path, &output_json).unwrap();

        // Verify output file exists and is valid JSON
        let output_content = fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
        
        assert_eq!(parsed["type"], "excalidraw");
        assert!(parsed["elements"].is_array());
        
        // Clean up
        drop(dir);
    }

    #[test]
    fn test_text_sizing_integration() {
        let edsl_content = "
---
layout: dagre
---

short[Hi]
medium[Medium Text]
long[This is a very long text that should make the node wider]

short -> medium -> long
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        let elements = parsed["elements"].as_array().unwrap();
        
        // Find rectangle elements and their widths
        let rectangles: Vec<_> = elements
            .iter()
            .filter(|e| e["type"] == "rectangle")
            .collect();
        
        assert_eq!(rectangles.len(), 3);
        
        // Get widths
        let widths: Vec<i64> = rectangles
            .iter()
            .map(|e| e["width"].as_i64().unwrap())
            .collect();
        
        // Verify that longer text results in wider nodes
        // The exact widths depend on the text measurement algorithm
        assert!(widths[0] < widths[2]); // "Hi" < "This is a very long text..."
        assert!(widths[1] < widths[2]); // "Medium Text" < "This is a very long text..."
    }

    #[test]
    fn test_container_basic_functionality() {
        let edsl_content = "
---
layout: dagre
---

# Test container with internal nodes
container \"Test Container\" as test_container {
  style: {
    backgroundColor: \"#f0f0f0\";
    strokeStyle: dashed;
  }
  
  internal1[Internal Node 1]
  internal2[Internal Node 2]
  
  internal1 -> internal2: Internal Connection
}

external[External Node]
external -> internal1: External Connection
";

        let compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl_content);
        
        assert!(result.is_ok());
        let excalidraw_json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&excalidraw_json).unwrap();
        
        let elements = parsed["elements"].as_array().unwrap();
        
        // Should have container + nodes + text elements + edges
        assert!(elements.len() > 5);
        
        // Verify we have both rectangle elements (for nodes and containers)
        let rectangles = elements
            .iter()
            .filter(|e| e["type"] == "rectangle")
            .count();
        
        assert!(rectangles >= 3); // At least container + 2 internal nodes + 1 external
    }
}