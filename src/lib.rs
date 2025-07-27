// src/lib.rs
#![allow(clippy::large_enum_variant)]
#![allow(clippy::result_large_err)]

pub mod ast;
pub mod error;
pub mod generator;
pub mod igr;
pub mod layout;
pub mod parser;
#[cfg(feature = "routing")]
pub mod routing;

#[cfg(feature = "templates")]
pub mod template;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "server")]
pub mod server;

pub use error::{EDSLError, Result};

use crate::generator::ExcalidrawGenerator;
use crate::igr::IntermediateGraph;
use crate::layout::LayoutManager;
use crate::parser::parse_edsl;

#[cfg(feature = "templates")]
use crate::template::TemplateProcessor;

/// The main EDSL compiler that orchestrates parsing, layout, and generation
pub struct EDSLCompiler {
    layout_manager: LayoutManager,
    #[cfg(feature = "llm")]
    llm_optimizer: Option<llm::LLMLayoutOptimizer>,
}

impl EDSLCompiler {
    /// Create a new EDSL compiler with default settings
    pub fn new() -> Self {
        Self {
            layout_manager: LayoutManager::new(),
            #[cfg(feature = "llm")]
            llm_optimizer: None,
        }
    }

    /// Process templates if the feature is enabled
    fn process_templates(
        &self,
        parsed_doc: crate::ast::ParsedDocument,
    ) -> Result<crate::ast::ParsedDocument> {
        #[cfg(feature = "templates")]
        {
            let template_processor = TemplateProcessor::new();
            template_processor.process_document(parsed_doc)
        }

        #[cfg(not(feature = "templates"))]
        {
            Ok(parsed_doc)
        }
    }

    /// Enable LLM layout optimization with the provided API key
    #[cfg(feature = "llm")]
    pub fn with_llm_optimization(mut self, api_key: String) -> Self {
        self.llm_optimizer = Some(llm::LLMLayoutOptimizer::new(api_key));
        self
    }

    /// Compile EDSL source code to Excalidraw JSON
    pub fn compile(&mut self, edsl_source: &str) -> Result<String> {
        // Parse EDSL
        let parsed_doc = parse_edsl(edsl_source)?;

        // Process templates if present
        let processed_doc = self.process_templates(parsed_doc)?;

        // Build intermediate graph representation
        let mut igr = IntermediateGraph::from_ast(processed_doc)?;

        // Apply layout algorithms
        self.layout_manager.layout(&mut igr)?;

        // Apply LLM optimization if enabled
        #[cfg(feature = "llm")]
        if let Some(optimizer) = &mut self.llm_optimizer {
            optimizer.optimize_layout(&mut igr, edsl_source)?;
        }

        // Generate Excalidraw file
        let file = ExcalidrawGenerator::generate_file(&igr)?;

        // Serialize to JSON
        serde_json::to_string_pretty(&file).map_err(EDSLError::Json)
    }

    /// Compile EDSL source code and return raw elements (without JSON serialization)
    pub fn compile_to_elements(
        &mut self,
        edsl_source: &str,
    ) -> Result<Vec<generator::ExcalidrawElementSkeleton>> {
        let parsed_doc = parse_edsl(edsl_source)?;
        let processed_doc = self.process_templates(parsed_doc)?;
        let mut igr = IntermediateGraph::from_ast(processed_doc)?;

        self.layout_manager.layout(&mut igr)?;

        #[cfg(feature = "llm")]
        if let Some(optimizer) = &mut self.llm_optimizer {
            optimizer.optimize_layout(&mut igr, edsl_source)?;
        }

        ExcalidrawGenerator::generate(&igr)
    }

    /// Parse and validate EDSL source code without generating output
    pub fn validate(&self, edsl_source: &str) -> Result<()> {
        let parsed_doc = parse_edsl(edsl_source)?;
        let processed_doc = self.process_templates(parsed_doc)?;
        let _igr = IntermediateGraph::from_ast(processed_doc)?;
        Ok(())
    }

