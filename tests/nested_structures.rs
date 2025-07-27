use excalidraw_dsl::{EDSLCompiler, Result};
use serde_json::Value;

/// Helper function to compile EDSL and return JSON
fn compile_to_json(edsl: &str) -> Result<Value> {
    let mut compiler = EDSLCompiler::new();
    let json_output = compiler.compile(edsl)?;
    Ok(serde_json::from_str(&json_output)?)
}

#[test]
fn test_nested_containers() {
    let edsl = r#"
container "Outer System" {
    outer_node[Outer Node]

    container "Inner System" {
        inner_node1[Inner Node 1]
        inner_node2[Inner Node 2]

        inner_node1 -> inner_node2
    }

    outer_node -> inner_node1
}

external[External]
external -> outer_node
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Count different element types
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    let nodes: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] != true)
        .collect();

    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();

    // Should have 2 containers (outer and inner)
    assert_eq!(containers.len(), 2);

    // Should have 4 nodes (outer_node, inner_node1, inner_node2, external)
    assert_eq!(nodes.len(), 4);

    // Should have 3 edges
    assert_eq!(edges.len(), 3);

    // Verify container labels
    let container_labels: Vec<&str> = elements
        .iter()
        .filter(|e| e["type"] == "text" && e["text"].as_str().unwrap().contains("System"))
        .map(|e| e["text"].as_str().unwrap())
        .collect();

    assert!(container_labels.contains(&"Outer System"));
    assert!(container_labels.contains(&"Inner System"));
}

#[test]
#[ignore = "Nested containers not yet implemented"]
fn test_deeply_nested_containers() {
    let edsl = r#"
container "Level 1" {
    node1[Node 1]

    container "Level 2" {
        node2[Node 2]

        container "Level 3" {
            node3[Node 3]

            container "Level 4" {
                node4[Node 4]
            }
        }
    }
}

node1 -> node2
node2 -> node3
node3 -> node4
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Count containers
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    // Should have 4 containers (one for each level)
    assert_eq!(containers.len(), 4);

    // Verify all level labels exist
    for i in 1..=4 {
        let level_label = format!("Level {i}");
        let found = elements
            .iter()
            .any(|e| e["type"] == "text" && e["text"] == level_label);
        assert!(found, "Should find label for {level_label}");
    }
}

#[test]
#[ignore = "Nested groups not yet implemented"]
fn test_nested_groups() {
    let edsl = r#"
group "Outer Group" {
    outer1[Outer 1]

    group "Inner Group" {
        inner1[Inner 1]
        inner2[Inner 2]

        inner1 -> inner2
    }

    outer1 -> inner1
}

external[External]
external -> outer1
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Groups should be represented as containers
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    // Should have 2 groups represented as containers
    assert_eq!(containers.len(), 2);

    // Verify group labels
    let group_labels: Vec<&str> = elements
        .iter()
        .filter(|e| e["type"] == "text" && e["text"].as_str().unwrap().contains("Group"))
        .map(|e| e["text"].as_str().unwrap())
        .collect();

    assert!(group_labels.contains(&"Outer Group"));
    assert!(group_labels.contains(&"Inner Group"));
}

#[test]
#[ignore = "Mixed nested structures not yet implemented"]
fn test_mixed_containers_and_groups() {
    let edsl = r#"
container "System Container" {
    sys_node[System Node]

    group "Feature Group" {
        feat1[Feature 1]
        feat2[Feature 2]

        container "Sub Container" {
            sub1[Sub 1]
            sub2[Sub 2]

            sub1 -> sub2
        }

        feat1 -> sub1
    }

    sys_node -> feat1
}

group "External Group" {
    ext1[External 1]
    ext2[External 2]

    ext1 -> ext2
}

sys_node -> ext1
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Count all containers (both container and group types)
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    // Should have 4 containers total (1 container + 2 groups + 1 sub-container)
    assert_eq!(containers.len(), 4);

    // Verify all nodes exist
    let node_labels = vec![
        "System Node",
        "Feature 1",
        "Feature 2",
        "Sub 1",
        "Sub 2",
        "External 1",
        "External 2",
    ];

    for label in node_labels {
        let found = elements
            .iter()
            .any(|e| e["type"] == "text" && e["text"] == label);
        assert!(found, "Should find node with label: {label}");
    }
}

