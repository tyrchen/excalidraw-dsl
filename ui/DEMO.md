# ExcaliDraw DSL UI Demo

## Testing the UI (Without Backend)

The UI includes a robust fallback system that works even when the backend server is not running.

### Current Features Working:

1. **Mock Compilation**: The UI includes a client-side EDSL compiler for development
2. **Real-time Preview**: Live Excalidraw diagram rendering as you type
3. **File Management**: Create, import, export EDSL files
4. **Syntax Highlighting**: Monaco editor with custom EDSL syntax
5. **Validation**: Client-side syntax validation with error reporting

### Test Content

Try pasting this EDSL content into the editor:

```edsl
---
layout: dagre
---

# Frontend Demo - Working Example
ui[React UI] {
  strokeColor: "#3b82f6";
  backgroundColor: "#dbeafe";
}

mock[Mock Compiler] {
  strokeColor: "#10b981";
  backgroundColor: "#d1fae5";
}

excalidraw[Excalidraw Canvas] {
  strokeColor: "#f59e0b";
  backgroundColor: "#fef3c7";
}

# Flow
ui -> mock -> excalidraw
```

### Common Syntax Errors and Fixes

❌ **Wrong**: `backgroundColor: #dbeafe;`
✅ **Correct**: `backgroundColor: "#dbeafe";`

❌ **Wrong**: Using nodes before defining them
✅ **Correct**: Define all nodes first, then connections

❌ **Wrong**: Cycles in graph (e.g., `a -> b -> a`)
✅ **Correct**: Use acyclic graphs (no cycles)

### Alternative Example with Edge Labels

```edsl
---
layout: dagre
---

# Example EDSL Diagram - Define all nodes first
start[Start]
process[Process Data]
decision[Decision]
end[End]
error[Error]

# Then define the connections (acyclic flow)
start -> process
process -> decision{Decision?}
decision -> end{Success}
decision -> error{Error}
```

### Connection Status

- **Orange status**: "Server offline - using mock compiler"
- This is expected when the backend is not running
- Toggle WebSocket on/off to test different modes
- All core functionality works in mock mode

### With Backend Server

To test with the real backend:

1. Run `./start-backend.sh` in another terminal
2. Refresh the UI - should show green "WebSocket connected"
3. Real-time compilation with the Rust backend
4. Sub-second response times

### Error Handling

The UI gracefully handles:
- Server offline (falls back to mock)
- WebSocket connection failures (falls back to HTTP)
- HTTP API failures (falls back to mock)
- Invalid EDSL syntax (shows validation errors)
- Network timeouts and interruptions

This ensures the development experience remains smooth regardless of backend availability.