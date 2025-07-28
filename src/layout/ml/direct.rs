// src/layout/ml/direct.rs
//! Direct ML Layout - uses GNN to predict node positions directly instead of delegating to other engines

use crate::error::Result;
use crate::igr::IntermediateGraph;
use crate::layout::{LayoutContext, LayoutStrategy};
use std::path::Path;
use std::sync::Arc;

use super::GNNLayoutPredictor;

/// True ML Layout that predicts node positions directly using Graph Neural Networks
#[cfg(feature = "ml-layout")]
pub struct DirectMLLayoutStrategy {
    gnn_predictor: GNNLayoutPredictor,
    fallback_strategy: Arc<dyn LayoutStrategy>,
}

#[cfg(feature = "ml-layout")]
impl DirectMLLayoutStrategy {
    /// Create a new direct ML layout with trained GNN model
    pub fn new(fallback_strategy: Arc<dyn LayoutStrategy>) -> Result<Self> {
        // Try to load trained GNN model
        let gnn_predictor = if std::path::Path::new("./models/models/gnn_model.bin").exists() {
            log::info!("Loading trained GNN model from ./models/models/gnn_model.bin");
            match GNNLayoutPredictor::from_path(Path::new("./models/models/gnn_model.bin")) {
                Ok(predictor) => {
                    log::info!("Successfully loaded trained GNN model");
                    predictor
                }
                Err(e) => {
                    log::warn!("Failed to load trained GNN model: {e}, using default");
                    GNNLayoutPredictor::new(32, 64, 3)?
                }
            }
        } else {
            log::info!("No trained GNN model found, using default architecture");
            GNNLayoutPredictor::new(32, 64, 3)?
        };

        Ok(Self {
            gnn_predictor,
            fallback_strategy,
        })
    }

    /// Create with specific model path
    pub fn with_model_path(
        model_path: &Path,
        fallback_strategy: Arc<dyn LayoutStrategy>,
    ) -> Result<Self> {
        let gnn_predictor = GNNLayoutPredictor::from_path(model_path)?;
        log::info!("Loaded GNN model from {}", model_path.display());

        Ok(Self {
            gnn_predictor,
            fallback_strategy,
        })
    }
}

#[cfg(feature = "ml-layout")]
impl LayoutStrategy for DirectMLLayoutStrategy {
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        let node_count = igr.graph.node_count();
        let edge_count = igr.graph.edge_count();

        log::info!(
            "Direct ML Layout: Predicting positions for {node_count} nodes, {edge_count} edges using GNN"
        );

        // Try GNN-based position prediction
        match self.gnn_predictor.predict_layout(igr) {
            Ok(prediction) => {
                if prediction.confidence > 0.3 {
                    log::info!(
                        "GNN prediction successful with confidence: {:.3}",
                        prediction.confidence
                    );

                    // Apply predicted positions to the graph
                    for (node_idx, &(x, y)) in prediction.positions.iter() {
                        if let Some(node) = igr.graph.node_weight_mut(*node_idx) {
                            // Scale positions to reasonable canvas size (Excalidraw units)
                            node.x = x * 100.0; // Scale up from normalized coordinates
                            node.y = y * 100.0;

                            log::debug!(
                                "Node {} positioned at ({:.1}, {:.1})",
                                node.id,
                                node.x,
                                node.y
                            );
                        }
                    }

                    log::info!("✅ Direct ML layout completed successfully");
                    return Ok(());
                } else {
                    log::warn!(
                        "GNN confidence ({:.3}) too low, falling back to traditional layout",
                        prediction.confidence
                    );
                }
            }
            Err(e) => {
                log::warn!("GNN prediction failed: {e}, using fallback strategy");
            }
        }

        // Fallback to traditional layout
        log::info!("Using fallback strategy: {}", self.fallback_strategy.name());
        self.fallback_strategy.apply(igr, context)
    }

    fn name(&self) -> &'static str {
        "direct-ml"
    }

    fn supports(&self, igr: &IntermediateGraph) -> bool {
        // Direct ML layout supports any graph size
        let node_count = igr.graph.node_count();

        // More effective for medium to large graphs
        if (3..=100).contains(&node_count) {
            true
        } else {
            // For very small or very large graphs, use fallback
            self.fallback_strategy.supports(igr)
        }
    }
}
