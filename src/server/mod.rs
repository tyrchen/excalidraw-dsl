// src/server/mod.rs
pub mod http;
pub mod websocket;

pub use http::{create_router, start_server, AppState};
