// src/layout/manager.rs
#[cfg(feature = "ml-layout")]
use super::{AdaptiveStrategy, LayoutEngineAdapter, LayoutStrategy, MLLayoutStrategy};
use super::{CachedLayout, DagreLayout, ElkLayout, ForceLayout, LayoutCacheKey, LayoutEngine};
use crate::error::{LayoutError, Result};
use crate::igr::IntermediateGraph;
use petgraph::visit::IntoNodeReferences;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct LayoutManager {
    engines: HashMap<String, Box<dyn LayoutEngine>>,
    cache: Mutex<HashMap<LayoutCacheKey, CachedLayout>>,
    cache_enabled: bool,
    parallel_enabled: bool,
    thread_pool: Option<Arc<rayon::ThreadPool>>,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        let mut manager = LayoutManager {
            engines: HashMap::new(),
            cache: Mutex::new(HashMap::new()),
            cache_enabled: true,
            parallel_enabled: true,
            thread_pool: None,
        };

        // Register available layout engines
        manager.register("dagre", Box::new(DagreLayout::new()));
        manager.register("force", Box::new(ForceLayout::new()));
        manager.register("elk", Box::new(ElkLayout::new()));

        // Register ML layout if feature is enabled
        #[cfg(feature = "ml-layout")]
        {
            // Create an adaptive strategy as fallback for ML
            let adaptive_fallback = Arc::new(
                AdaptiveStrategy::new()
                    .add_strategy(Arc::new(LayoutEngineAdapter::new(DagreLayout::new())))
                    .add_strategy(Arc::new(LayoutEngineAdapter::new(ForceLayout::new())))
                    .add_strategy(Arc::new(LayoutEngineAdapter::new(ElkLayout::new()))),
            );

            // Create ML layout adapter
            if let Ok(ml_strategy) = MLLayoutStrategy::new(adaptive_fallback) {
                let ml_strategy_arc = Arc::new(ml_strategy);
                manager.register("ml", Box::new(MLLayoutEngine(ml_strategy_arc.clone())));
                manager.register("ml-enhanced", Box::new(MLLayoutEngine(ml_strategy_arc)));
            }
        }

        manager
    }

    /// Create a layout manager with custom thread pool
    pub fn with_thread_pool(mut self, pool: Arc<rayon::ThreadPool>) -> Self {
        self.thread_pool = Some(pool);
        self
    }

    /// Enable or disable parallel processing
    pub fn enable_parallel(&mut self, enabled: bool) {
        self.parallel_enabled = enabled;
    }

    pub fn enable_cache(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    pub fn register(&mut self, name: &str, engine: Box<dyn LayoutEngine>) {
        self.engines.insert(name.to_string(), engine);
    }

    pub fn layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        let layout_name = igr.global_config.layout.as_deref().unwrap_or("dagre");

        let engine = self
            .engines
            .get(layout_name)
            .ok_or_else(|| LayoutError::UnknownEngine(layout_name.to_string()))?;

        // Check cache if enabled
        if self.cache_enabled {
            let cache_key = LayoutCacheKey::from_igr(igr, layout_name);

            // Try to get from cache
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_layout) = cache.get(&cache_key) {
                    // Apply cached positions
                    let updates: Vec<_> = igr
                        .graph
                        .node_references()
                        .filter_map(|(node_idx, node_data)| {
                            cached_layout
                                .positions
                                .get(&node_data.id)
                                .map(|&(x, y)| (node_idx, x, y))
                        })
                        .collect();

                    // Apply updates after collecting
                    for (node_idx, x, y) in updates {
                        igr.graph[node_idx].x = x;
                        igr.graph[node_idx].y = y;
                    }
                    return Ok(());
                }
            }

            // Not in cache, compute layout
            engine.layout(igr)?;

            // Store in cache
            if let Ok(mut cache) = self.cache.lock() {
                let mut positions = HashMap::new();
                for (_, node_data) in igr.graph.node_references() {
                    positions.insert(node_data.id.clone(), (node_data.x, node_data.y));
                }

                // Simple LRU: remove oldest if cache is too large
                if cache.len() > 100 {
                    // Remove a random entry (simple eviction)
                    if let Some(key) = cache.keys().next().cloned() {
                        cache.remove(&key);
                    }
                }

                cache.insert(cache_key, CachedLayout { positions });
            }

            Ok(())
        } else {
            engine.layout(igr)
        }
    }

    /// Layout with parallel processing for subgraphs if enabled
    pub fn layout_parallel(&self, igr: &mut IntermediateGraph) -> Result<()> {
        if !self.parallel_enabled || igr.containers.is_empty() {
            // Fall back to regular layout if parallel is disabled or no containers
            return self.layout(igr);
        }

        let layout_name = igr.global_config.layout.as_deref().unwrap_or("dagre");
        let engine = self
            .engines
            .get(layout_name)
            .ok_or_else(|| LayoutError::UnknownEngine(layout_name.to_string()))?;

        // Pre-allocate vectors for parallel processing
        let container_count = igr.containers.len();
        let mut container_layouts: Vec<Option<HashMap<String, (f64, f64)>>> =
            vec![None; container_count];

        // Process containers in parallel
        if let Some(pool) = &self.thread_pool {
            pool.install(|| {
                container_layouts
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(_idx, layout_opt)| {
                        // Create a subgraph for this container
                        // Note: This is a simplified example - real implementation would need
                        // to properly extract subgraphs
                        *layout_opt = Some(HashMap::new());
                    });
            });
        } else {
            // Use default thread pool
            container_layouts
                .par_iter_mut()
                .enumerate()
                .for_each(|(_idx, layout_opt)| {
                    *layout_opt = Some(HashMap::new());
                });
        }

        // Apply the main layout
        engine.layout(igr)?;

        Ok(())
    }

    /// Get statistics about cache usage
    pub fn cache_stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.lock() {
            CacheStats {
                entries: cache.len(),
                max_entries: 100,
                hit_rate: 0.0, // Would need to track hits/misses for real stats
            }
        } else {
            CacheStats::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hit_rate: f64,
}

/// Adapter to use ML layout strategy as a layout engine
#[cfg(feature = "ml-layout")]
struct MLLayoutEngine(Arc<dyn LayoutStrategy>);

#[cfg(feature = "ml-layout")]
impl LayoutEngine for MLLayoutEngine {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        use super::LayoutContext;
        let context = LayoutContext::default();
        self.0.apply(igr, &context)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}
