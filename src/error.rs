// src/error.rs

use std::path::PathBuf;
use thiserror::Error;

/// Extended error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// File path where the error occurred (if applicable)
    pub file_path: Option<PathBuf>,
    /// Line number where the error occurred
    pub line: Option<usize>,
    /// Column number where the error occurred
    pub column: Option<usize>,
    /// The problematic source code snippet
    pub source_snippet: Option<String>,
    /// Additional context information
    pub additional_context: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file_path: None,
            line: None,
            column: None,
            source_snippet: None,
            additional_context: Vec::new(),
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.source_snippet = Some(snippet);
        self
    }

    pub fn add_context(mut self, context: String) -> Self {
        self.additional_context.push(context);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
pub enum EDSLError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Build error: {0}")]
    Build(#[from] BuildError),

    #[error("Layout error: {0}")]
    Layout(#[from] LayoutError),

    #[error("Generator error: {0}")]
    Generator(#[from] GeneratorError),

    #[cfg(feature = "llm")]
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

impl EDSLError {
    /// Add context to any error type
    pub fn with_context<F>(self, f: F) -> ContextualError
    where
        F: FnOnce() -> ErrorContext,
    {
        ContextualError {
            inner: self,
            context: f(),
        }
    }
}

/// An error with additional context information
#[derive(Error, Debug)]
#[error("{inner}\nContext: {}", format_context(context))]
pub struct ContextualError {
    #[source]
    pub inner: EDSLError,
    pub context: ErrorContext,
}

fn format_context(ctx: &ErrorContext) -> String {
    let mut parts = Vec::new();

    if let Some(ref path) = ctx.file_path {
        parts.push(format!("File: {}", path.display()));
    }

    if let Some(line) = ctx.line {
        if let Some(col) = ctx.column {
            parts.push(format!("Location: line {line}, column {col}"));
        } else {
            parts.push(format!("Line: {line}"));
        }
    }

    if let Some(ref snippet) = ctx.source_snippet {
        parts.push(format!("Source: {snippet}"));
    }

    for ctx_info in &ctx.additional_context {
        parts.push(ctx_info.clone());
    }

    parts.join("\n  ")
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("Invalid YAML configuration: {0}")]
    InvalidConfig(String),

    #[error("Pest parsing failed: {0}")]
    PestError(#[from] Box<pest::error::Error<crate::parser::Rule>>),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Unknown node referenced: {0}")]
    UnknownNode(String),

    #[error("Circular container dependency")]
    CircularDependency,

    #[error("Invalid attribute value for {attribute}: {value}")]
    InvalidAttribute { attribute: String, value: String },

    #[error("Duplicate node ID: {0}")]
    DuplicateNode(String),

    #[error("Empty container: {0}")]
    EmptyContainer(String),

    #[error("Unknown component type: {0}")]
    UnknownComponentType(String),
}

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Unknown layout engine: {0}")]
    UnknownEngine(String),

    #[error("Layout calculation failed: {0}")]
    CalculationFailed(String),

    #[error("Invalid graph structure for layout")]
    InvalidGraph,

    #[error("Layout timeout")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Invalid element type: {0}")]
    InvalidElementType(String),

    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),

    #[error("Invalid coordinate: {x}, {y}")]
    InvalidCoordinate { x: f64, y: f64 },

    #[error("Element generation failed: {0}")]
    GenerationFailed(String),
}

#[cfg(feature = "llm")]
#[derive(Error, Debug)]
pub enum LLMError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("API quota exceeded")]
    QuotaExceeded,

    #[error("LLM service unavailable")]
    ServiceUnavailable,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Request timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, EDSLError>;

/// Error recovery strategies for different error types
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Try an alternative approach
    Alternative(String),
    /// Use a default value
    Default(String),
    /// Skip the problematic element
    Skip,
    /// Retry with modified parameters
    Retry { suggestion: String },
    /// Manual intervention required
    Manual { instructions: String },
}

/// Trait for errors that can suggest recovery strategies
pub trait RecoverableError {
    /// Get suggested recovery strategies for this error
    fn recovery_strategies(&self) -> Vec<RecoveryStrategy>;
}

impl RecoverableError for ParseError {
    fn recovery_strategies(&self) -> Vec<RecoveryStrategy> {
        match self {
            ParseError::Syntax { .. } => vec![
                RecoveryStrategy::Skip,
                RecoveryStrategy::Manual {
                    instructions: "Check syntax and ensure proper formatting".to_string(),
                },
            ],
            ParseError::InvalidConfig(_) => vec![
                RecoveryStrategy::Default("Use default configuration".to_string()),
                RecoveryStrategy::Manual {
                    instructions: "Verify YAML configuration format".to_string(),
                },
            ],
            _ => vec![RecoveryStrategy::Skip],
        }
    }
}

impl RecoverableError for LayoutError {
    fn recovery_strategies(&self) -> Vec<RecoveryStrategy> {
        match self {
            LayoutError::UnknownEngine(_engine) => vec![
                RecoveryStrategy::Alternative("Use 'dagre' as fallback layout".to_string()),
                RecoveryStrategy::Default("dagre".to_string()),
            ],
            LayoutError::CalculationFailed(_) => vec![
                RecoveryStrategy::Retry {
                    suggestion: "Try with simplified graph structure".to_string(),
                },
                RecoveryStrategy::Alternative("Use manual positioning".to_string()),
            ],
            LayoutError::Timeout => vec![
                RecoveryStrategy::Retry {
                    suggestion: "Increase timeout or simplify graph".to_string(),
                },
                RecoveryStrategy::Alternative("Use cached layout if available".to_string()),
            ],
            _ => vec![],
        }
    }
}

impl RecoverableError for GeneratorError {
    fn recovery_strategies(&self) -> Vec<RecoveryStrategy> {
        match self {
            GeneratorError::InvalidElementType(t) => vec![
                RecoveryStrategy::Alternative(format!("Use 'rectangle' instead of '{t}'")),
                RecoveryStrategy::Skip,
            ],
            GeneratorError::MissingAttribute(attr) => vec![
                RecoveryStrategy::Default(format!("Use default value for '{attr}'")),
                RecoveryStrategy::Manual {
                    instructions: format!("Provide required attribute '{attr}'"),
                },
            ],
            _ => vec![],
        }
    }
}
