#[cfg(feature = "llm")]
mod llm_tests {
    use excalidraw_dsl::EDSLCompiler;

    // Test basic LLM functionality through the public API

    #[test]
    #[ignore = "Requires valid OpenAI API key"]
    fn test_llm_enabled_compiler() {
        // Test that we can create a compiler with LLM support
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test_api_key".to_string());
        let mut compiler = EDSLCompiler::builder()
            .with_llm_optimization(api_key)
            .build();

        // The compiler should work normally even with LLM enabled
        let edsl = "node[Test Node]";
        let result = compiler.compile(edsl);
        if let Err(e) = &result {
            eprintln!("Compilation failed: {e:?}");
        }
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires valid OpenAI API key"]
    fn test_llm_optimization_with_complex_diagram() {
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test_api_key".to_string());
        let mut compiler = EDSLCompiler::builder()
            .with_llm_optimization(api_key)
            .build();

        let edsl = r#"
---
layout: dagre
---

user[User]
api[API Gateway]
service[Service]
database[Database]

user -> api
api -> service
service -> database
"#;

        // This would make an actual LLM call in real usage
        // For testing, we just verify it compiles without error
        let result = compiler.compile(edsl);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires valid OpenAI API key"]
    fn test_llm_with_containers() {
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test_api_key".to_string());
        let mut compiler = EDSLCompiler::builder()
            .with_llm_optimization(api_key)
            .build();

        let edsl = r#"
container "System" {
    api[API]
    service[Service]

    api -> service
}

user[User]
user -> api
"#;

        let result = compiler.compile(edsl);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires valid OpenAI API key"]
    fn test_llm_with_groups() {
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test_api_key".to_string());
        let mut compiler = EDSLCompiler::builder()
            .with_llm_optimization(api_key)
            .build();

        let edsl = r#"
group "Feature Group" {
    feat1[Feature 1]
    feat2[Feature 2]

    feat1 -> feat2
}

user[User]
user -> feat1
"#;

        let result = compiler.compile(edsl);
        assert!(result.is_ok());
    }

    // Integration test would require mocking the LLM API
    // For now, we'll skip actual optimization tests that would make API calls
}
