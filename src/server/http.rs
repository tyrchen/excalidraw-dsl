// src/server/http.rs
use crate::{EDSLCompiler, EDSLError, Result};
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListQuery {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub success: bool,
    pub files: Option<Vec<FileInfo>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContentResponse {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFileResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub compiler: Arc<EDSLCompiler>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
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
    // Define allowed origins (configure these based on your deployment)
    let allowed_origins = [
        "http://localhost:3000",
        "http://localhost:5173",
        "http://localhost:5174", // Vite dev server
        "http://localhost:5175",
        "http://localhost:5176",
        "https://excalidraw.com",
        "https://excalidraw-dsl.com",
    ]
    .iter()
    .map(|origin| origin.parse::<HeaderValue>().unwrap())
    .collect::<Vec<_>>();

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/compile", post(compile_handler))
        .route("/api/validate", post(validate_handler))
        .route("/api/ws", get(websocket_handler))
        .route("/api/files", get(list_files_handler))
        .route("/api/file/{path}", get(get_file_content_handler))
        .route("/api/file/save", post(save_file_handler))
        .layer(
            ServiceBuilder::new()
                // Add request body size limit (2MB)
                .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
                // Add CORS
                .layer(
                    CorsLayer::new()
                        .allow_origin(allowed_origins)
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                        .allow_headers([
                            header::CONTENT_TYPE,
                            header::AUTHORIZATION,
                            header::UPGRADE,
                            header::CONNECTION,
                        ])
                        .expose_headers([header::CONTENT_TYPE])
                        .allow_credentials(true),
                ),
        )
        .with_state(state)
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
    let preview = req
        .edsl_content
        .lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    log::debug!("EDSL content preview:\n{preview}");

    match state.compiler.compile(&req.edsl_content) {
        Ok(excalidraw_json) => {
            // Parse the JSON string to a Value for the response
            match serde_json::from_str::<serde_json::Value>(&excalidraw_json) {
                Ok(data) => {
                    log::info!("Compilation successful, returning full Excalidraw file format");
                    Json(CompileResponse {
                        success: true,
                        data: Some(data),
                        error: None,
                    })
                    .into_response()
                }
                Err(e) => {
                    log::error!("JSON parsing failed: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(CompileResponse {
                            success: false,
                            data: None,
                            error: Some(format!("JSON parsing error: {e}")),
                        }),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            let error_message = e.to_string();
            log::warn!("Compilation failed: {error_message}");

            // Log detailed error information for debugging
            if error_message.contains("Parse error") {
                log::debug!("Parse error details: {e:?}");
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
    log::info!("WebSocket upgrade request received");
    ws.on_upgrade(|socket| super::websocket::handle_websocket(socket, state))
}

/// List EDSL files in a directory
async fn list_files_handler(Query(query): Query<FileListQuery>) -> Response {
    use std::fs;
    use std::time::SystemTime;

    log::info!("Listing files in directory: {}", query.path);

    let path = PathBuf::from(&query.path);

    // Security check: ensure path doesn't contain .. to prevent directory traversal
    if query.path.contains("..") {
        return (
            StatusCode::BAD_REQUEST,
            Json(FileListResponse {
                success: false,
                files: None,
                error: Some("Invalid path: directory traversal not allowed".to_string()),
            }),
        )
            .into_response();
    }

    match fs::read_dir(&path) {
        Ok(entries) => {
            let mut files = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("edsl") {
                        if let Ok(metadata) = entry.metadata() {
                            let modified = metadata
                                .modified()
                                .unwrap_or(SystemTime::UNIX_EPOCH)
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            files.push(FileInfo {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                size: metadata.len(),
                                modified,
                            });
                        }
                    }
                }
            }

            files.sort_by(|a, b| a.name.cmp(&b.name));

            Json(FileListResponse {
                success: true,
                files: Some(files),
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            log::error!("Failed to read directory: {e}");
            (
                StatusCode::BAD_REQUEST,
                Json(FileListResponse {
                    success: false,
                    files: None,
                    error: Some(format!("Failed to read directory: {e}")),
                }),
            )
                .into_response()
        }
    }
}

/// Get EDSL file content
async fn get_file_content_handler(Path(path): Path<String>) -> Response {
    use std::fs;

    log::info!("Getting file content: {path}");

    // Security check: ensure path doesn't contain .. to prevent directory traversal
    if path.contains("..") {
        return (
            StatusCode::BAD_REQUEST,
            Json(FileContentResponse {
                success: false,
                content: None,
                error: Some("Invalid path: directory traversal not allowed".to_string()),
            }),
        )
            .into_response();
    }

    let file_path = PathBuf::from(&path);

    // Check if file has .edsl extension
    if file_path.extension().and_then(|s| s.to_str()) != Some("edsl") {
        return (
            StatusCode::BAD_REQUEST,
            Json(FileContentResponse {
                success: false,
                content: None,
                error: Some("Only .edsl files are allowed".to_string()),
            }),
        )
            .into_response();
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => Json(FileContentResponse {
            success: true,
            content: Some(content),
            error: None,
        })
        .into_response(),
        Err(e) => {
            log::error!("Failed to read file: {e}");
            (
                StatusCode::NOT_FOUND,
                Json(FileContentResponse {
                    success: false,
                    content: None,
                    error: Some(format!("Failed to read file: {e}")),
                }),
            )
                .into_response()
        }
    }
}

/// Save EDSL file content
async fn save_file_handler(Json(req): Json<SaveFileRequest>) -> Response {
    use std::fs;

    log::info!("Saving file: {}", req.path);

    // Security check: ensure path doesn't contain .. to prevent directory traversal
    if req.path.contains("..") {
        return (
            StatusCode::BAD_REQUEST,
            Json(SaveFileResponse {
                success: false,
                error: Some("Invalid path: directory traversal not allowed".to_string()),
            }),
        )
            .into_response();
    }

    let file_path = PathBuf::from(&req.path);

    // Check if file has .edsl extension
    if file_path.extension().and_then(|s| s.to_str()) != Some("edsl") {
        return (
            StatusCode::BAD_REQUEST,
            Json(SaveFileResponse {
                success: false,
                error: Some("Only .edsl files are allowed".to_string()),
            }),
        )
            .into_response();
    }

    match fs::write(&file_path, &req.content) {
        Ok(_) => {
            log::info!("File saved successfully: {}", req.path);
            Json(SaveFileResponse {
                success: true,
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            log::error!("Failed to save file: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveFileResponse {
                    success: false,
                    error: Some(format!("Failed to save file: {e}")),
                }),
            )
                .into_response()
        }
    }
}

/// Start the HTTP server
pub async fn start_server(port: u16, state: AppState) -> Result<()> {
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .map_err(EDSLError::Io)?;

    log::info!("EDSL server starting on port {port}");
    log::info!("Health check: http://localhost:{port}/health");
    log::info!("API endpoints:");
    log::info!("  POST http://localhost:{port}/api/compile");
    log::info!("  POST http://localhost:{port}/api/validate");
    log::info!("  GET  http://localhost:{port}/api/files?path=<directory>");
    log::info!("  GET  http://localhost:{port}/api/file/<filepath>");
    log::info!("  POST http://localhost:{port}/api/file/save");
    log::info!("  WS   http://localhost:{port}/api/ws");

    axum::serve(listener, app).await.map_err(EDSLError::Io)?;

    Ok(())
}
