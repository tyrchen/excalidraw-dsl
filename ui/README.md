# ExcaliDraw DSL Editor

A web-based editor for creating ExcaliDraw diagrams using a domain-specific language (EDSL).

## Features

- **Monaco Editor Integration**: Full-featured code editor with EDSL syntax highlighting
- **Live Preview**: Real-time rendering of EDSL diagrams using @excalidraw/excalidraw
- **File Management**: Create, import, export, and manage EDSL files
- **Compiler Settings**: Configure layout algorithms, validation, and output options
- **Responsive Layout**: Resizable panels and collapsible sidebars

## Getting Started

1. **Install Dependencies**:

   ```bash
   yarn install
   ```

2. **Start Development Server**:

   ```bash
   yarn dev
   ```

3. **Open in Browser**:
   Navigate to `http://localhost:5173` (or the port shown in terminal)

## EDSL Syntax

The ExcaliDraw DSL supports the following syntax:

### Basic Nodes

```edsl
# Simple node
node1[Label]

# Node with styling
node2[Label] {
  strokeColor: #ff0000;
  backgroundColor: #ffeeee;
}
```

### Connections

```edsl
# Basic arrow
node1 -> node2

# Bidirectional
node1 <-> node2

# Dashed line
node1 ~> node2

# Simple line
node1 -- node2
```

### Configuration

Add YAML frontmatter to configure the diagram:

```edsl
---
layout: dagre
theme: default
---

# Your diagram here
start[Start] -> end[End]
```

## UI Components

- **EdslEditor**: Monaco-based code editor with EDSL syntax highlighting
- **ExcalidrawPreview**: Live preview using @excalidraw/excalidraw
- **FileManager**: File operations sidebar (create, import, export, delete)
- **SettingsPanel**: Compiler configuration and status display

## Architecture

- **React + TypeScript**: Modern React with full TypeScript support
- **Zustand**: Lightweight state management
- **Tailwind CSS**: Utility-first CSS framework
- **shadcn/ui**: Pre-built accessible UI components
- **Monaco Editor**: VS Code editor in the browser
- **@excalidraw/excalidraw**: Diagram rendering engine

## Development

The UI is designed to work with the Rust EDSL compiler backend. Currently, it includes a mock compiler service for development and testing.

### Mock Compiler Service

The `EDSLCompilerService` in `src/services/edsl-compiler.ts` provides:

- EDSL parsing and validation
- Mock compilation to ExcaliDraw elements
- Error handling and reporting

### State Management

All application state is managed through Zustand store (`src/store/edsl-store.ts`):

- File management
- Editor content
- Compiler options
- Compilation results
- UI state

## Backend Integration

The UI now supports both HTTP and WebSocket connections to the Rust backend:

### HTTP API Integration

The `EDSLCompilerService` makes HTTP requests to the Rust server:

- `POST /api/compile` - Compile EDSL to Excalidraw elements
- `POST /api/validate` - Validate EDSL syntax
- `GET /health` - Server health check

### WebSocket Integration (Real-time)

For faster, real-time compilation:

- WebSocket endpoint: `ws://localhost:3002/api/ws`
- Automatic fallback to HTTP if WebSocket fails
- Connection status indicator in the editor
- Debounced validation and compilation with sub-second response times

### Running the Full Stack

1. **Start the Rust backend server:**

   ```bash
   # Terminal 1: Start the EDSL server
   make run-server
   # or
   cargo run --bin edsl-server --features server -- --port 3002
   ```

2. **Start the React frontend:**

   ```bash
   # Terminal 2: Start the UI
   make run-ui
   # or
   cd ui && yarn dev
   ```

3. **Or run both together:**

   ```bash
   make run-full
   ```

### Configuration

The frontend automatically connects to `http://localhost:3002` by default. You can:

- Toggle WebSocket on/off in the editor
- View connection status (WebSocket vs HTTP mode)
- See real-time compilation performance improvements

### Fallback Behavior

- If the backend server is not running, the UI gracefully falls back to a mock compiler
- WebSocket failures automatically fall back to HTTP API
- HTTP failures fall back to client-side mock compilation

This ensures the UI remains functional even without the backend running.
