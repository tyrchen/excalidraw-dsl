use excalidraw_dsl::EDSLCompiler;

#[test]
#[cfg(feature = "templates")]
fn test_template_expansion() {
    let edsl = r#"
---
layout: elk
---

template simple {
  layers {
    "Frontend" {
      components: ["Web", "Mobile"]
      layout: horizontal
    }
    "Backend" {
      components: ["API", "Database"]
      layout: horizontal
    }
  }

  connections {
    pattern: each-to-next-layer
  }
}

diagram "Simple Architecture" {
  type: architecture
  template: simple
}
    "#;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl);

    assert!(result.is_ok(), "Template compilation should succeed");
    let json = result.unwrap();

    // Should contain nodes generated from template
    assert!(json.contains("Web"));
    assert!(json.contains("Mobile"));
    assert!(json.contains("API"));
    assert!(json.contains("Database"));

    // Should contain connections
    assert!(json.contains("arrow"));
}

#[test]
fn test_diagram_without_template() {
    let edsl = r#"
diagram "Custom Diagram" {
  type: flow
}

node1[Node 1]
node2[Node 2]
node1 -> node2
    "#;

    let mut compiler = EDSLCompiler::new();
    let result = compiler.compile(edsl);

    assert!(result.is_ok(), "Diagram without template should succeed");
    let json = result.unwrap();

    assert!(json.contains("Node 1"));
    assert!(json.contains("Node 2"));
}

#[test]
#[cfg(feature = "templates")]
fn test_template_validation() {
    let edsl = r#"
template test {
  layers {
    "Layer 1" {
      components: ["A", "B"]
    }
  }
}
    "#;

    let compiler = EDSLCompiler::new();
    let result = compiler.validate(edsl);

    assert!(result.is_ok(), "Template validation should succeed");
}
