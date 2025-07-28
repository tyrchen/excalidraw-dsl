// src/lib.rs

pub mod ast;
pub mod error;
pub mod fluent;
pub mod generator;
pub mod igr;
pub mod layout;
pub mod parser;
pub mod presets;
#[cfg(feature = "routing")]
pub mod routing;

#[cfg(feature = "templates")]
pub mod template;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "ml-layout")]
pub mod training;

#[cfg(test)]
mod tests;

pub use error::{EDSLError, Result};
pub use fluent::DiagramBuilder;
pub use presets::{DiagramPresets, ThemePresets};

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
    /// Whether to validate output after generation
    #[allow(dead_code)]
    validate_output: bool,
    /// Whether to use parallel processing for layout calculations
    #[allow(dead_code)]
    parallel_layout: bool,
    /// Maximum number of threads for parallel operations
    #[allow(dead_code)]
    max_threads: Option<usize>,
}

/// Builder for creating customized EDSLCompiler instances
pub struct EDSLCompilerBuilder {
    layout_manager: Option<LayoutManager>,
    #[cfg(feature = "llm")]
    llm_api_key: Option<String>,
    validate_output: bool,
    parallel_layout: bool,
    max_threads: Option<usize>,
    cache_enabled: bool,
}

impl Default for EDSLCompilerBuilder {
    fn default() -> Self {
        Self {
            layout_manager: None,
            #[cfg(feature = "llm")]
            llm_api_key: None,
            validate_output: false,
            parallel_layout: true,
            max_threads: None,
            cache_enabled: true,
        }
    }
}

impl EDSLCompilerBuilder {
    /// Create a new compiler builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom layout manager
    pub fn with_layout_manager(mut self, manager: LayoutManager) -> Self {
        self.layout_manager = Some(manager);
        self
    }

    /// Enable output validation
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.validate_output = enabled;
        self
    }

    /// Enable parallel layout processing
    pub fn with_parallel_layout(mut self, enabled: bool) -> Self {
        self.parallel_layout = enabled;
        self
    }

    /// Set maximum threads for parallel operations
    pub fn with_max_threads(mut self, threads: usize) -> Self {
        self.max_threads = Some(threads);
        self
    }

    /// Enable or disable layout caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Enable LLM optimization with API key
    #[cfg(feature = "llm")]
    pub fn with_llm_optimization(mut self, api_key: String) -> Self {
        self.llm_api_key = Some(api_key);
        self
    }

    /// Build the EDSLCompiler instance
    pub fn build(self) -> EDSLCompiler {
        let mut layout_manager = self.layout_manager.unwrap_or_default();
        layout_manager.enable_cache(self.cache_enabled);

        EDSLCompiler {
            layout_manager,
            #[cfg(feature = "llm")]
            llm_optimizer: self.llm_api_key.map(llm::LLMLayoutOptimizer::new),
            validate_output: self.validate_output,
            parallel_layout: self.parallel_layout,
            max_threads: self.max_threads,
        }
    }
}

impl EDSLCompiler {
    /// Create a new EDSL compiler with default settings
    pub fn new() -> Self {
        EDSLCompilerBuilder::new().build()
    }

    /// Create a new compiler builder for customization
    pub fn builder() -> EDSLCompilerBuilder {
        EDSLCompilerBuilder::new()
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
    ///
    /// # Deprecated
    /// Use `EDSLCompiler::builder().with_llm_optimization(api_key).build()` instead
    #[cfg(feature = "llm")]
    #[deprecated(note = "Use EDSLCompiler::builder().with_llm_optimization() instead")]
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
mod lib_tests {
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
