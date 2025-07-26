// src/layout/mod.rs
mod cache;
mod dagre;
mod elk;
mod force;
mod manager;

pub use cache::{CachedLayout, LayoutCacheKey};
pub use dagre::{DagreLayout, DagreLayoutOptions, Direction, RankingAlgorithm};
pub use elk::{ElkAlgorithm, ElkDirection, ElkLayout, ElkLayoutOptions, HierarchyHandling};
pub use force::{ForceLayout, ForceLayoutOptions};
pub use manager::LayoutManager;

use crate::error::Result;
use crate::igr::IntermediateGraph;

pub trait LayoutEngine: Send + Sync {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()>;
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::igr::IntermediateGraph;
    use std::collections::HashMap;

    #[test]
    fn test_dagre_layout_simple() {
        let document = ParsedDocument {
            config: GlobalConfig {
                layout: Some("dagre".to_string()),
                ..Default::default()
            },
            component_types: HashMap::new(),
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

        let mut igr = IntermediateGraph::from_ast(document).unwrap();
        let layout_manager = LayoutManager::new();

        layout_manager.layout(&mut igr).unwrap();

        // Check that nodes have been positioned
        let (_, node_a) = igr.get_node_by_id("a").unwrap();
        let (_, node_b) = igr.get_node_by_id("b").unwrap();

        // In a left-right layout, B should be to the right of A
        assert!(node_b.x > node_a.x);
    }
}
