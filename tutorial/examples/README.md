# Excalidraw DSL Examples

This directory contains example DSL files to help you learn the language.

## Examples Overview

### 01-basic.edsl
The simplest example showing basic nodes and edges. Perfect for beginners.
- Creating nodes with labels
- Connecting nodes with arrows
- Basic edge labels

### 02-containers.edsl
Learn how to organize nodes using containers.
- Creating containers
- Nesting nodes inside containers
- Connecting nodes across containers

### 03-styling.edsl
Customize the appearance of your diagrams.
- Setting background colors
- Changing text colors
- Styling edges (solid, dashed)
- Using different stroke widths

### 04-microservices.edsl
Real-world microservices architecture example.
- Using component types for consistent styling
- Creating service and database nodes
- Complex interconnections
- Message queue integration

### 05-groups.edsl
Logical grouping without visual boundaries.
- Creating groups
- Organizing team members
- Task assignments and dependencies

### 06-edge-chains.edsl
Efficient ways to create sequences.
- Simple chains
- Chains with labels
- Branching and merging paths

### 07-complex-system.edsl
Complete system architecture example.
- Multiple layers (client, service, data)
- Nested containers
- External service integration
- Different component types

### 08-text-colors.edsl
Text color customization examples.
- Setting text colors on nodes
- Container text color inheritance
- Combining text and background colors

## Running the Examples

To compile any example:

```bash
# From the tutorial directory
edsl examples/01-basic.edsl -o output.excalidraw

# Or from the project root
edsl tutorial/examples/01-basic.edsl -o output.excalidraw
```

Then open the output file in Excalidraw to see the result!

## Tips

1. Start with `01-basic.edsl` if you're new
2. Each example builds on previous concepts
3. Read the comments in each file for explanations
4. Try modifying the examples to learn
5. Combine techniques from different examples
