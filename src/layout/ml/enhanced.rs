// src/layout/ml/enhanced.rs
//! Enhanced ML layout strategy that integrates GNN, RL, feedback, and constraints

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use crate::layout::{LayoutContext, LayoutStrategy};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::{
    Axis, FeedbackCollector, FeedbackType, GNNLayoutPredictor, LayoutConstraint,
    MLStrategySelector, NeuralConstraintSolver, OnlineModelUpdater, RLLayoutOptimizer,
};

/// Configuration for enhanced ML layout
#[derive(Clone)]
pub struct EnhancedMLConfig {
    /// Enable GNN for initial layout prediction
    pub use_gnn: bool,
    /// Enable RL for layout optimization
    pub use_rl: bool,
    /// Enable constraint satisfaction
    pub use_constraints: bool,
    /// Enable online learning from feedback
    pub enable_feedback: bool,
    /// Maximum RL optimization episodes
    pub rl_max_episodes: usize,
    /// Constraint solver max iterations
    pub constraint_max_iterations: usize,
    /// Model directory path
    pub model_dir: Option<String>,
}

impl Default for EnhancedMLConfig {
    fn default() -> Self {
        Self {
            use_gnn: true,
            use_rl: true,
            use_constraints: true,
            enable_feedback: true,
            rl_max_episodes: 50,
            constraint_max_iterations: 20,
            model_dir: None,
        }
    }
}

/// Enhanced ML layout strategy with all Phase 2 components
pub struct EnhancedMLLayoutStrategy {
    config: EnhancedMLConfig,
    strategy_selector: Arc<MLStrategySelector>,
    gnn_predictor: Option<GNNLayoutPredictor>,
    rl_optimizer: Option<RLLayoutOptimizer>,
    constraint_solver: Option<NeuralConstraintSolver>,
    feedback_collector: Option<Arc<FeedbackCollector>>,
    online_updater: Option<OnlineModelUpdater>,
    fallback_strategy: Arc<dyn LayoutStrategy>,
}