#[test]
#[ignore = "Flow groups not yet implemented"]
fn test_flow_groups() {
    let edsl = r#"
flow "User Flow" {
    start[Start]
    middle[Middle]
    end[End]

    start -> middle
    middle -> end
}

flow "Admin Flow" {
    admin_start[Admin Start]
    admin_action[Admin Action]
    admin_end[Admin End]

    admin_start -> admin_action
    admin_action -> admin_end
}

start -> admin_start: "Switch to Admin"
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Flow groups should be represented as containers
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    // Should have 2 flow groups
    assert_eq!(containers.len(), 2);

    // Verify flow labels
    let flow_labels: Vec<&str> = elements
        .iter()
        .filter(|e| e["type"] == "text" && e["text"].as_str().unwrap().contains("Flow"))
        .map(|e| e["text"].as_str().unwrap())
        .collect();

    assert!(flow_labels.contains(&"User Flow"));
    assert!(flow_labels.contains(&"Admin Flow"));
}

#[test]
#[ignore = "Semantic groups not yet implemented"]
fn test_semantic_groups() {
    let edsl = r#"
service "API Service" {
    endpoint1[GET /users]
    endpoint2[POST /users]
    endpoint3[DELETE /users/:id]

    endpoint1 -> endpoint2: "Create after list"
}

layer "Data Layer" {
    db[Database]
    cache[Cache]

    cache -> db: "Cache miss"
}

endpoint2 -> db: "Store user"
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Semantic groups should be represented as containers
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    // Should have 2 semantic groups
    assert_eq!(containers.len(), 2);

    // Verify semantic group labels
    let group_labels: Vec<&str> = elements
        .iter()
        .filter(|e| {
            e["type"] == "text"
                && (e["text"].as_str().unwrap().contains("Service")
                    || e["text"].as_str().unwrap().contains("Layer"))
        })
        .map(|e| e["text"].as_str().unwrap())
        .collect();

    assert!(group_labels.contains(&"API Service"));
    assert!(group_labels.contains(&"Data Layer"));
}

#[test]
#[ignore = "Containers with internal edges not yet implemented"]
fn test_container_with_internal_edges_only() {
    let edsl = r#"
container "Isolated System" {
    node1[Node 1]
    node2[Node 2]
    node3[Node 3]

    node1 -> node2
    node2 -> node3
    node3 -> node1
}

other[Other Node]
"#;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Verify all edges are internal to container
    let edges: Vec<&Value> = elements.iter().filter(|e| e["type"] == "arrow").collect();

    assert_eq!(edges.len(), 3);

    // All nodes should exist
    let node_count = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] != true)
        .count();

    assert_eq!(node_count, 4); // 3 internal + 1 external
}

#[test]
#[ignore = "Empty nested containers not yet implemented"]
fn test_empty_nested_containers() {
    let edsl = r#"
container "Outer" {
    node[Node]

    container "Empty Inner" {
        # No nodes here
    }
}
"#;

    let result = compile_to_json(edsl);
    // Empty containers should cause an error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Empty container"));
}

#[test]
#[ignore = "Container and group styling not yet implemented"]
fn test_container_and_group_styling() {
    let edsl = r##"
container "Styled Container" style: {
    strokeColor: "#ff0000";
    backgroundColor: "#ffeeee";
} {
    node1[Node 1]
}

group "Styled Group" style: {
    strokeColor: "#00ff00";
    backgroundColor: "#eeffee";
} {
    node2[Node 2]
}
"##;

    let result = compile_to_json(edsl);
    assert!(result.is_ok());

    let json = result.unwrap();
    let elements = json["elements"].as_array().unwrap();

    // Find styled containers
    let containers: Vec<&Value> = elements
        .iter()
        .filter(|e| e["type"] == "rectangle" && e["isContainer"] == true)
        .collect();

    assert_eq!(containers.len(), 2);

    // Verify at least one container has red stroke
    let red_container = containers.iter().find(|c| c["strokeColor"] == "#ff0000");
    assert!(
        red_container.is_some(),
        "Should find container with red stroke"
    );

    // Verify at least one container has green stroke
    let green_container = containers.iter().find(|c| c["strokeColor"] == "#00ff00");
    assert!(
        green_container.is_some(),
        "Should find container with green stroke"
    );
}
