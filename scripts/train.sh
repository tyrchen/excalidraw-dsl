#!/bin/bash

# ML Layout Training Script
# Comprehensive training pipeline for ExcaliDraw-DSL ML layout models

set -e  # Exit on any error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
OUTPUT_DIR="./models"
CONFIG="quick-test"
PHASES="data,gnn,rl,constraint,enhanced,eval"
SAMPLES=1000
VERBOSE=false
M4_OPTIMIZE=false
SKIP_EVAL=false
CLEAN=false
BUILD_FEATURES=""

# Help function
show_help() {
    echo -e "${BLUE}ML Layout Training Script${NC}"
    echo -e "${CYAN}Usage: $0 [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo "  -o, --output DIR        Output directory for trained models (default: ./models)"
    echo "  -c, --config CONFIG     Training configuration: quick-test, m4-optimized (default: quick-test)"
    echo "  -p, --phases PHASES     Training phases to run (default: data,gnn,rl,constraint,enhanced,eval)"
    echo "  -s, --samples NUM       Number of training samples (default: 1000)"
    echo "  -v, --verbose           Enable verbose output"
    echo "  -m, --m4-optimize       Use Apple M4 optimizations"
    echo "  --skip-eval             Skip model evaluation phase"
    echo "  --clean                 Clean output directory before training"
    echo "  --features FEATURES     Additional cargo features to enable"
    echo "  -h, --help              Show this help message"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  $0                                    # Quick test training"
    echo "  $0 -c m4-optimized -m -v             # M4 optimized training with verbose output"
    echo "  $0 -p data,gnn,rl -s 5000            # Train only data, GNN, and RL with 5000 samples"
    echo "  $0 --clean -o ./my_models            # Clean training with custom output directory"
    echo ""
    echo -e "${YELLOW}Training Phases:${NC}"
    echo "  data       - Generate training datasets"
    echo "  gnn        - Train Graph Neural Network model"
    echo "  rl         - Train Reinforcement Learning model"
    echo "  constraint - Train constraint solver model"
    echo "  enhanced   - Train integrated enhanced model"
    echo "  eval       - Evaluate all trained models"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -c|--config)
            CONFIG="$2"
            shift 2
            ;;
        -p|--phases)
            PHASES="$2"
            shift 2
            ;;
        -s|--samples)
            SAMPLES="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -m|--m4-optimize)
            M4_OPTIMIZE=true
            shift
            ;;
        --skip-eval)
            SKIP_EVAL=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --features)
            BUILD_FEATURES="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Print header
echo -e "${BLUE}🚀 ExcaliDraw-DSL ML Layout Training Pipeline${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/main.rs" ]]; then
    echo -e "${RED}❌ Error: This script must be run from the excalidraw-dsl project root directory${NC}"
    exit 1
fi

# Check if ml-layout feature is available
if ! grep -q "ml-layout" Cargo.toml; then
    echo -e "${RED}❌ Error: ml-layout feature not found in Cargo.toml${NC}"
    exit 1
fi

# Clean output directory if requested
if [[ "$CLEAN" == "true" ]]; then
    echo -e "${YELLOW}🧹 Cleaning output directory: $OUTPUT_DIR${NC}"
    if [[ -d "$OUTPUT_DIR" ]]; then
        rm -rf "$OUTPUT_DIR"
    fi
fi

# Create output directory
echo -e "${CYAN}📁 Creating output directory: $OUTPUT_DIR${NC}"
mkdir -p "$OUTPUT_DIR"

# Build features string
CARGO_FEATURES="ml-layout"
if [[ -n "$BUILD_FEATURES" ]]; then
    CARGO_FEATURES="$CARGO_FEATURES,$BUILD_FEATURES"
fi

# Check if we need to build first
echo -e "${CYAN}🔨 Building project with features: $CARGO_FEATURES${NC}"
if ! cargo build --features "$CARGO_FEATURES" --bin edsl; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful${NC}"
echo ""

# Prepare training command
TRAIN_CMD="cargo run --features $CARGO_FEATURES --bin edsl -- train"
TRAIN_CMD="$TRAIN_CMD --output \"$OUTPUT_DIR\""
TRAIN_CMD="$TRAIN_CMD --config \"$CONFIG\""
TRAIN_CMD="$TRAIN_CMD --phases \"$PHASES\""
TRAIN_CMD="$TRAIN_CMD --samples $SAMPLES"

