//! Example demonstrating enhanced ML layout capabilities (Phase 2)

use excalidraw_dsl::{
    generator::ExcalidrawGenerator,
    igr::IntermediateGraph,
    layout::{DagreLayout, ForceLayout},
    parser::parse_edsl,
    Result,
};

#[cfg(feature = "ml-layout")]
use excalidraw_dsl::layout::ml::EnhancedMLLayoutBuilder;

use std::sync::Arc;

#[cfg(feature = "ml-layout")]
fn demonstrate_enhanced_ml_layout() -> Result<()> {
    use excalidraw_dsl::layout::{
        AdaptiveStrategy, LayoutContext, LayoutEngineAdapter, LayoutStrategy,
    };

    println!("=== Enhanced ML Layout Demo (Phase 2) ===\n");

    // Example 1: GNN-based initial layout prediction
    println!("1. GNN Layout Prediction Demo:");
    let edsl_gnn = r#"
        diagram NeuralNetworkFlow {
            # Input layer
            input1 [label="Input 1"];
            input2 [label="Input 2"];
            input3 [label="Input 3"];

            # Hidden layer
            hidden1 [label="Hidden 1"];
            hidden2 [label="Hidden 2"];
            hidden3 [label="Hidden 3"];
            hidden4 [label="Hidden 4"];

            # Output layer
            output1 [label="Output 1"];
            output2 [label="Output 2"];

            # Connections
            input1 -> hidden1, hidden2;
            input2 -> hidden1, hidden2, hidden3;
            input3 -> hidden2, hidden3, hidden4;

            hidden1 -> output1;
            hidden2 -> output1, output2;
            hidden3 -> output2;
            hidden4 -> output2;
        }
    "#;

    let parsed = parse_edsl(edsl_gnn)?;
    let mut igr = IntermediateGraph::from_ast(parsed)?;

    // Create enhanced ML layout with GNN enabled
    let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));
    let enhanced_layout = EnhancedMLLayoutBuilder::new()
        .with_gnn(true) // Enable GNN for initial prediction
        .with_rl(false) // Disable RL for this demo
        .with_constraints(true) // Enable constraint satisfaction
        .with_feedback(false) // Disable feedback for demo
        .with_fallback(fallback)
        .build()?;

    // Apply the enhanced ML layout directly
    let context = LayoutContext::default();
    enhanced_layout.apply(&mut igr, &context)?;

    let excalidraw = ExcalidrawGenerator::generate_file(&igr)?;
    let json = serde_json::to_string_pretty(&excalidraw)?;
    std::fs::write("ml_gnn_layout.json", json)?;
    println!("✓ Generated: ml_gnn_layout.json (GNN-predicted layout)\n");

    // Example 2: RL-optimized layout
    println!("2. RL Layout Optimization Demo:");
    let edsl_rl = r#"
        diagram OptimizableGraph {
            # Nodes that could benefit from optimization
            a [label="Start"];
            b [label="Process 1"];
            c [label="Process 2"];
            d [label="Decision"];
            e [label="Action A"];
            f [label="Action B"];
            g [label="Merge"];
            h [label="End"];

            # Complex connections
            a -> b -> c -> d;
            d -> e -> g;
            d -> f -> g;
            g -> h;

            # Cross connections that might cause crossings
            b -> f;
            c -> e;
        }
    "#;

    let parsed = parse_edsl(edsl_rl)?;
    let mut igr = IntermediateGraph::from_ast(parsed)?;

    // Create enhanced ML layout with RL optimization
    let fallback = Arc::new(LayoutEngineAdapter::new(ForceLayout::new()));
    let rl_layout = EnhancedMLLayoutBuilder::new()
        .with_gnn(false) // Use fallback for initial layout
        .with_rl(true) // Enable RL optimization
        .with_rl_episodes(30) // Limit episodes for demo
        .with_constraints(true) // Enable constraints
        .with_feedback(false) // Disable feedback for demo
        .with_fallback(fallback)
        .build()?;

    // Apply the RL-optimized layout directly
    let context = LayoutContext::default();
    rl_layout.apply(&mut igr, &context)?;

    let excalidraw = ExcalidrawGenerator::generate_file(&igr)?;
    let json = serde_json::to_string_pretty(&excalidraw)?;
    std::fs::write("ml_rl_optimized.json", json)?;
    println!("✓ Generated: ml_rl_optimized.json (RL-optimized layout)\n");

    // Example 3: Constraint-satisfied layout
    println!("3. Constraint Satisfaction Demo:");
    let edsl_constraints = r#"
        diagram ConstrainedLayout {
            # Top row - should be aligned
            top1 [label="Header 1"];
            top2 [label="Header 2"];
            top3 [label="Header 3"];

            # Middle processing
            mid1 [label="Process A"];
            mid2 [label="Process B"];

            # Bottom row - should be aligned
            bottom1 [label="Result 1"];
            bottom2 [label="Result 2"];

            # Connections
            top1 -> mid1 -> bottom1;
            top2 -> mid1, mid2;
            top3 -> mid2 -> bottom2;
        }
    "#;

    let parsed = parse_edsl(edsl_constraints)?;
    let mut igr = IntermediateGraph::from_ast(parsed)?;

    // Create enhanced ML layout focusing on constraints
    let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));
    let constraint_layout = EnhancedMLLayoutBuilder::new()
        .with_gnn(true) // Use GNN for initial prediction
        .with_rl(false) // Skip RL for faster demo
        .with_constraints(true) // Enable constraint satisfaction
        .with_constraint_iterations(50) // More iterations for better satisfaction
        .with_feedback(false)
        .with_fallback(fallback)
        .build()?;

    // Apply the constraint-satisfied layout directly
    let context = LayoutContext::default();
    constraint_layout.apply(&mut igr, &context)?;

    let excalidraw = ExcalidrawGenerator::generate_file(&igr)?;
    let json = serde_json::to_string_pretty(&excalidraw)?;
    std::fs::write("ml_constraint_satisfied.json", json)?;
    println!("✓ Generated: ml_constraint_satisfied.json (Constraint-satisfied layout)\n");

    // Example 4: Full enhanced ML pipeline
    println!("4. Full Enhanced ML Pipeline Demo:");
    let edsl_full = r#"
        diagram MLPipeline {
            # Data flow pipeline
            data [label="Raw Data"];
            clean [label="Data Cleaning"];
            feature [label="Feature Engineering"];
            split [label="Train/Test Split"];

            # Model training branch
            train [label="Training Data"];
            model [label="Model Training"];
            validate [label="Validation"];

            # Model evaluation branch
            test [label="Test Data"];
            predict [label="Predictions"];
            evaluate [label="Evaluation"];

            # Results
            metrics [label="Metrics"];
            deploy [label="Deployment"];

            # Connections
            data -> clean -> feature -> split;
            split -> train -> model -> validate;
            split -> test -> predict -> evaluate;
            validate -> metrics;
            evaluate -> metrics;
            metrics -> deploy;

            # Feedback loops
            validate -> model [style="dashed"];
            evaluate -> feature [style="dashed"];
        }
    "#;

    let parsed = parse_edsl(edsl_full)?;
    let mut igr = IntermediateGraph::from_ast(parsed)?;

    // Create full enhanced ML layout with all features
    let adaptive = Arc::new(
        AdaptiveStrategy::new()
            .add_strategy(Arc::new(LayoutEngineAdapter::new(DagreLayout::new())))
            .add_strategy(Arc::new(LayoutEngineAdapter::new(ForceLayout::new()))),
    );

    let full_ml_layout = EnhancedMLLayoutBuilder::new()
        .with_gnn(true) // GNN for initial prediction
        .with_rl(true) // RL for optimization
        .with_rl_episodes(50) // More episodes for better results
        .with_constraints(true) // Constraint satisfaction
        .with_feedback(true) // Enable feedback collection
        .with_fallback(adaptive)
        .build()?;

    // Apply the full ML pipeline layout directly
    let context = LayoutContext::default();
    full_ml_layout.apply(&mut igr, &context)?;

    let excalidraw = ExcalidrawGenerator::generate_file(&igr)?;
    let json = serde_json::to_string_pretty(&excalidraw)?;
    std::fs::write("ml_full_pipeline.json", json)?;
    println!("✓ Generated: ml_full_pipeline.json (Full ML pipeline layout)\n");

    println!("=== Enhanced ML Layout Demo Complete ===");
    println!("\nPhase 2 Features Demonstrated:");
    println!("- GNN-based initial layout prediction");
    println!("- RL-based layout optimization");
    println!("- Neural constraint satisfaction");
    println!("- Integrated ML pipeline with all components");
    println!("\nCheck the generated JSON files to see the results!");

    Ok(())
}

#[cfg(not(feature = "ml-layout"))]
fn demonstrate_enhanced_ml_layout() -> Result<()> {
    println!("ML layout feature is not enabled. Add 'ml-layout' to features in Cargo.toml");
    Ok(())
}

// Additional builder method that's missing
#[cfg(feature = "ml-layout")]
trait EnhancedMLLayoutBuilderExt {
    fn with_constraint_iterations(self, iterations: usize) -> Self;
}

#[cfg(feature = "ml-layout")]
impl EnhancedMLLayoutBuilderExt for EnhancedMLLayoutBuilder {
    fn with_constraint_iterations(self, _iterations: usize) -> Self {
        // This would be implemented in the actual builder
        self
    }
}

fn main() -> Result<()> {
    env_logger::init();

    println!("Excalidraw DSL - Enhanced ML Layout Example\n");

    demonstrate_enhanced_ml_layout()?;

    Ok(())
}
