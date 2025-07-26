// src/server/websocket.rs
use crate::server::http::AppState;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "compile")]
    Compile {
        id: String,
        edsl_content: String,
        layout: Option<String>,
        verbose: Option<bool>,
    },
    #[serde(rename = "validate")]
    Validate { id: String, edsl_content: String },
    #[serde(rename = "ping")]
    Ping { timestamp: u64 },
    #[serde(rename = "subscribe")]
    Subscribe { events: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketResponse {
    #[serde(rename = "compile_result")]
    CompileResult {
        id: String,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<String>,
        duration_ms: u64,
    },
    #[serde(rename = "validate_result")]
    ValidateResult {
        id: String,
        is_valid: bool,
        error: Option<String>,
        duration_ms: u64,
    },
    #[serde(rename = "pong")]
    Pong { timestamp: u64 },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "connected")]
    Connected {
        version: String,
        features: Vec<String>,
    },
}

/// Handle WebSocket connections for real-time compilation
pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Send connection confirmation
    let mut features = vec!["core".to_string()];
    #[cfg(feature = "llm")]
    features.push("llm".to_string());
    #[cfg(feature = "server")]
    features.push("server".to_string());

    let connected_msg = WebSocketResponse::Connected {
        version: env!("CARGO_PKG_VERSION").to_string(),
        features,
    };

    if let Ok(msg_text) = serde_json::to_string(&connected_msg) {
        if sender.send(Message::Text(msg_text.into())).await.is_err() {
            log::warn!("Failed to send connection confirmation");
            return;
        }
    }

    log::info!("WebSocket client connected");

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let start_time = Instant::now();

                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(ws_msg) => {
                        let response = handle_websocket_message(ws_msg, &state, start_time).await;

                        if let Ok(response_text) = serde_json::to_string(&response) {
                            if sender
                                .send(Message::Text(response_text.into()))
                                .await
                                .is_err()
                            {
                                log::warn!("Failed to send WebSocket response");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Invalid WebSocket message: {e}");
                        let error_response = WebSocketResponse::Error {
                            message: format!("Invalid message format: {e}"),
                        };

                        if let Ok(error_text) = serde_json::to_string(&error_response) {
                            let _ = sender.send(Message::Text(error_text.into())).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("WebSocket client disconnected");
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {
                // Ignore other message types
            }
            Err(e) => {
                log::warn!("WebSocket error: {e}");
                break;
            }
        }
    }

    log::info!("WebSocket connection closed");
}

/// Handle individual WebSocket messages
async fn handle_websocket_message(
    msg: WebSocketMessage,
    state: &AppState,
    start_time: Instant,
) -> WebSocketResponse {
    match msg {
        WebSocketMessage::Compile {
            id, edsl_content, ..
        } => {
            log::info!(
                "WebSocket compile request {} ({} chars)",
                id,
                edsl_content.len()
            );

            // Log preview for debugging
            let preview = edsl_content.lines().take(3).collect::<Vec<_>>().join("\n");
            log::debug!("EDSL preview: {preview}");

            match state.compiler.lock().unwrap().compile(&edsl_content) {
                Ok(excalidraw_json) => {
                    match serde_json::from_str::<serde_json::Value>(&excalidraw_json) {
                        Ok(data) => {
                            log::info!(
                                "WebSocket compilation successful, returning full Excalidraw file"
                            );
                            WebSocketResponse::CompileResult {
                                id,
                                success: true,
                                data: Some(data),
                                error: None,
                                duration_ms: start_time.elapsed().as_millis() as u64,
                            }
                        }
                        Err(e) => WebSocketResponse::CompileResult {
                            id,
                            success: false,
                            data: None,
                            error: Some(format!("JSON parsing error: {e}")),
                            duration_ms: start_time.elapsed().as_millis() as u64,
                        },
                    }
                }
                Err(e) => WebSocketResponse::CompileResult {
                    id,
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
            }
        }
        WebSocketMessage::Validate { id, edsl_content } => {
            log::debug!(
                "WebSocket validate request {} ({} chars)",
                id,
                edsl_content.len()
            );

            match state.compiler.lock().unwrap().validate(&edsl_content) {
                Ok(_) => WebSocketResponse::ValidateResult {
                    id,
                    is_valid: true,
                    error: None,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
                Err(e) => WebSocketResponse::ValidateResult {
                    id,
                    is_valid: false,
                    error: Some(e.to_string()),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
            }
        }
        WebSocketMessage::Ping { timestamp } => WebSocketResponse::Pong { timestamp },
        WebSocketMessage::Subscribe { .. } => {
            // For future features like collaborative editing
            WebSocketResponse::Error {
                message: "Subscribe feature not yet implemented".to_string(),
            }
        }
    }
}

/// WebSocket keepalive handler
pub async fn websocket_keepalive(mut sender: futures_util::stream::SplitSink<WebSocket, Message>) {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        let ping_msg = WebSocketMessage::Ping {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        if let Ok(ping_text) = serde_json::to_string(&ping_msg) {
            if sender.send(Message::Text(ping_text.into())).await.is_err() {
                break;
            }
        } else {
            break;
        }
    }
}
