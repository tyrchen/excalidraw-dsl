// tests/simple_integration_test.rs
use excalidraw_dsl::EDSLCompiler;

#[test]
fn test_basic_compilation() {
    let compiler = EDSLCompiler::new();
    
    // Test simple nodes and edge
    let edsl = "a[Node A]\nb[Node B]\na -> b";
    let result = compiler.compile(edsl);
    assert!(result.is_ok());
    
    // Test with edge labels
    let edsl_with_labels = "start[Start]\nend[End]\nstart -> end{Success}";
    let result = compiler.compile(edsl_with_labels);
    assert!(result.is_ok());
}

#[test]
fn test_error_handling() {
    let compiler = EDSLCompiler::new();
    
    // Test undefined node reference
    let edsl = "a[Node A]\na -> b";
    let result = compiler.compile(edsl);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown node referenced"));
}

#[test]
fn test_compile_to_elements() {
    let compiler = EDSLCompiler::new();
    
    let edsl = "a[A]\nb[B]\na -> b";
    let result = compiler.compile_to_elements(edsl);
    assert!(result.is_ok());
    
    let elements = result.unwrap();
    assert_eq!(elements.len(), 3); // 2 nodes + 1 edge
}