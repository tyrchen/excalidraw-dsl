// src/error.rs
use thiserror::Error;

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

    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("Invalid YAML configuration: {0}")]
    InvalidConfig(String),

    #[error("Pest parsing failed: {0}")]
    PestError(#[from] pest::error::Error<crate::parser::Rule>),
    
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