if [[ "$VERBOSE" == "true" ]]; then
    TRAIN_CMD="$TRAIN_CMD --verbose"
fi

if [[ "$M4_OPTIMIZE" == "true" ]]; then
    TRAIN_CMD="$TRAIN_CMD --m4-optimize"
fi

if [[ "$SKIP_EVAL" == "true" ]]; then
    TRAIN_CMD="$TRAIN_CMD --skip-eval"
fi

# Display training configuration
echo -e "${YELLOW}📋 Training Configuration:${NC}"
echo "  Output Directory: $OUTPUT_DIR"
echo "  Configuration: $CONFIG"
echo "  Training Phases: $PHASES"
echo "  Sample Count: $SAMPLES"
echo "  Verbose Mode: $VERBOSE"
echo "  M4 Optimization: $M4_OPTIMIZE"
echo "  Skip Evaluation: $SKIP_EVAL"
echo "  Cargo Features: $CARGO_FEATURES"
echo ""

# Confirm before starting (unless verbose mode is on, which implies automation)
if [[ "$VERBOSE" != "true" ]]; then
    echo -e "${YELLOW}Press Enter to start training, or Ctrl+C to cancel...${NC}"
    read
fi

# Record start time
START_TIME=$(date +%s)
echo -e "${PURPLE}⏰ Training started at: $(date)${NC}"
echo ""

# Run training
echo -e "${CYAN}🎯 Executing training command:${NC}"
echo "$TRAIN_CMD"
echo ""

# Execute the training command
if eval "$TRAIN_CMD"; then
    # Record end time and calculate duration
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    MINUTES=$((DURATION / 60))
    SECONDS=$((DURATION % 60))

    echo ""
    echo -e "${GREEN}🎉 Training completed successfully!${NC}"
    echo -e "${CYAN}⏱️  Total time: ${MINUTES}m ${SECONDS}s${NC}"
    echo ""

    # Show output files
    echo -e "${YELLOW}📄 Generated Files:${NC}"
    if [[ -f "$OUTPUT_DIR/training_metrics.json" ]]; then
        echo "  📊 Metrics: $OUTPUT_DIR/training_metrics.json"
    fi
    if [[ -f "$OUTPUT_DIR/training_report.md" ]]; then
        echo "  📄 Report: $OUTPUT_DIR/training_report.md"
    fi
    if [[ -d "$OUTPUT_DIR/models" ]]; then
        echo "  🗂️  Models: $OUTPUT_DIR/models/"
        find "$OUTPUT_DIR/models" -name "*.bin" -exec echo "    - {}" \; 2>/dev/null || true
    fi
    if [[ -d "$OUTPUT_DIR/data" ]]; then
        echo "  📚 Data: $OUTPUT_DIR/data/"
    fi

    echo ""
    echo -e "${CYAN}🚀 Next Steps:${NC}"
    echo "  1. Review the training report: $OUTPUT_DIR/training_report.md"
    echo "  2. Examine metrics: $OUTPUT_DIR/training_metrics.json"
    echo "  3. Use the trained models in your layout configurations"
    echo ""

    # Apple M4 specific recommendations
    if [[ "$M4_OPTIMIZE" == "true" ]]; then
        echo -e "${PURPLE}🍎 M4 Optimization Notes:${NC}"
        echo "  - Models are optimized for Apple M4 Metal Performance Shaders"
        echo "  - Consider enabling GPU acceleration in your production environment"
        echo "  - Monitor memory usage as M4 optimizations may use more VRAM"
        echo ""
    fi

else
    echo ""
    echo -e "${RED}❌ Training failed!${NC}"
    echo -e "${YELLOW}💡 Troubleshooting tips:${NC}"
    echo "  1. Check that all dependencies are installed"
    echo "  2. Ensure sufficient disk space in $OUTPUT_DIR"
    echo "  3. Run with --verbose flag for detailed error messages"
    echo "  4. Try the 'quick-test' configuration for faster debugging"
    echo "  5. Check the build logs above for compilation errors"
    echo ""
    exit 1
fi

# Offer to open report if it exists
if [[ -f "$OUTPUT_DIR/training_report.md" ]] && command -v code &> /dev/null; then
    echo -e "${CYAN}Would you like to open the training report in VS Code? (y/N)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        code "$OUTPUT_DIR/training_report.md"
    fi
fi

echo -e "${GREEN}✨ Training pipeline complete!${NC}"
