// src/layout/cache.rs
use crate::igr::IntermediateGraph;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Cache key for layout results
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LayoutCacheKey {
    pub graph_hash: u64,
    pub engine: String,
}

impl LayoutCacheKey {
    pub fn from_igr(igr: &IntermediateGraph, engine: &str) -> Self {
        let mut hasher = DefaultHasher::new();

        // Hash nodes
        let mut node_ids: Vec<_> = igr
            .graph
            .node_indices()
            .map(|idx| &igr.graph[idx].id)
            .collect();
        node_ids.sort();

        for id in &node_ids {
            id.hash(&mut hasher);
        }

        // Hash edges
        let mut edge_pairs: Vec<_> = igr
            .graph
            .edge_indices()
            .map(|idx| {
                let (source, target) = igr.graph.edge_endpoints(idx).unwrap();
                (&igr.graph[source].id, &igr.graph[target].id)
            })
            .collect();
        edge_pairs.sort();

        for (source, target) in &edge_pairs {
            source.hash(&mut hasher);
            target.hash(&mut hasher);
        }

        Self {
            graph_hash: hasher.finish(),
            engine: engine.to_string(),
        }
    }
}

/// Cached layout positions
#[derive(Clone, Debug)]
pub struct CachedLayout {
    pub positions: HashMap<String, (f64, f64)>,
}
