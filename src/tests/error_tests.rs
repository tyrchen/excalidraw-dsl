// src/tests/error_tests.rs

use crate::error::{BuildError, EDSLError, GeneratorError, LayoutError, ParseError, Result};
use std::io;

#[cfg(feature = "llm")]
use crate::error::LLMError;

#[test]
fn test_parse_error_display() {
    let err = ParseError::Syntax {
        line: 42,
        message: "Unexpected token".to_string(),
    };
    assert_eq!(err.to_string(), "Syntax error at line 42: Unexpected token");

    let err = ParseError::InvalidConfig("Missing layout field".to_string());
    assert_eq!(
        err.to_string(),
        "Invalid YAML configuration: Missing layout field"
    );

    let err = ParseError::ValidationError("Invalid node ID".to_string());
    assert_eq!(err.to_string(), "Validation error: Invalid node ID");
}

#[test]
fn test_build_error_display() {
    let err = BuildError::UnknownNode("node123".to_string());
    assert_eq!(err.to_string(), "Unknown node referenced: node123");

    let err = BuildError::CircularDependency;
    assert_eq!(err.to_string(), "Circular container dependency");

    let err = BuildError::InvalidAttribute {
        attribute: "strokeWidth".to_string(),
        value: "25".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Invalid attribute value for strokeWidth: 25"
    );

    let err = BuildError::DuplicateNode("node1".to_string());
    assert_eq!(err.to_string(), "Duplicate node ID: node1");

    let err = BuildError::EmptyContainer("container1".to_string());
    assert_eq!(err.to_string(), "Empty container: container1");

    let err = BuildError::UnknownComponentType("custom_type".to_string());
    assert_eq!(err.to_string(), "Unknown component type: custom_type");
}

#[test]
fn test_layout_error_display() {
    let err = LayoutError::UnknownEngine("custom".to_string());
    assert_eq!(err.to_string(), "Unknown layout engine: custom");

    let err = LayoutError::CalculationFailed("Division by zero".to_string());
    assert_eq!(
        err.to_string(),
        "Layout calculation failed: Division by zero"
    );

    let err = LayoutError::InvalidGraph;
    assert_eq!(err.to_string(), "Invalid graph structure for layout");

    let err = LayoutError::Timeout;
    assert_eq!(err.to_string(), "Layout timeout");
}

#[test]
fn test_generator_error_display() {
    let err = GeneratorError::InvalidElementType("custom_shape".to_string());
    assert_eq!(err.to_string(), "Invalid element type: custom_shape");

    let err = GeneratorError::MissingAttribute("label".to_string());
    assert_eq!(err.to_string(), "Missing required attribute: label");

    let err = GeneratorError::InvalidCoordinate {
        x: f64::NAN,
        y: 100.0,
    };
    assert_eq!(err.to_string(), "Invalid coordinate: NaN, 100");

    let err = GeneratorError::GenerationFailed("Out of memory".to_string());
    assert_eq!(err.to_string(), "Element generation failed: Out of memory");
}

#[cfg(feature = "llm")]
#[test]
fn test_llm_error_display() {
    let err = LLMError::InvalidResponse("Empty response".to_string());
    assert_eq!(err.to_string(), "Invalid API response: Empty response");

    let err = LLMError::QuotaExceeded;
    assert_eq!(err.to_string(), "API quota exceeded");

    let err = LLMError::ServiceUnavailable;
    assert_eq!(err.to_string(), "LLM service unavailable");

    let err = LLMError::AuthenticationFailed;
    assert_eq!(err.to_string(), "Authentication failed");

    let err = LLMError::Timeout;
    assert_eq!(err.to_string(), "Request timeout");
}

#[test]
fn test_edsl_error_conversion() {
    // Test ParseError conversion
    let parse_err = ParseError::Syntax {
        line: 10,
        message: "Test".to_string(),
    };
    let edsl_err: EDSLError = parse_err.into();
    assert!(matches!(edsl_err, EDSLError::Parse(_)));

    // Test BuildError conversion
    let build_err = BuildError::UnknownNode("test".to_string());
    let edsl_err: EDSLError = build_err.into();
    assert!(matches!(edsl_err, EDSLError::Build(_)));

    // Test LayoutError conversion
    let layout_err = LayoutError::InvalidGraph;
    let edsl_err: EDSLError = layout_err.into();
    assert!(matches!(edsl_err, EDSLError::Layout(_)));

    // Test GeneratorError conversion
    let gen_err = GeneratorError::InvalidElementType("test".to_string());
    let edsl_err: EDSLError = gen_err.into();
    assert!(matches!(edsl_err, EDSLError::Generator(_)));
}

#[test]
fn test_edsl_error_display() {
    let err = EDSLError::Validation {
        message: "Test validation error".to_string(),
    };
    assert_eq!(err.to_string(), "Validation error: Test validation error");

    let err = EDSLError::Configuration {
        message: "Invalid configuration".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Configuration error: Invalid configuration"
    );
}

#[test]
fn test_io_error_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let edsl_err: EDSLError = io_err.into();
    assert!(matches!(edsl_err, EDSLError::Io(_)));
}

#[test]
fn test_json_error_conversion() {
    let json_str = "{ invalid json }";
    let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let edsl_err: EDSLError = json_err.into();
    assert!(matches!(edsl_err, EDSLError::Json(_)));
}

#[test]
fn test_yaml_error_conversion() {
    let yaml_str = "invalid: \n  - yaml: structure:";
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(yaml_str).unwrap_err();
    let edsl_err: EDSLError = yaml_err.into();
    assert!(matches!(edsl_err, EDSLError::Yaml(_)));
}

#[test]
fn test_error_result_type() {
    // Test that Result type alias works correctly
    fn test_function() -> Result<String> {
        Ok("Success".to_string())
    }

    assert!(test_function().is_ok());

    fn test_error_function() -> Result<String> {
        Err(EDSLError::Validation {
            message: "Test error".to_string(),
        })
    }

    assert!(test_error_function().is_err());
}

#[test]
fn test_parse_error_from_pest() {
    // This test would require creating a pest error, which is complex
    // Just verify the structure exists
    let err = ParseError::ValidationError("Test".to_string());
    assert!(err.to_string().contains("Validation error"));
}

#[test]
fn test_error_chain() {
    // Test that errors can be chained through ? operator
    fn inner_function() -> Result<String> {
        Err(BuildError::UnknownNode("test".to_string()).into())
    }

    fn outer_function() -> Result<String> {
        inner_function()?;
        Ok("Never reached".to_string())
    }

    let result = outer_function();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EDSLError::Build(_)));
}

#[test]
fn test_custom_error_messages() {
    // Test that custom error messages are preserved
    let custom_message = "This is a detailed error explanation with context";
    let err = EDSLError::Validation {
        message: custom_message.to_string(),
    };

    assert!(err.to_string().contains(custom_message));
}

#[test]
fn test_error_debug_format() {
    let err = BuildError::CircularDependency;
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("CircularDependency"));
}

#[test]
fn test_invalid_coordinate_formatting() {
    let err = GeneratorError::InvalidCoordinate {
        x: f64::INFINITY,
        y: -f64::INFINITY,
    };
    let err_str = err.to_string();
    assert!(err_str.contains("inf"));
    assert!(err_str.contains("-inf"));
}
