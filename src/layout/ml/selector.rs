// src/layout/ml/selector.rs
//! ML-driven strategy selector for intelligent layout algorithm selection

use crate::error::Result;
use crate::igr::IntermediateGraph;
use crate::layout::{
    AdaptiveStrategy, DagreLayout, ElkLayout, ForceLayout, LayoutContext, LayoutEngineAdapter,
    LayoutStrategy,
};
use std::path::Path;
use std::sync::Arc;

use super::{
    GraphFeatureExtractor, LayoutPredictionModel, ModelType, PerformanceMetrics, QualityMetrics,
};

/// Prediction result from the ML strategy selector
#[derive(Clone)]
pub struct StrategyPrediction {
    pub strategy_name: String,
    pub strategy: Option<Arc<dyn LayoutStrategy>>,
    pub confidence: f64,
    pub performance_estimate: PerformanceMetrics,
    pub quality_estimate: QualityMetrics,
}

/// ML-based strategy selector that uses machine learning to choose optimal layout algorithms
pub struct MLStrategySelector {
    feature_extractor: GraphFeatureExtractor,
    strategy_model: LayoutPredictionModel,
    performance_model: LayoutPredictionModel,
    quality_model: LayoutPredictionModel,
    available_strategies: Vec<(String, Arc<dyn LayoutStrategy>)>,
}

impl MLStrategySelector {
    /// Create a new ML strategy selector with default models
    pub fn new() -> Result<Self> {
        let feature_extractor = GraphFeatureExtractor::new();
        let strategy_model = LayoutPredictionModel::new(ModelType::StrategySelector)?;
        let performance_model = LayoutPredictionModel::new(ModelType::PerformancePredictor)?;
        let quality_model = LayoutPredictionModel::new(ModelType::QualityPredictor)?;

        let available_strategies = Self::create_default_strategies();

        Ok(Self {
            feature_extractor,
            strategy_model,
            performance_model,
            quality_model,
            available_strategies,
        })
    }

    /// Create a selector from pre-trained model files
    pub fn from_path(model_dir: &str) -> Result<Self> {
        let feature_extractor = GraphFeatureExtractor::new();

        // Try to load trained models, use the best available model for each prediction type
        // Use GNN model as our primary predictor since enhanced model is an ensemble config
        let strategy_path = Path::new(model_dir).join("gnn_model.bin");
        let performance_path = Path::new(model_dir).join("gnn_model.bin"); // GNN for performance prediction
        let quality_path = Path::new(model_dir).join("gnn_model.bin"); // GNN for quality prediction too

        let strategy_model = if strategy_path.exists() {
            LayoutPredictionModel::from_path(ModelType::StrategySelector, &strategy_path)?
        } else {
            // Fallback to default model
            log::warn!("Enhanced model not found, using default strategy selector");
            LayoutPredictionModel::new(ModelType::StrategySelector)?
        };

        let performance_model = if performance_path.exists() {
            LayoutPredictionModel::from_path(ModelType::PerformancePredictor, &performance_path)?
        } else {
            log::warn!("GNN model not found, using default performance predictor");
            LayoutPredictionModel::new(ModelType::PerformancePredictor)?
        };

        let quality_model = if quality_path.exists() {
            LayoutPredictionModel::from_path(ModelType::QualityPredictor, &quality_path)?
        } else {
            log::warn!("Enhanced model not found, using default quality predictor");
            LayoutPredictionModel::new(ModelType::QualityPredictor)?
        };

        let available_strategies = Self::create_default_strategies();

        Ok(Self {
            feature_extractor,
            strategy_model,
            performance_model,
            quality_model,
            available_strategies,
        })
    }

