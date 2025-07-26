// src/server/rate_limit.rs
use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Simple in-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    /// Maximum requests per window
    max_requests: u32,
    /// Time window duration
    window: Duration,
    /// Client tracking map
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
}

#[derive(Debug)]
struct ClientInfo {
    request_count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, req: Request, next: Next) -> Result<Response, StatusCode> {
        // Extract client identifier (IP address)
        let client_id =
            if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
                addr.ip().to_string()
            } else {
                // Fallback to a default if no IP available
                "unknown".to_string()
            };

        // Check rate limit
        let now = Instant::now();
        let mut clients = self.clients.lock().unwrap();

        let client_info = clients.entry(client_id.clone()).or_insert(ClientInfo {
            request_count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(client_info.window_start) > self.window {
            client_info.request_count = 0;
            client_info.window_start = now;
        }

        // Check if limit exceeded
        if client_info.request_count >= self.max_requests {
            log::warn!("Rate limit exceeded for client: {client_id}");
            return Ok((
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded. Please try again later.",
            )
                .into_response());
        }

        // Increment counter
        client_info.request_count += 1;
        drop(clients); // Release lock before proceeding

        // Continue with request
        Ok(next.run(req).await)
    }

    /// Clean up old entries periodically (optional)
    pub fn cleanup_old_entries(&self) {
        let now = Instant::now();
        let mut clients = self.clients.lock().unwrap();

        clients.retain(|_, info| now.duration_since(info.window_start) <= self.window * 2);
    }
}

/// Middleware function for rate limiting
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    limiter.check_rate_limit(req, next).await
}