    /// Validate Excalidraw JSON file format
    pub fn validate_excalidraw(&self, json_content: &str) -> Result<()> {
        use serde_json::Value;

        let value: Value = serde_json::from_str(json_content).map_err(EDSLError::Json)?;

        // Check if it's an object (native Excalidraw format)
        match &value {
            Value::Object(map) => {
                // Check for type field
                if map.get("type").and_then(|v| v.as_str()) == Some("excalidraw") {
                    // Validate elements array
                    if let Some(Value::Array(elements)) = map.get("elements") {
                        for (i, element) in elements.iter().enumerate() {
                            Self::validate_excalidraw_element(element, i)?;
                        }
                    } else {
                        return Err(EDSLError::Validation {
                            message: "Missing 'elements' array in Excalidraw format".into(),
                        });
                    }
                } else {
                    return Err(EDSLError::Validation {
                        message: "Invalid Excalidraw format: missing or incorrect 'type' field"
                            .into(),
                    });
                }
            }
            _ => {
                return Err(EDSLError::Validation {
                    message: "Invalid Excalidraw format: expected object with type 'excalidraw'"
                        .into(),
                });
            }
        }

        Ok(())
    }

    fn validate_excalidraw_element(element: &serde_json::Value, index: usize) -> Result<()> {
        let obj = element.as_object().ok_or_else(|| EDSLError::Validation {
            message: format!("Element {index} is not an object"),
        })?;

        // Required fields
        let required_fields = ["type", "id", "x", "y"];
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(EDSLError::Validation {
                    message: format!("Element {index} missing required field '{field}'"),
                });
            }
        }

        // Validate element type
        if let Some(type_val) = obj.get("type").and_then(|v| v.as_str()) {
            match type_val {
                "rectangle" | "ellipse" | "diamond" | "arrow" | "line" | "text" => {}
                _ => {
                    return Err(EDSLError::Validation {
                        message: format!("Element {index} has invalid type '{type_val}'"),
                    })
                }
            }
        }

        // Validate numeric fields
        let numeric_fields = [
            "x",
            "y",
            "width",
            "height",
            "angle",
            "strokeWidth",
            "opacity",
            "fontSize",
        ];
        for field in &numeric_fields {
            if let Some(val) = obj.get(*field) {
                if !val.is_number() {
                    return Err(EDSLError::Validation {
                        message: format!("Element {index} field '{field}' must be a number"),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the intermediate graph representation for debugging/inspection
    pub fn get_igr(&self, edsl_source: &str) -> Result<IntermediateGraph> {
        let parsed_doc = parse_edsl(edsl_source)?;
        let processed_doc = self.process_templates(parsed_doc)?;
        let mut igr = IntermediateGraph::from_ast(processed_doc)?;
        self.layout_manager.layout(&mut igr)?;
        Ok(igr)
    }
}

impl Default for EDSLCompiler {
    fn default() -> Self {
        Self::new()
    }
}

// Additional integration tests are located in tests/ directory

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::visit::IntoNodeReferences;

    #[test]
    fn test_basic_compilation() {
        let edsl = r#"
---
layout: dagre
---

user[User]
api[API Gateway]
db[Database] {
  shape: cylinder;
}

user -> api -> db
        "#;

        let mut compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl);

        if let Err(e) = &result {
            eprintln!("Compilation error: {e:?}");
        }
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("rectangle")); // User and API Gateway nodes
        assert!(json.contains("ellipse")); // Database node (cylinder approximated as ellipse)
        assert!(json.contains("arrow")); // Edges
    }

    #[test]
    fn test_container_compilation() {
        let edsl = r#"
container "Backend Services" {
  api[API Gateway]
  user_service[User Service]
  api -> user_service;
}
        "#;

        let mut compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Backend Services"));
    }

    #[test]
    fn test_validation() {
        let edsl =
            "web_server[Web Server] {\n    shape: \"rectangle\";\n    strokeColor: \"#ff0000\";\n}";

        let compiler = EDSLCompiler::new();
        let result = compiler.validate(edsl);

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_edsl() {
        let edsl = r#"
        node_a -> nonexistent_node
        "#;

        let mut compiler = EDSLCompiler::new();
        let result = compiler.compile(edsl);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_igr() {
        let edsl = r#"
        a[Node A]
        b[Node B]
        a -> b
        "#;

        let compiler = EDSLCompiler::new();
        let igr = compiler.get_igr(edsl).unwrap();

        assert_eq!(igr.graph.node_count(), 2);
        assert_eq!(igr.graph.edge_count(), 1);

        // Check that layout has been applied (nodes have positions)
        for (_, node) in igr.graph.node_references() {
            assert!(node.x != 0.0 || node.y != 0.0); // At least one node should be positioned
        }
    }
}
