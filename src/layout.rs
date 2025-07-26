// src/layout.rs
use crate::error::{LayoutError, Result};
use crate::igr::{BoundingBox, IntermediateGraph};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

pub trait LayoutEngine {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()>;
    fn name(&self) -> &'static str;
}

pub struct LayoutManager {
    engines: HashMap<String, Box<dyn LayoutEngine>>,
}

impl LayoutManager {
    pub fn new() -> Self {
        let mut manager = LayoutManager {
            engines: HashMap::new(),
        };

        // Register available layout engines
        manager.register("dagre", Box::new(DagreLayout::new()));
        manager.register("force", Box::new(ForceLayout::new()));

        manager
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

        engine.layout(igr)
    }
}

// Basic Dagre-like hierarchical layout
pub struct DagreLayout {
    options: DagreLayoutOptions,
}

#[derive(Debug, Clone)]
pub struct DagreLayoutOptions {
    pub node_sep: f64,
    pub rank_sep: f64,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl Default for DagreLayoutOptions {
    fn default() -> Self {
        Self {
            node_sep: 50.0,
            rank_sep: 100.0,
            direction: Direction::TopBottom,
        }
    }
}

impl DagreLayout {
    pub fn new() -> Self {
        Self {
            options: DagreLayoutOptions::default(),
        }
    }

    pub fn with_options(options: DagreLayoutOptions) -> Self {
        Self { options }
    }
}

impl LayoutEngine for DagreLayout {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        if igr.graph.node_count() == 0 {
            return Ok(());
        }

        // Simple layered layout algorithm
        let layers = self.assign_layers(igr)?;
        self.position_nodes(igr, &layers)?;
        self.calculate_container_bounds(igr);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "dagre"
    }
}

impl DagreLayout {
    fn assign_layers(&self, igr: &IntermediateGraph) -> Result<Vec<Vec<NodeIndex>>> {
        use petgraph::algo::toposort;
        use petgraph::Direction as PetDirection;

        // Perform topological sort to get a rough ordering
        let topo_order = toposort(&igr.graph, None).map_err(|_| LayoutError::InvalidGraph)?;

        let mut layers: Vec<Vec<NodeIndex>> = Vec::new();
        let mut node_to_layer: HashMap<NodeIndex, usize> = HashMap::new();

        // Assign nodes to layers based on their dependencies
        for node_idx in topo_order {
            let mut max_predecessor_layer = 0;

            // Find the maximum layer of all predecessors
            for edge in igr.graph.edges_directed(node_idx, PetDirection::Incoming) {
                let predecessor = edge.source();
                if let Some(&pred_layer) = node_to_layer.get(&predecessor) {
                    max_predecessor_layer = max_predecessor_layer.max(pred_layer + 1);
                }
            }

            // Ensure we have enough layers
            while layers.len() <= max_predecessor_layer {
                layers.push(Vec::new());
            }

            layers[max_predecessor_layer].push(node_idx);
            node_to_layer.insert(node_idx, max_predecessor_layer);
        }

        // If no edges (isolated nodes), put all nodes in layer 0
        if layers.is_empty() && !igr.graph.node_indices().collect::<Vec<_>>().is_empty() {
            layers.push(igr.graph.node_indices().collect());
        }

        Ok(layers)
    }

    fn position_nodes(&self, igr: &mut IntermediateGraph, layers: &[Vec<NodeIndex>]) -> Result<()> {
        let mut current_y = 0.0;

        for layer in layers {
            if layer.is_empty() {
                continue;
            }

            // Calculate total width needed for this layer
            let total_width: f64 = layer.iter().map(|&idx| igr.graph[idx].width).sum();
            let total_spacing = (layer.len().saturating_sub(1)) as f64 * self.options.node_sep;
            let layer_width = total_width + total_spacing;

            // Start positioning from the left
            let start_x = -layer_width / 2.0;
            let mut current_x = start_x;

            // Find the maximum height in this layer
            let max_height = layer
                .iter()
                .map(|&idx| igr.graph[idx].height)
                .fold(0.0, f64::max);

            // Position nodes in this layer
            for &node_idx in layer {
                let node = &mut igr.graph[node_idx];

                match self.options.direction {
                    Direction::TopBottom => {
                        node.x = current_x + node.width / 2.0;
                        node.y = current_y + max_height / 2.0;
                    }
                    Direction::BottomTop => {
                        node.x = current_x + node.width / 2.0;
                        node.y = -(current_y + max_height / 2.0);
                    }
                    Direction::LeftRight => {
                        node.x = current_y + max_height / 2.0;
                        node.y = current_x + node.width / 2.0;
                    }
                    Direction::RightLeft => {
                        node.x = -(current_y + max_height / 2.0);
                        node.y = current_x + node.width / 2.0;
                    }
                }

                current_x += node.width + self.options.node_sep;
            }

            current_y += max_height + self.options.rank_sep;
        }

        Ok(())
    }

    fn calculate_container_bounds(&self, igr: &mut IntermediateGraph) {
        for container in &mut igr.containers {
            if container.children.is_empty() {
                continue;
            }

            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for &child_idx in &container.children {
                let node = &igr.graph[child_idx];
                let node_min_x = node.x - node.width / 2.0;
                let node_max_x = node.x + node.width / 2.0;
                let node_min_y = node.y - node.height / 2.0;
                let node_max_y = node.y + node.height / 2.0;

                min_x = min_x.min(node_min_x);
                max_x = max_x.max(node_max_x);
                min_y = min_y.min(node_min_y);
                max_y = max_y.max(node_max_y);
            }

            // Add padding
            let padding = 20.0;
            container.bounds = Some(BoundingBox {
                x: min_x - padding,
                y: min_y - padding,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding,
            });
        }
    }
}

