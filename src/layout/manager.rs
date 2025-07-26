// src/layout/manager.rs
use super::{CachedLayout, DagreLayout, ElkLayout, ForceLayout, LayoutCacheKey, LayoutEngine};
use crate::error::{LayoutError, Result};
use crate::igr::IntermediateGraph;
use petgraph::visit::IntoNodeReferences;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct LayoutManager {
    engines: HashMap<String, Box<dyn LayoutEngine>>,
    cache: Mutex<HashMap<LayoutCacheKey, CachedLayout>>,
    cache_enabled: bool,
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
        };

        // Register available layout engines
        manager.register("dagre", Box::new(DagreLayout::new()));
        manager.register("force", Box::new(ForceLayout::new()));
        manager.register("elk", Box::new(ElkLayout::new()));

        manager
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
}