    /// Select the best strategy for the given graph
    pub fn select_strategy(
        &self,
        igr: &IntermediateGraph,
        _context: &LayoutContext,
    ) -> Result<StrategyPrediction> {
        // Extract features from the graph
        let features = self.feature_extractor.extract(igr)?;
        let feature_vector = features.to_vector();

        // Get strategy predictions
        let strategy_probs = self.strategy_model.predict(&feature_vector)?;

        // Find the best strategy, handling NaN values and applying heuristics
        let (mut best_idx, &confidence) = strategy_probs
            .iter()
            .enumerate()
            .filter(|(_, &val)| val.is_finite())
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| {
                crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                    "All strategy predictions are invalid (NaN or Inf)".to_string(),
                ))
            })?;

        let mut confidence = confidence; // Make it mutable for heuristic overrides

        // Apply heuristics to override poor ML predictions for complex graphs
        if features.node_count > 10 && features.edge_count > 15 {
            // For complex graphs, prefer ELK over adaptive strategy
            if let Some((elk_idx, _)) = self
                .available_strategies
                .iter()
                .enumerate()
                .find(|(_, (name, _))| name == "elk")
            {
                best_idx = elk_idx;
                confidence = 0.95; // High confidence for our heuristic choice
                log::debug!(
                    "Heuristic override: Complex graph ({} nodes, {} edges) -> ELK layout",
                    features.node_count,
                    features.edge_count
                );
            }
        } else if !features.is_dag && features.node_count > 5 {
            // For graphs with cycles, prefer Force layout
            if let Some((force_idx, _)) = self
                .available_strategies
                .iter()
                .enumerate()
                .find(|(_, (name, _))| name == "force")
            {
                best_idx = force_idx;
                confidence = 0.90; // High confidence for cycle handling
                log::debug!(
                    "Heuristic override: Graph with cycles ({} nodes) -> Force layout",
                    features.node_count
                );
            }
        }

        // Get performance predictions
        let performance_preds = self.performance_model.predict(&feature_vector)?;
        let performance_estimate = PerformanceMetrics {
            expected_time_ms: performance_preds[0].max(0.0),
            memory_usage_mb: performance_preds[1].max(0.0),
            cpu_utilization: performance_preds[2].clamp(0.0, 1.0),
        };

        // Get quality predictions
        let quality_preds = self.quality_model.predict(&feature_vector)?;
        let quality_estimate = QualityMetrics {
            edge_crossing_score: quality_preds[0],
            node_overlap_score: quality_preds[1],
            space_utilization: quality_preds[2],
            symmetry_score: quality_preds[3],
            aesthetic_score: quality_preds[4],
        };

        // Map index to strategy
        let (strategy_name, strategy) = if best_idx < self.available_strategies.len() {
            let (name, strat) = &self.available_strategies[best_idx];
            (name.clone(), Some(strat.clone()))
        } else {
            ("unknown".to_string(), None)
        };

        log::debug!(
            "ML Strategy Selection - Graph features: nodes={}, edges={}, density={:.3}, is_dag={}",
            features.node_count,
            features.edge_count,
            features.density,
            features.is_dag
        );

        log::debug!(
            "ML Predictions - Strategy: {} (conf={:.2}), Time: {:.1}ms, Quality: {:.2}",
            strategy_name,
            confidence,
            performance_estimate.expected_time_ms,
            quality_estimate.aesthetic_score
        );

        Ok(StrategyPrediction {
            strategy_name,
            strategy,
            confidence,
            performance_estimate,
            quality_estimate,
        })
    }

    /// Evaluate multiple strategies and return ranked predictions
    pub fn evaluate_all_strategies(
        &self,
        igr: &IntermediateGraph,
        _context: &LayoutContext,
    ) -> Result<Vec<StrategyPrediction>> {
        let features = self.feature_extractor.extract(igr)?;
        let feature_vector = features.to_vector();

        let strategy_probs = self.strategy_model.predict(&feature_vector)?;
        let performance_preds = self.performance_model.predict(&feature_vector)?;
        let quality_preds = self.quality_model.predict(&feature_vector)?;

        let performance_estimate = PerformanceMetrics {
            expected_time_ms: performance_preds[0].max(0.0),
            memory_usage_mb: performance_preds[1].max(0.0),
            cpu_utilization: performance_preds[2].clamp(0.0, 1.0),
        };

        let quality_estimate = QualityMetrics {
            edge_crossing_score: quality_preds[0],
            node_overlap_score: quality_preds[1],
            space_utilization: quality_preds[2],
            symmetry_score: quality_preds[3],
            aesthetic_score: quality_preds[4],
        };

        let mut predictions: Vec<StrategyPrediction> = self
            .available_strategies
            .iter()
            .enumerate()
            .map(|(idx, (name, strategy))| {
                let confidence = if idx < strategy_probs.len() {
                    strategy_probs[idx]
                } else {
                    0.0
                };

                StrategyPrediction {
                    strategy_name: name.clone(),
                    strategy: Some(strategy.clone()),
                    confidence,
                    performance_estimate: performance_estimate.clone(),
                    quality_estimate: quality_estimate.clone(),
                }
            })
            .collect();

        // Sort by confidence (descending), handling NaN values
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(predictions)
    }

    /// Create default strategies available for selection
    fn create_default_strategies() -> Vec<(String, Arc<dyn LayoutStrategy>)> {
        vec![
            (
                "dagre".to_string(),
                Arc::new(LayoutEngineAdapter::new(DagreLayout::new())) as Arc<dyn LayoutStrategy>,
            ),
            (
                "force".to_string(),
                Arc::new(LayoutEngineAdapter::new(ForceLayout::new())) as Arc<dyn LayoutStrategy>,
            ),
            (
                "elk".to_string(),
                Arc::new(LayoutEngineAdapter::new(ElkLayout::new())) as Arc<dyn LayoutStrategy>,
            ),
            (
                "adaptive".to_string(),
                Arc::new(
                    AdaptiveStrategy::new()
                        .add_strategy(Arc::new(LayoutEngineAdapter::new(ElkLayout::new())))
                        .add_strategy(Arc::new(LayoutEngineAdapter::new(ForceLayout::new())))
                        .add_strategy(Arc::new(LayoutEngineAdapter::new(DagreLayout::new()))),
                ) as Arc<dyn LayoutStrategy>,
            ),
        ]
    }

    /// Update models based on user feedback
    pub fn update_from_feedback(
        &mut self,
        igr: &IntermediateGraph,
        selected_strategy: &str,
        user_satisfaction: f64,
    ) -> Result<()> {
        // This would typically involve:
        // 1. Extract features from the graph
        // 2. Create training sample with actual outcome
        // 3. Update model weights using online learning
        // For now, we just log the feedback

        let features = self.feature_extractor.extract(igr)?;
        log::info!(
            "User feedback recorded - Strategy: {}, Satisfaction: {:.2}, Graph: {} nodes, {} edges",
            selected_strategy,
            user_satisfaction,
            features.node_count,
            features.edge_count
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap;

    #[test]
    fn test_strategy_selector_creation() {
        let selector = MLStrategySelector::new().unwrap();
        assert_eq!(selector.available_strategies.len(), 4);
    }

    #[test]
    fn test_strategy_selection() {
        let selector = MLStrategySelector::new().unwrap();

        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("A".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("B".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![EdgeDefinition {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
                style: None,
            }],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let context = LayoutContext::default();

        let prediction = selector.select_strategy(&igr, &context).unwrap();

        assert!(!prediction.strategy_name.is_empty());
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
        assert!(prediction.strategy.is_some());
    }
}