// Simple force-directed layout
pub struct ForceLayout {
    options: ForceLayoutOptions,
}

#[derive(Debug, Clone)]
pub struct ForceLayoutOptions {
    pub iterations: usize,
    pub repulsion_strength: f64,
    pub attraction_strength: f64,
    pub damping: f64,
}

impl Default for ForceLayoutOptions {
    fn default() -> Self {
        Self {
            iterations: 100,
            repulsion_strength: 1000.0,
            attraction_strength: 0.1,
            damping: 0.9,
        }
    }
}

impl ForceLayout {
    pub fn new() -> Self {
        Self {
            options: ForceLayoutOptions::default(),
        }
    }

    pub fn with_options(options: ForceLayoutOptions) -> Self {
        Self { options }
    }
}

impl LayoutEngine for ForceLayout {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        if igr.graph.node_count() == 0 {
            return Ok(());
        }

        // Initialize random positions
        self.initialize_positions(igr);

        // Run force simulation
        for _ in 0..self.options.iterations {
            self.apply_forces(igr);
        }

        self.calculate_container_bounds(igr);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "force"
    }
}

impl ForceLayout {
    fn initialize_positions(&self, igr: &mut IntermediateGraph) {
        use std::f64::consts::PI;

        let node_count = igr.graph.node_count();
        let radius = (node_count as f64).sqrt() * 50.0;

        for (i, node_idx) in igr.graph.node_indices().enumerate() {
            let angle = 2.0 * PI * i as f64 / node_count as f64;
            let node = &mut igr.graph[node_idx];
            node.x = radius * angle.cos();
            node.y = radius * angle.sin();
        }
    }

    fn apply_forces(&self, igr: &mut IntermediateGraph) {
        let mut velocities: HashMap<NodeIndex, (f64, f64)> = HashMap::new();

        // Initialize velocities
        for node_idx in igr.graph.node_indices() {
            velocities.insert(node_idx, (0.0, 0.0));
        }

        // Repulsion forces between all nodes
        let nodes: Vec<_> = igr.graph.node_indices().collect();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let node_i = nodes[i];
                let node_j = nodes[j];

                let pos_i = (igr.graph[node_i].x, igr.graph[node_i].y);
                let pos_j = (igr.graph[node_j].x, igr.graph[node_j].y);

                let dx = pos_i.0 - pos_j.0;
                let dy = pos_i.1 - pos_j.1;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                let force = self.options.repulsion_strength / (distance * distance);
                let fx = force * dx / distance;
                let fy = force * dy / distance;

                let vel_i = velocities.get_mut(&node_i).unwrap();
                vel_i.0 += fx;
                vel_i.1 += fy;

                let vel_j = velocities.get_mut(&node_j).unwrap();
                vel_j.0 -= fx;
                vel_j.1 -= fy;
            }
        }

        // Attraction forces along edges
        for edge in igr.graph.edge_indices() {
            let (source, target) = igr.graph.edge_endpoints(edge).unwrap();

            let pos_source = (igr.graph[source].x, igr.graph[source].y);
            let pos_target = (igr.graph[target].x, igr.graph[target].y);

            let dx = pos_target.0 - pos_source.0;
            let dy = pos_target.1 - pos_source.1;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);

            let force = self.options.attraction_strength * distance;
            let fx = force * dx / distance;
            let fy = force * dy / distance;

            let vel_source = velocities.get_mut(&source).unwrap();
            vel_source.0 += fx;
            vel_source.1 += fy;

            let vel_target = velocities.get_mut(&target).unwrap();
            vel_target.0 -= fx;
            vel_target.1 -= fy;
        }

        // Apply velocities with damping
        for node_idx in igr.graph.node_indices() {
            let (vx, vy) = velocities[&node_idx];
            let node = &mut igr.graph[node_idx];
            node.x += vx * self.options.damping;
            node.y += vy * self.options.damping;
        }
    }

    fn calculate_container_bounds(&self, igr: &mut IntermediateGraph) {
        // Same implementation as DagreLayout
        for container in &mut igr.containers {
            if container.children.is_empty() {
                continue;
            }

            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for &child_idx in &container.children {
                let node = &igr.graph[child_idx];
                let node_min_x = node.x - node.width / 2.0;
                let node_max_x = node.x + node.width / 2.0;
                let node_min_y = node.y - node.height / 2.0;
                let node_max_y = node.y + node.height / 2.0;

                min_x = min_x.min(node_min_x);
                max_x = max_x.max(node_max_x);
                min_y = min_y.min(node_min_y);
                max_y = max_y.max(node_max_y);
            }

            // Add padding
            let padding = 20.0;
            container.bounds = Some(BoundingBox {
                x: min_x - padding,
                y: min_y - padding,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding,
            });
        }
    }
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
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("A".to_string()),
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("B".to_string()),
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![EdgeDefinition {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: HashMap::new(),
            }],
            containers: vec![],
        };

        let mut igr = IntermediateGraph::from_ast(document).unwrap();
        let layout_manager = LayoutManager::new();

        layout_manager.layout(&mut igr).unwrap();

        // Check that nodes have been positioned
        let (_, node_a) = igr.get_node_by_id("a").unwrap();
        let (_, node_b) = igr.get_node_by_id("b").unwrap();

        // In a top-bottom layout, B should be below A
        assert!(node_b.y > node_a.y);
    }
}
