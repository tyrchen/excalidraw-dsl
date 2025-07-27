# CLI Examples

## Basic Usage

### Convert EDSL to Excalidraw
```bash
# Basic conversion
edsl convert diagram.edsl

# Specify output file
edsl convert diagram.edsl -o output.excalidraw

# Use different layout algorithm
edsl convert diagram.edsl --layout force

# Verbose output
edsl convert diagram.edsl -v
```

### Validate EDSL Syntax
```bash
# Basic validation
edsl validate diagram.edsl

# Verbose validation with statistics
edsl validate diagram.edsl -v
```

### Validate Excalidraw Files
```bash
# Validate Excalidraw JSON file
edsl validate-excalidraw drawing.excalidraw

# Verbose validation with element count
edsl validate-excalidraw drawing.excalidraw -v

# Using alias
edsl validate-ex drawing.excalidraw
```

### Watch Mode
```bash
# Watch file and auto-recompile on changes
edsl watch diagram.edsl

# Watch with custom output
edsl watch diagram.edsl -o live-output.excalidraw

# Or use convert with watch flag
edsl convert diagram.edsl --watch
```

### Run Server
```bash
# Start server on default port 3002
edsl server

# Custom port
edsl server --port 8080

# Custom host and port
edsl server --host 127.0.0.1 --port 8080

# Verbose logging
edsl server -v
```

## Real Examples

### Convert All Examples
```bash
# Using Make
make examples

# Or manually
edsl convert examples/simple.edsl -o examples/simple.excalidraw
edsl convert examples/complex-architecture.edsl -o examples/complex-architecture.excalidraw
edsl convert examples/decision-tree.edsl -o examples/decision-tree.excalidraw
```

### Development Workflow
```bash
# Start server and UI together
make run-full

# Or run them separately
edsl server &
cd ui && yarn dev
```

### Installation
```bash
# Install globally
cargo install --path . --features server

# Then use from anywhere
edsl convert my-diagram.edsl
```

## Tips

1. The `convert` command has an alias `compile` for those who prefer it:
   ```bash
   edsl compile diagram.edsl
   ```

2. The `server` command has an alias `serve`:
   ```bash
   edsl serve --port 8080
   ```

3. Use verbose mode (`-v`) to see detailed information about the compilation process

4. The watch mode is great for live editing - keep it running while you edit your EDSL file in your favorite editor
