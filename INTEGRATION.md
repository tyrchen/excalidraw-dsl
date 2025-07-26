# ExcaliDraw DSL Integration Guide

This guide demonstrates the complete integration between the Rust backend and React frontend with HTTP API and WebSocket support.

## Quick Start

### Option 1: Run Everything Together
```bash
make run-full
```
This starts both the server (port 3002) and UI (port 5173+) simultaneously.

### Option 2: Run Separately  
```bash
# Terminal 1: Start the backend server
make run-server

# Terminal 2: Start the UI
make run-ui
```

## Backend Features

### HTTP API Server
- **Framework**: Axum with tokio async runtime
- **Port**: 3002 (configurable)
- **CORS**: Enabled for frontend communication
- **Logging**: Structured logging with configurable levels

### Endpoints
- `GET /health` - Server health and feature status
- `POST /api/compile` - Compile EDSL to Excalidraw elements  
- `POST /api/validate` - Validate EDSL syntax
- `WS /api/ws` - WebSocket for real-time compilation

### WebSocket Protocol
Real-time bi-directional communication with message types:
- `compile` - Real-time compilation requests
- `validate` - Real-time validation requests
- `ping/pong` - Connection keepalive
- Connection status and version exchange

## Frontend Features

### Real-time Integration
- **WebSocket First**: Attempts WebSocket connection on startup
- **Graceful Fallback**: HTTP API → Mock compilation
- **Connection Status**: Visual indicators for connection state
- **Performance**: Sub-second response times with WebSocket

### UI Components
- **EdslEditor**: Monaco editor with EDSL syntax highlighting
- **ExcalidrawPreview**: Live diagram preview with error handling
- **ConnectionStatus**: WebSocket/HTTP mode indicator with toggle
- **FileManager**: Complete file operations with import/export

## Testing the Integration

### 1. Backend Server Test
```bash
# Start the server
cargo run --bin edsl-server --features server -- --port 3002

# Test health endpoint
curl http://localhost:3002/health

# Test compilation
curl -X POST http://localhost:3002/api/compile \
  -H "Content-Type: application/json" \
  -d '{"edsl_content":"node1[Test] -> node2[Node]"}'

# Test validation  
curl -X POST http://localhost:3002/api/validate \
  -H "Content-Type: application/json" \
  -d '{"edsl_content":"valid[Syntax]"}'
```

### 2. Frontend Integration Test
1. Open UI at http://localhost:5175 (or displayed port)
2. Check connection status indicator (should show "WebSocket connected")
3. Edit EDSL content in the editor
4. Watch real-time validation and compilation
5. View live preview in the right panel

### 3. Fallback Testing
1. Stop the backend server
2. UI should show "HTTP mode (slower)" status
3. Edit content - should still work with mock compilation
4. Restart backend - should reconnect automatically

## Example EDSL Content

Try this in the editor to see the full integration:

```edsl
---
layout: dagre
---

# Full Stack Integration Demo
frontend[React Frontend] {
  strokeColor: #3b82f6;
  backgroundColor: #dbeafe;
}

websocket[WebSocket Connection] {
  strokeColor: #10b981;
  backgroundColor: #d1fae5;
}

backend[Rust Backend] {
  strokeColor: #f59e0b;
  backgroundColor: #fef3c7;
}

parser[EDSL Parser] {
  strokeColor: #8b5cf6;
  backgroundColor: #ede9fe;
}

excalidraw[Excalidraw Render] {
  strokeColor: #ef4444;
  backgroundColor: #fee2e2;
}

# Connection flow
frontend -> websocket -> backend
backend -> parser -> excalidraw
excalidraw -> frontend
```

## Performance Comparison

| Method | Typical Response Time | Use Case |
|--------|----------------------|----------|
| WebSocket | 50-200ms | Real-time editing |
| HTTP API | 200-500ms | Batch operations |
| Mock Fallback | 10-50ms | Offline development |

## Troubleshooting

### Server Issues
- **Port conflicts**: Change port with `--port 3003`
- **Permission errors**: Check firewall settings
- **Build failures**: Run `cargo clean && cargo build --features server`

### WebSocket Issues  
- **Connection failed**: Check server is running on correct port
- **Timeout errors**: Server may be overloaded or network issues
- **Automatic fallback**: UI gracefully falls back to HTTP mode

### UI Issues
- **No preview**: Check console for compilation errors
- **Connection status**: Toggle WebSocket on/off to test fallback
- **Mock mode**: Server unavailable, using client-side compilation

## Development Workflow

1. **Backend Development**: 
   - Modify Rust code in `src/`
   - Restart server with `make run-server`
   - Test with curl or integration script

2. **Frontend Development**:
   - Modify React code in `ui/src/`
   - Hot reload automatically updates UI
   - Test WebSocket reconnection

3. **Full Integration**:
   - Use `make run-full` for complete stack
   - Test real-time compilation flow
   - Verify fallback behavior

## Production Deployment

For production deployment:
1. Build optimized Rust binary: `cargo build --release --features server`
2. Build production UI: `cd ui && yarn build`
3. Serve UI from Rust server or separate CDN
4. Configure proper CORS and security headers
5. Set up monitoring and logging
6. Use environment variables for configuration

The integration provides a robust, real-time development experience with graceful degradation for various network conditions.