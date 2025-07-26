// src/server/http.rs
use crate::{EDSLCompiler, EDSLError, Result};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileRequest {
    pub edsl_content: String,
    pub layout: Option<String>,
    pub verbose: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub edsl_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub is_valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub features: Vec<String>,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub compiler: Arc<EDSLCompiler>,
}

impl AppState {
    pub fn new() -> Self {
        // Use default compiler (LLM optimization disabled by default)
        Self {
            compiler: Arc::new(EDSLCompiler::new()),
        }
    }

    #[cfg(feature = "llm")]
    pub fn with_llm(_api_key: String) -> Self {
        // Note: LLM optimization disabled in server context due to runtime conflicts
        log::warn!("LLM optimization disabled in server context due to runtime conflicts");
        Self {
            compiler: Arc::new(EDSLCompiler::new()),
        }
    }
}

/// Create the main HTTP router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/compile", post(compile_handler))
        .route("/api/validate", post(validate_handler))
        .route("/api/ws", get(websocket_handler))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .expose_headers([header::CONTENT_TYPE]),
        )
}

/// Health check endpoint
async fn health_handler() -> Json<HealthResponse> {
    let mut features = vec!["core".to_string()];

    #[cfg(feature = "llm")]
    features.push("llm".to_string());

    #[cfg(feature = "server")]
    features.push("server".to_string());

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        features,
    })
}

/// Compile EDSL to Excalidraw elements
async fn compile_handler(
    State(state): State<AppState>,
    Json(req): Json<CompileRequest>,
) -> Response {
    log::info!("Compiling EDSL content ({} chars)", req.edsl_content.len());
    
    // Log first few lines of content for debugging
    let preview = req.edsl_content
        .lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    log::debug!("EDSL content preview:\n{}", preview);

    match state.compiler.compile_to_elements(&req.edsl_content) {
        Ok(elements) => {
            // Convert elements to JSON Value for frontend compatibility
            match serde_json::to_value(&elements) {
                Ok(data) => Json(CompileResponse {
                    success: true,
                    data: Some(data),
                    error: None,
                })
                .into_response(),
                Err(e) => {
                    log::error!("JSON serialization failed: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(CompileResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Serialization error: {}", e)),
                        }),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            let error_message = e.to_string();
            log::warn!("Compilation failed: {}", error_message);
            
            // Log detailed error information for debugging
            if error_message.contains("Parse error") {
                log::debug!("Parse error details: {:?}", e);
                log::info!("💡 Hint: Make sure to quote color values in style blocks (e.g., \"#3b82f6\" not #3b82f6)");
            }
            
            (
                StatusCode::BAD_REQUEST,
                Json(CompileResponse {
                    success: false,
                    data: None,
                    error: Some(error_message),
                }),
            )
                .into_response()
        }
    }
}

/// Validate EDSL syntax
async fn validate_handler(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Response {
    log::debug!("Validating EDSL content ({} chars)", req.edsl_content.len());

    match state.compiler.validate(&req.edsl_content) {
        Ok(_) => Json(ValidateResponse {
            is_valid: true,
            error: None,
        })
        .into_response(),
        Err(e) => Json(ValidateResponse {
            is_valid: false,
            error: Some(e.to_string()),
        })
        .into_response(),
    }
}

/// WebSocket upgrade handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| super::websocket::handle_websocket(socket, state))
}

/// Start the HTTP server
pub async fn start_server(port: u16, state: AppState) -> Result<()> {
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| EDSLError::Io(e.into()))?;

    log::info!("EDSL server starting on port {}", port);
    log::info!("Health check: http://localhost:{}/health", port);
    log::info!("API endpoints:");
    log::info!("  POST http://localhost:{}/api/compile", port);
    log::info!("  POST http://localhost:{}/api/validate", port);
    log::info!("  WS   http://localhost:{}/api/ws", port);

    axum::serve(listener, app)
        .await
        .map_err(|e| EDSLError::Io(e.into()))?;

    Ok(())
}
