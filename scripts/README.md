# ExcaliDraw-DSL ML Training Scripts

This directory contains training scripts for the ML layout models.

## Training Script

### Usage

```bash
# Quick test training
./scripts/train.sh

# Apple M4 optimized training with verbose output
./scripts/train.sh -c m4-optimized -m -v

# Train specific phases only
./scripts/train.sh -p data,gnn,rl

# Custom configuration
./scripts/train.sh -o ./my_models -s 5000 --clean
```

### Options

- `-o, --output DIR`: Output directory for trained models (default: ./models)
- `-c, --config CONFIG`: Training configuration (quick-test, m4-optimized)
- `-p, --phases PHASES`: Comma-separated training phases to run
- `-s, --samples NUM`: Number of training samples to generate
- `-v, --verbose`: Enable verbose output
- `-m, --m4-optimize`: Use Apple M4 optimizations
- `--skip-eval`: Skip model evaluation phase
- `--clean`: Clean output directory before training
- `--features FEATURES`: Additional cargo features to enable

### Training Phases

- **data**: Generate training datasets
- **gnn**: Train Graph Neural Network model
- **rl**: Train Reinforcement Learning model
- **constraint**: Train constraint solver model
- **enhanced**: Train integrated enhanced model
- **eval**: Evaluate all trained models

### CLI Command

You can also use the CLI directly:

```bash
# Train with default settings
cargo run --features ml-layout --bin edsl -- train

# Custom training
cargo run --features ml-layout --bin edsl -- train \
  --output ./models \
  --config m4-optimized \
  --phases data,gnn,rl,enhanced \
  --samples 2000 \
  --verbose \
  --m4-optimize
```

### Output Files

After training completes, you'll find:

- `models/`: Directory containing trained model files
  - `gnn_model.bin`: GNN model
  - `rl_model.bin`: RL model
  - `constraint_model.bin`: Constraint solver model
  - `enhanced_model.bin`: Enhanced integrated model
- `data/`: Training datasets
- `training_metrics.json`: Detailed training metrics
- `training_report.md`: Human-readable training report
- `training.log`: Detailed training logs

### Apple M4 Optimization

When using `--m4-optimize`, the training process is optimized for Apple M4 processors with:

- Metal Performance Shaders acceleration
- Optimized memory allocation patterns
- M4-specific neural network configurations
- Enhanced parallel processing

### Troubleshooting

If training fails:

1. Ensure you have sufficient disk space
2. Check that the `ml-layout` feature is enabled
3. Run with `--verbose` for detailed error messages
4. Try the `quick-test` configuration first
5. Check build logs for compilation errors

### Example Workflows

#### Development Testing
```bash
# Quick validation
./scripts/train.sh -c quick-test -p data,gnn -s 100 --skip-eval -v
```

#### Production Training
```bash
# Full training with M4 optimization
./scripts/train.sh -c m4-optimized -m -s 10000 --clean -v
```

#### Specific Model Training
```bash
# Train only RL and enhanced models
./scripts/train.sh -p rl,enhanced -s 2000 -v
```
