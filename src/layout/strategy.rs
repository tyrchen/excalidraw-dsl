// src/layout/strategy.rs
use crate::error::Result;
use crate::igr::IntermediateGraph;
use std::sync::Arc;

/// Context passed to layout strategies containing relevant information
#[derive(Clone)]
pub struct LayoutContext {
    /// Maximum width for the layout area
    pub max_width: Option<f64>,
    /// Maximum height for the layout area
    pub max_height: Option<f64>,
    /// Node spacing preferences
    pub node_spacing: f64,
    /// Edge spacing preferences
    pub edge_spacing: f64,
    /// Whether to optimize for readability
    pub optimize_readability: bool,
    /// Custom parameters for specific strategies
    pub custom_params: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self {
            max_width: None,
            max_height: None,
            node_spacing: 100.0,
            edge_spacing: 50.0,
            optimize_readability: true,
            custom_params: std::collections::HashMap::new(),
        }
    }
}

/// Strategy trait for different layout algorithms
pub trait LayoutStrategy: Send + Sync {
    /// Apply the layout strategy to the graph
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()>;

    /// Get the name of this strategy
    fn name(&self) -> &'static str;

    /// Check if this strategy supports the given graph structure
    fn supports(&self, _igr: &IntermediateGraph) -> bool {
        // By default, all strategies support all graphs
        true
    }

    /// Get a hint about the computational complexity
    fn complexity_hint(&self) -> ComplexityHint {
        ComplexityHint::Medium
    }
}

/// Hint about the computational complexity of a layout strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityHint {
    /// Very fast, suitable for real-time updates
    Low,
    /// Moderate speed, suitable for most interactive use cases
    Medium,
    /// Slower, may need progress indication for large graphs
    High,
    /// Very slow, should be run asynchronously with progress updates
    VeryHigh,
}

/// Composite strategy that tries multiple strategies in order
pub struct CompositeStrategy {
    strategies: Vec<Arc<dyn LayoutStrategy>>,
    fallback: Arc<dyn LayoutStrategy>,
}

impl CompositeStrategy {
    pub fn new(fallback: Arc<dyn LayoutStrategy>) -> Self {
        Self {
            strategies: Vec::new(),
            fallback,
        }
    }

    pub fn add_strategy(mut self, strategy: Arc<dyn LayoutStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }
}

impl LayoutStrategy for CompositeStrategy {
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        // Try each strategy in order
        for strategy in &self.strategies {
            if strategy.supports(igr) {
                match strategy.apply(igr, context) {
                    Ok(()) => return Ok(()),
                    Err(_) => continue, // Try next strategy
                }
            }
        }

        // Fall back to the default strategy
        self.fallback.apply(igr, context)
    }

    fn name(&self) -> &'static str {
        "composite"
    }
}

/// Strategy selector that chooses the best strategy based on graph characteristics
pub struct AdaptiveStrategy {
    strategies: Vec<Arc<dyn LayoutStrategy>>,
}

impl Default for AdaptiveStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveStrategy {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(mut self, strategy: Arc<dyn LayoutStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Analyze the graph and select the best strategy
    fn select_strategy(&self, igr: &IntermediateGraph) -> Option<&Arc<dyn LayoutStrategy>> {
        let node_count = igr.graph.node_count();
        let edge_count = igr.graph.edge_count();
        let has_containers = !igr.containers.is_empty();
        let has_groups = !igr.groups.is_empty();

        // Heuristics for strategy selection
        let is_hierarchical = has_containers || has_groups;
        let is_dense = edge_count > node_count * 2;
        let is_large = node_count > 100;

        self.strategies
            .iter()
            .find(|strategy| match strategy.name() {
                "dagre" => is_hierarchical || !is_dense,
                "force" => !is_hierarchical && is_dense && !is_large,
                "elk" => is_hierarchical && is_large,
                _ => true,
            })
    }
}

impl LayoutStrategy for AdaptiveStrategy {
    fn apply(&self, igr: &mut IntermediateGraph, context: &LayoutContext) -> Result<()> {
        if let Some(strategy) = self.select_strategy(igr) {
            strategy.apply(igr, context)
        } else {
            Err(crate::error::EDSLError::Layout(
                crate::error::LayoutError::CalculationFailed(
                    "No suitable layout strategy found".to_string(),
                ),
            ))
        }
    }

    fn name(&self) -> &'static str {
        "adaptive"
    }
}
