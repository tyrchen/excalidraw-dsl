// src/layout/ml/mod.rs
//! Machine Learning enhanced layout module for intelligent layout selection and optimization

#[cfg(feature = "ml-layout")]
mod features;
#[cfg(feature = "ml-layout")]
mod model;
#[cfg(feature = "ml-layout")]
mod selector;
#[cfg(feature = "ml-layout")]
mod training;

// Phase 2 modules
#[cfg(feature = "ml-layout")]
mod constraints;
#[cfg(feature = "ml-layout")]
mod enhanced;
#[cfg(feature = "ml-layout")]
mod feedback;
#[cfg(feature = "ml-layout")]
mod gnn;
#[cfg(feature = "ml-layout")]
mod rl;

#[cfg(all(test, feature = "ml-layout"))]
mod tests;

// Phase 1 exports
#[cfg(feature = "ml-layout")]
pub use features::{GraphFeatureExtractor, GraphFeatures};
#[cfg(feature = "ml-layout")]
pub use model::{LayoutPredictionModel, ModelType, PerformanceMetrics, QualityMetrics};
#[cfg(feature = "ml-layout")]
pub use selector::{MLStrategySelector, StrategyPrediction};
#[cfg(feature = "ml-layout")]
pub use training::{TrainingData, TrainingDataCollector};

// Phase 2 exports
#[cfg(feature = "ml-layout")]
pub use constraints::{
    Axis, ConstraintSolution, Direction, LayoutConstraint, NeuralConstraintSolver,
};
#[cfg(feature = "ml-layout")]
pub use enhanced::{EnhancedMLConfig, EnhancedMLLayoutBuilder, EnhancedMLLayoutStrategy};
#[cfg(feature = "ml-layout")]
pub use feedback::{FeedbackCollector, FeedbackSession, FeedbackType, OnlineModelUpdater};
#[cfg(feature = "ml-layout")]
pub use gnn::{GNNLayoutPredictor, LayoutPrediction};
#[cfg(feature = "ml-layout")]
pub use rl::{LayoutAction, LayoutEnvironment, LayoutState, RLLayoutOptimizer};

use crate::error::Result;
use crate::igr::IntermediateGraph;
use crate::layout::{LayoutContext, LayoutStrategy};
use std::sync::Arc;

/// ML-enhanced layout strategy that uses machine learning to select optimal layouts
#[cfg(feature = "ml-layout")]
pub struct MLLayoutStrategy {
    selector: Arc<MLStrategySelector>,
    fallback_strategy: Arc<dyn LayoutStrategy>,
}

#[cfg(feature = "ml-layout")]
impl MLLayoutStrategy {
    pub fn new(fallback_strategy: Arc<dyn LayoutStrategy>) -> Result<Self> {
        let selector = Arc::new(MLStrategySelector::new()?);
        Ok(Self {
            selector,
            fallback_strategy,
        })
    }

    pub fn with_model_path(
        model_path: &str,
        fallback_strategy: Arc<dyn LayoutStrategy>,
    ) -> Result<Self> {
        let selector = Arc::new(MLStrategySelector::from_path(model_path)?);
        Ok(Self {
            selector,
            fallback_strategy,
        })
    }
}

#[cfg(feature = "ml-layout")]
impl LayoutStrategy for MLLayoutStrategy {
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        // Try ML-based strategy selection
        match self.selector.select_strategy(igr, context) {
            Ok(prediction) => {
                log::info!(
                    "ML selected strategy: {} with confidence: {:.2}",
                    prediction.strategy_name,
                    prediction.confidence
                );

                // If confidence is high enough, use the selected strategy
                if prediction.confidence > 0.7 {
                    if let Some(strategy) = prediction.strategy {
                        return strategy.apply(igr, context);
                    }
                }

                // Otherwise fall back
                log::info!("ML confidence too low, using fallback strategy");
                self.fallback_strategy.apply(igr, context)
            }
            Err(e) => {
                log::warn!("ML strategy selection failed: {e}, using fallback");
                self.fallback_strategy.apply(igr, context)
            }
        }
    }

    fn name(&self) -> &'static str {
        "ml-enhanced"
    }

    fn supports(&self, igr: &IntermediateGraph) -> bool {
        // ML strategy supports all graphs
        self.fallback_strategy.supports(igr)
    }
}
