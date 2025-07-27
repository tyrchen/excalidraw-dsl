// src/layout/force.rs
use super::LayoutEngine;
use crate::error::Result;
use crate::igr::{BoundingBox, ContainerData, EdgeData, IntermediateGraph, NodeData};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

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
            iterations: 200,
            repulsion_strength: 5000.0,
            attraction_strength: 0.05,
            damping: 0.85,
        }
    }
}

impl Default for ForceLayout {
    fn default() -> Self {
        Self::new()
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
        // Increase radius for better initial spacing
        let radius = (node_count as f64).sqrt() * 100.0;

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

                // Add minimum distance based on node sizes
                let min_distance = (igr.graph[node_i].width + igr.graph[node_j].width) / 2.0 + 50.0;
                let effective_distance = distance.max(min_distance * 0.1); // Prevent division by very small numbers

                let force =
                    self.options.repulsion_strength / (effective_distance * effective_distance);
                let fx = force * dx / effective_distance;
                let fy = force * dy / effective_distance;

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
        // Calculate bounds in reverse order to handle nested containers
        // Process children before parents
        let mut processed = vec![false; igr.containers.len()];

        // Helper to calculate bounds for a single container
        fn calculate_single_container_bounds(
            idx: usize,
            containers: &mut Vec<ContainerData>,
            graph: &petgraph::Graph<NodeData, EdgeData>,
            processed: &mut Vec<bool>,
        ) {
            if processed[idx] {
                return;
            }

            // First process all nested containers
            let nested_indices = containers[idx].nested_containers.clone();
            for &nested_idx in &nested_indices {
                calculate_single_container_bounds(nested_idx, containers, graph, processed);
            }

            // Collect data we need before mutating
            let children = containers[idx].children.clone();
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            // Include bounds of child nodes
            for &child_idx in &children {
                let node = &graph[child_idx];
                let node_min_x = node.x - node.width / 2.0;
                let node_max_x = node.x + node.width / 2.0;
                let node_min_y = node.y - node.height / 2.0;
                let node_max_y = node.y + node.height / 2.0;

                min_x = min_x.min(node_min_x);
                max_x = max_x.max(node_max_x);
                min_y = min_y.min(node_min_y);
                max_y = max_y.max(node_max_y);
            }

            // Include bounds of nested containers
            for &nested_idx in &nested_indices {
                if let Some(ref nested_bounds) = containers[nested_idx].bounds {
                    min_x = min_x.min(nested_bounds.x);
                    max_x = max_x.max(nested_bounds.x + nested_bounds.width);
                    min_y = min_y.min(nested_bounds.y);
                    max_y = max_y.max(nested_bounds.y + nested_bounds.height);
                }
            }

            // Only set bounds if we found any content
            if min_x != f64::INFINITY {
                // Add padding
                let padding = 20.0;
                containers[idx].bounds = Some(BoundingBox {
                    x: min_x - padding,
                    y: min_y - padding,
                    width: (max_x - min_x) + 2.0 * padding,
                    height: (max_y - min_y) + 2.0 * padding,
                });
            }

            processed[idx] = true;
        }

        // Process all root containers (those without parents)
        for i in 0..igr.containers.len() {
            if igr.containers[i].parent_container.is_none() {
                calculate_single_container_bounds(
                    i,
                    &mut igr.containers,
                    &igr.graph,
                    &mut processed,
                );
            }
        }
    }
}
