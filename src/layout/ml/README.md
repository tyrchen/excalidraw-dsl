# ML Layout Module

This module implements Phase 1 of the ML-enhanced layout system for Excalidraw-DSL as specified in the design document (specs/0004-ml-layout.md).

## Features

### Implemented in Phase 1

1. **Feature Extraction System**
   - Comprehensive graph feature extraction (19 features)
   - Structural, topological, and hierarchical features
   - Efficient computation with sampling for large graphs

2. **ML Model Abstractions**
   - Neural network models using Candle framework
   - Strategy selector model (4 strategies: dagre, force, elk, adaptive)
   - Performance predictor model (time, memory, CPU)
   - Quality predictor model (5 quality metrics)

3. **ML Strategy Selector**
   - Intelligent strategy selection based on graph features
   - Confidence scoring for predictions
   - Performance and quality estimation
   - Fallback mechanism for low confidence

4. **Training Data Collection**
   - Automatic collection during layout operations
   - User feedback integration
   - JSON-based storage format
   - Buffered writing for efficiency

5. **Integration with Existing System**
   - Seamless integration with LayoutManager
   - Works alongside existing layout engines
   - Adaptive fallback strategy
   - Cache-compatible

## Usage

### Enable the Feature

Add the `ml-layout` feature to your Cargo.toml:

```toml
[dependencies]
excalidraw-dsl = { version = "0.1", features = ["ml-layout"] }
```

### Basic Usage

```rust
use excalidraw_dsl::layout::{LayoutManager, MLLayoutStrategy};

// Create layout manager with ML support
let layout_manager = LayoutManager::new();

// Set layout to "ml" or "ml-enhanced"
igr.global_config.layout = Some("ml".to_string());

// Apply layout
layout_manager.layout(&mut igr)?;
```

### Advanced Usage with Custom Models

```rust
use excalidraw_dsl::layout::ml::{MLStrategySelector, MLLayoutStrategy};
use std::sync::Arc;

// Load pre-trained models
let selector = MLStrategySelector::from_path("path/to/models")?;

// Create ML strategy with custom fallback
let ml_strategy = MLLayoutStrategy::with_model_path(
    "path/to/models",
    fallback_strategy
)?;
```

### Collecting Training Data

```rust
use excalidraw_dsl::layout::ml::{TrainingDataCollector, LayoutSession};

let collector = TrainingDataCollector::new()
    .with_output_path("training_data.jsonl".to_string())
    .with_buffer_size(100);

// Collect data during layout operations
let session = LayoutSession {
    id: uuid::Uuid::new_v4().to_string(),
    igr: igr.clone(),
    context: LayoutContext::default(),
    selected_strategy: "dagre".to_string(),
    start_time: Instant::now(),
    features: extractor.extract(&igr)?,
};

collector.collect(session, performance_metrics, quality_metrics)?;
```

## Architecture

### Module Structure

```
src/layout/ml/
├── mod.rs          # Module entry point and MLLayoutStrategy
├── features.rs     # Graph feature extraction
├── model.rs        # Neural network models
├── selector.rs     # ML strategy selection logic
├── training.rs     # Training data collection
└── tests.rs        # Comprehensive tests
```

### Key Components

1. **GraphFeatureExtractor**: Extracts 19 numerical features from graphs
2. **LayoutPredictionModel**: Neural network wrapper for predictions
3. **MLStrategySelector**: Selects optimal layout strategy using ML
4. **TrainingDataCollector**: Collects and manages training data
5. **MLLayoutStrategy**: Main strategy implementation

## Model Architecture

### Strategy Selector Network
- Input: 19 graph features
- Hidden layers: 64 → 32 neurons
- Output: 4 strategies (softmax)

### Performance Predictor Network
- Input: 19 graph features
- Hidden layers: 64 → 32 neurons
- Output: 3 metrics (time, memory, CPU)

### Quality Predictor Network
- Input: 19 graph features
- Hidden layers: 64 → 32 neurons
- Output: 5 quality scores (sigmoid)

## Future Enhancements (Phase 2+)

- GNN-based layout prediction
- Reinforcement learning optimization
- Genetic algorithm integration
- Neural constraint solver
- Online learning from user feedback
- Model versioning and A/B testing

## Performance Considerations

- Models run on CPU by default
- Inference time: ~5-10ms for small models
- Feature extraction: ~1-5ms for typical graphs
- Memory overhead: ~10-20MB for models
- Training data: ~1KB per sample

## Testing

Run ML layout tests with:

```bash
cargo test --features ml-layout
```

Run the demo:

```bash
cargo run --example ml_layout_demo --features ml-layout
```