impl EnhancedMLLayoutStrategy {
    /// Create new enhanced ML layout strategy
    pub fn new(
        config: EnhancedMLConfig,
        fallback_strategy: Arc<dyn LayoutStrategy>,
    ) -> Result<Self> {
        let strategy_selector = if let Some(ref model_dir) = config.model_dir {
            Arc::new(MLStrategySelector::from_path(model_dir)?)
        } else {
            Arc::new(MLStrategySelector::new()?)
        };

        let gnn_predictor = if config.use_gnn {
            Some(GNNLayoutPredictor::new(32, 64, 3)?)
        } else {
            None
        };

        let rl_optimizer = if config.use_rl {
            Some(RLLayoutOptimizer::new()?)
        } else {
            None
        };

        let constraint_solver = if config.use_constraints {
            Some(NeuralConstraintSolver::new(
                config.constraint_max_iterations,
            )?)
        } else {
            None
        };

        let (feedback_collector, online_updater) = if config.enable_feedback {
            let collector = Arc::new(FeedbackCollector::new(1000));
            let updater = OnlineModelUpdater::new(0.001, 32);
            (Some(collector), Some(updater))
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            strategy_selector,
            gnn_predictor,
            rl_optimizer,
            constraint_solver,
            feedback_collector,
            online_updater,
            fallback_strategy,
        })
    }

    /// Load models from directory
    pub fn from_model_dir(
        model_dir: &str,
        config: EnhancedMLConfig,
        fallback_strategy: Arc<dyn LayoutStrategy>,
    ) -> Result<Self> {
        let mut config = config;
        config.model_dir = Some(model_dir.to_string());

        let mut strategy = Self::new(config, fallback_strategy)?;

        // Load pre-trained models if available
        let model_path = Path::new(model_dir);

        if strategy.gnn_predictor.is_some() {
            let gnn_path = model_path.join("gnn_layout.safetensors");
            if gnn_path.exists() {
                strategy.gnn_predictor = Some(GNNLayoutPredictor::from_path(&gnn_path)?);
                log::info!("Loaded pre-trained GNN model");
            }
        }

        if strategy.constraint_solver.is_some() {
            let constraint_path = model_path.join("constraint_solver.safetensors");
            if constraint_path.exists() {
                strategy.constraint_solver = Some(NeuralConstraintSolver::from_path(
                    &constraint_path,
                    strategy.config.constraint_max_iterations,
                )?);
                log::info!("Loaded pre-trained constraint solver");
            }
        }

        Ok(strategy)
    }

    /// Extract constraints from the graph context
    fn extract_constraints(
        &self,
        igr: &IntermediateGraph,
        _context: &LayoutContext,
    ) -> Vec<LayoutConstraint> {
        let mut constraints = vec![];

        // Add minimum distance constraints between all nodes
        let nodes: Vec<NodeIndex> = igr.graph.node_indices().collect();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                constraints.push(LayoutConstraint::MinDistance {
                    node1: nodes[i],
                    node2: nodes[j],
                    distance: 60.0, // Minimum node spacing
                });
            }
        }

        // Add alignment constraints for nodes at the same level (simplified)
        // In a real implementation, this would analyze the graph structure
        if nodes.len() >= 3 {
            // Example: align first three nodes horizontally
            constraints.push(LayoutConstraint::Alignment {
                nodes: nodes[..3.min(nodes.len())].to_vec(),
                axis: Axis::Horizontal,
            });
        }

        constraints
    }

    /// Apply the enhanced ML layout strategy
    fn apply_enhanced(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Step 1: Strategy selection
        let strategy_prediction = self.strategy_selector.select_strategy(igr, context)?;
        log::info!(
            "ML selected base strategy: {} (confidence: {:.2})",
            strategy_prediction.strategy_name,
            strategy_prediction.confidence
        );

        // Start feedback session if enabled
        let session_id = if let Some(ref collector) = self.feedback_collector {
            Some(collector.start_session(
                igr,
                strategy_prediction.strategy_name.clone(),
                vec![], // Alternative strategies
                strategy_prediction.quality_estimate.aesthetic_score,
            )?)
        } else {
            None
        };

        // Step 2: Initial layout with GNN or selected strategy
        let initial_positions = if self.config.use_gnn && self.gnn_predictor.is_some() {
            log::info!("Using GNN for initial layout prediction");
            let gnn = self.gnn_predictor.as_ref().unwrap();
            let prediction = gnn.predict_layout(igr)?;

            log::info!(
                "GNN prediction confidence: {:.2}, predicted {} positions",
                prediction.confidence,
                prediction.positions.len()
            );

            // Apply GNN positions to graph
            for (node_idx, &(x, y)) in &prediction.positions {
                if let Some(node) = igr.graph.node_weight_mut(*node_idx) {
                    node.x = x;
                    node.y = y;
                }
            }

            prediction.positions
        } else {
            // Use selected strategy for initial layout
            if let Some(ref strategy) = strategy_prediction.strategy {
                strategy.apply(igr, context)?;
            } else {
                self.fallback_strategy.apply(igr, context)?;
            }

            // Extract positions
            let mut positions = HashMap::new();
            for node_idx in igr.graph.node_indices() {
                if let Some(node) = igr.graph.node_weight(node_idx) {
                    positions.insert(node_idx, (node.x, node.y));
                }
            }
            positions
        };

        // Step 3: Apply constraints if enabled
        let constrained_positions =
            if self.config.use_constraints && self.constraint_solver.is_some() {
                log::info!("Applying constraint satisfaction");
                let constraints = self.extract_constraints(igr, context);
                let solver = self.constraint_solver.as_ref().unwrap();

                let solution = solver.solve(igr, &constraints, Some(&initial_positions))?;
                log::info!(
                    "Constraint solver: {} satisfied, {} violated (feasibility: {:.2})",
                    solution.satisfied_constraints.len(),
                    solution.violated_constraints.len(),
                    solution.feasibility_score
                );

                solution.positions
            } else {
                initial_positions
            };

        // Step 4: Optimize with RL if enabled
        let final_positions = if self.config.use_rl && self.rl_optimizer.is_some() {
            log::info!(
                "Optimizing layout with RL ({} episodes)",
                self.config.rl_max_episodes
            );
            let optimizer = self.rl_optimizer.as_ref().unwrap();

            optimizer.optimize_layout(igr, constrained_positions, self.config.rl_max_episodes)?
        } else {
            constrained_positions
        };

        // Step 5: Apply final positions to graph
        for (node_idx, &(x, y)) in &final_positions {
            if let Some(node) = igr.graph.node_weight_mut(*node_idx) {
                node.x = x;
                node.y = y;
            }
        }

        // Complete feedback session if enabled
        if let (Some(ref collector), Some(session_id)) = (&self.feedback_collector, session_id) {
            let elapsed = start_time.elapsed();
            collector.complete_session(
                &session_id,
                FeedbackType::Accept, // Would be determined by user interaction
                0.9,                  // Final quality score
                elapsed.as_millis() as u64,
            )?;

            // Trigger online update if enabled
            if let Some(ref updater) = self.online_updater {
                updater.queue_feedback(collector)?;
            }
        }

        log::info!(
            "Enhanced ML layout completed in {:.2}s",
            start_time.elapsed().as_secs_f64()
        );

        Ok(())
    }
}

impl LayoutStrategy for EnhancedMLLayoutStrategy {
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        // Try enhanced ML layout
        match self.apply_enhanced(igr, context) {
            Ok(()) => Ok(()),
            Err(e) => {
                log::warn!("Enhanced ML layout failed: {e}, falling back to standard strategy");
                self.fallback_strategy.apply(igr, context)
            }
        }
    }

    fn name(&self) -> &'static str {
        "enhanced-ml"
    }

    fn supports(&self, igr: &IntermediateGraph) -> bool {
        // Enhanced ML supports all graphs
        self.fallback_strategy.supports(igr)
    }
}

/// Builder for enhanced ML layout strategy
pub struct EnhancedMLLayoutBuilder {
    config: EnhancedMLConfig,
    fallback_strategy: Option<Arc<dyn LayoutStrategy>>,
}

impl EnhancedMLLayoutBuilder {
    pub fn new() -> Self {
        Self {
            config: EnhancedMLConfig::default(),
            fallback_strategy: None,
        }
    }

    pub fn with_gnn(mut self, enabled: bool) -> Self {
        self.config.use_gnn = enabled;
        self
    }

    pub fn with_rl(mut self, enabled: bool) -> Self {
        self.config.use_rl = enabled;
        self
    }

    pub fn with_constraints(mut self, enabled: bool) -> Self {
        self.config.use_constraints = enabled;
        self
    }

    pub fn with_feedback(mut self, enabled: bool) -> Self {
        self.config.enable_feedback = enabled;
        self
    }

    pub fn with_rl_episodes(mut self, episodes: usize) -> Self {
        self.config.rl_max_episodes = episodes;
        self
    }

    pub fn with_model_dir(mut self, dir: &str) -> Self {
        self.config.model_dir = Some(dir.to_string());
        self
    }

    pub fn with_fallback(mut self, strategy: Arc<dyn LayoutStrategy>) -> Self {
        self.fallback_strategy = Some(strategy);
        self
    }

    pub fn build(self) -> Result<EnhancedMLLayoutStrategy> {
        let fallback = self.fallback_strategy.ok_or_else(|| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                "Fallback strategy required".to_string(),
            ))
        })?;

        EnhancedMLLayoutStrategy::new(self.config, fallback)
    }
}

impl Default for EnhancedMLLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::layout::{DagreLayout, LayoutEngineAdapter};
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn test_enhanced_ml_strategy() {
        let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));
        let strategy = EnhancedMLLayoutBuilder::new()
            .with_gnn(true)
            .with_rl(false) // Disable RL for faster test
            .with_constraints(true)
            .with_feedback(false)
            .with_fallback(fallback)
            .build()
            .unwrap();

        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: StdHashMap::new(),
            templates: StdHashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
                NodeDefinition {
                    id: "c".to_string(),
                    label: Some("Node C".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
            ],
            edges: vec![
                EdgeDefinition {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: StdHashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "b".to_string(),
                    to: "c".to_string(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: StdHashMap::new(),
                    style: None,
                },
            ],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let mut igr = IntermediateGraph::from_ast(document).unwrap();
        let context = LayoutContext::default();

        // Apply enhanced ML layout
        strategy.apply(&mut igr, &context).unwrap();

        // Check that all nodes have positions
        for node_idx in igr.graph.node_indices() {
            let node = igr.graph.node_weight(node_idx).unwrap();
            // Check that position is set (x and y should be non-zero or at least one should be non-zero)
            assert!(
                node.x != 0.0 || node.y != 0.0,
                "Node should have position after layout"
            );
        }
    }

    #[test]
    fn test_enhanced_ml_builder() {
        let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));

        let strategy = EnhancedMLLayoutBuilder::new()
            .with_gnn(true)
            .with_rl(true)
            .with_constraints(true)
            .with_feedback(true)
            .with_rl_episodes(100)
            .with_fallback(fallback)
            .build();

        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        assert!(strategy.config.use_gnn);
        assert!(strategy.config.use_rl);
        assert_eq!(strategy.config.rl_max_episodes, 100);
    }
}
