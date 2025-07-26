// src/layout/elk.rs
use super::LayoutEngine;
use crate::ast::GroupType;
use crate::error::Result;
use crate::igr::{BoundingBox, IntermediateGraph};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction as PetDirection;
use std::collections::{HashMap, HashSet};

// ELK Layout Engine - Hierarchical and layered layout algorithm
pub struct ElkLayout {
    options: ElkLayoutOptions,
}

#[derive(Debug, Clone)]
pub struct ElkLayoutOptions {
    pub algorithm: ElkAlgorithm,
    pub spacing_node_node: f64,
    pub spacing_edge_node: f64,
    pub spacing_edge_edge: f64,
    pub direction: ElkDirection,
    pub hierarchy_handling: HierarchyHandling,
}

#[derive(Debug, Clone)]
pub enum ElkAlgorithm {
    Layered,
    Stress,
    Force,
    Tree,
}

#[derive(Debug, Clone)]
pub enum ElkDirection {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Debug, Clone)]
pub enum HierarchyHandling {
    IncludeChildren,
    SeparateChildren,
    IgnoreChildren,
}

impl Default for ElkLayoutOptions {
    fn default() -> Self {
        Self {
            algorithm: ElkAlgorithm::Layered,
            spacing_node_node: 20.0,
            spacing_edge_node: 12.0,
            spacing_edge_edge: 10.0,
            direction: ElkDirection::Right,
            hierarchy_handling: HierarchyHandling::IncludeChildren,
        }
    }
}

impl Default for ElkLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl ElkLayout {
    pub fn new() -> Self {
        Self {
            options: ElkLayoutOptions::default(),
        }
    }

    pub fn with_options(options: ElkLayoutOptions) -> Self {
        Self { options }
    }
}

impl LayoutEngine for ElkLayout {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        if igr.graph.node_count() == 0 {
            return Ok(());
        }

        match self.options.algorithm {
            ElkAlgorithm::Layered => self.layered_layout(igr)?,
            ElkAlgorithm::Stress => self.stress_layout(igr)?,
            ElkAlgorithm::Force => self.force_layout(igr)?,
            ElkAlgorithm::Tree => self.tree_layout(igr)?,
        }

        // Calculate bounds for containers and groups
        self.calculate_container_bounds(igr);
        self.calculate_group_bounds(igr);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "elk"
    }
}

impl ElkLayout {
    fn layered_layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Enhanced layered layout with ELK-style improvements
        let mut layers = self.build_layers_elk(igr)?;

        // Apply ELK-style crossing minimization
        self.minimize_crossings_elk(igr, &mut layers);

        // Position nodes with ELK spacing
        self.position_nodes_elk(igr, &layers)?;

        Ok(())
    }

    fn stress_layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Stress minimization layout - positions nodes to minimize stress
        let iterations = 200;
        let cooling_factor: f64 = 0.95;

        // Initialize with circular layout
        self.initialize_circular(igr);

        for iteration in 0..iterations {
            let temperature = 1.0 * cooling_factor.powi(iteration);
            self.apply_stress_forces(igr, temperature);
        }

        // Ensure all positions are non-negative
        self.normalize_positions(igr);

        Ok(())
    }

    fn force_layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Enhanced force-directed layout with ELK parameters
        let iterations = 300;
        let initial_temp = 200.0; // Increased initial temperature for better spread

        self.initialize_random(igr);

        for i in 0..iterations {
            let temperature = initial_temp * (1.0 - i as f64 / iterations as f64);
            self.apply_elk_forces(igr, temperature);
        }

        // Ensure all positions are non-negative
        self.normalize_positions(igr);

        Ok(())
    }

    fn tree_layout(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Tree layout for hierarchical structures
        let roots = self.find_roots(igr);

        if roots.is_empty() {
            // No clear hierarchy, fallback to layered
            return self.layered_layout(igr);
        }

        // Layout each tree with proper spacing
        let mut current_x = 0.0;
        for root in roots {
            let tree_width = self.layout_tree_recursive(igr, root, current_x, 50.0, 0)?;
            current_x += tree_width + 100.0; // Space between trees
        }

        Ok(())
    }

    fn build_layers_elk(&self, igr: &IntermediateGraph) -> Result<Vec<Vec<NodeIndex>>> {
        // Enhanced layering with ELK algorithms
        let node_ranks = self.calculate_node_ranks_elk(igr)?;
        let mut layers_map: HashMap<i32, Vec<NodeIndex>> = HashMap::new();

        for (node, &rank) in node_ranks.iter() {
            layers_map.entry(rank).or_insert_with(Vec::new).push(*node);
        }

        let mut sorted_ranks: Vec<i32> = layers_map.keys().copied().collect();
        sorted_ranks.sort();

        Ok(sorted_ranks
            .into_iter()
            .map(|rank| layers_map.remove(&rank).unwrap())
            .collect())
    }

    fn calculate_node_ranks_elk(&self, igr: &IntermediateGraph) -> Result<HashMap<NodeIndex, i32>> {
        // ELK-style ranking with network simplex algorithm approximation
        let mut ranks = HashMap::new();
        let mut visited = HashSet::new();

        // Find sources (nodes with no incoming edges)
        let sources: Vec<NodeIndex> = igr
            .graph
            .node_indices()
            .filter(|&node| {
                igr.graph
                    .edges_directed(node, PetDirection::Incoming)
                    .count()
                    == 0
            })
            .collect();

        // Start ranking from sources
        for source in sources {
            self.rank_from_source_elk(igr, source, 0, &mut ranks, &mut visited);
        }

        // Handle remaining nodes (in case of cycles)
        for node in igr.graph.node_indices() {
            ranks.entry(node).or_insert(0);
        }

        Ok(ranks)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn rank_from_source_elk(
        &self,
        igr: &IntermediateGraph,
        node: NodeIndex,
        rank: i32,
        ranks: &mut HashMap<NodeIndex, i32>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        if visited.contains(&node) {
            return;
        }

        visited.insert(node);
        ranks.insert(node, rank);

        // Recursively rank successors
        for edge in igr.graph.edges_directed(node, PetDirection::Outgoing) {
            let target = edge.target();
            let new_rank = rank + 1;

            // Update rank if this path gives a longer rank
            if let Some(&existing_rank) = ranks.get(&target) {
                if new_rank > existing_rank {
                    self.rank_from_source_elk(igr, target, new_rank, ranks, visited);
                }
            } else {
                self.rank_from_source_elk(igr, target, new_rank, ranks, visited);
            }
        }
    }

    fn minimize_crossings_elk(&self, igr: &IntermediateGraph, layers: &mut [Vec<NodeIndex>]) {
        // ELK-style crossing minimization with multiple passes
        for _ in 0..8 {
            // Forward pass
            for i in 1..layers.len() {
                let (before, after) = layers.split_at_mut(i);
                let reference_layer = &before[i - 1];
                let current_layer = &mut after[0];
                self.reorder_layer_elk(igr, current_layer, reference_layer, true);
            }

            // Backward pass
            for i in (0..layers.len() - 1).rev() {
                let (before, after) = layers.split_at_mut(i + 1);
                let current_layer = &mut before[i];
                let reference_layer = &after[0];
                self.reorder_layer_elk(igr, current_layer, reference_layer, false);
            }
        }
    }

    fn reorder_layer_elk(
        &self,
        igr: &IntermediateGraph,
        layer: &mut [NodeIndex],
        fixed_layer: &[NodeIndex],
        forward: bool,
    ) {
        // Calculate positions of nodes in fixed layer
        let positions: HashMap<NodeIndex, f64> = fixed_layer
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i as f64))
            .collect();

        // Calculate barycenter for each node in current layer
        let mut node_barycenters: Vec<(NodeIndex, f64)> = layer
            .iter()
            .map(|&node| {
                let connected_nodes: Vec<NodeIndex> = if forward {
                    igr.graph
                        .edges_directed(node, PetDirection::Incoming)
                        .map(|e| e.source())
                        .collect()
                } else {
                    igr.graph
                        .edges_directed(node, PetDirection::Outgoing)
                        .map(|e| e.target())
                        .collect()
                };

                let barycenter = if connected_nodes.is_empty() {
                    // No connections, use current position
                    layer.iter().position(|&n| n == node).unwrap() as f64
                } else {
                    // Calculate average position of connected nodes
                    let sum: f64 = connected_nodes
                        .iter()
                        .filter_map(|&n| positions.get(&n))
                        .sum();
                    sum / connected_nodes.len() as f64
                };

                (node, barycenter)
            })
            .collect();

        // Sort by barycenter
        node_barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Update layer order
        for (i, (node, _)) in node_barycenters.iter().enumerate() {
            layer[i] = *node;
        }
    }

    fn position_nodes_elk(
        &self,
        igr: &mut IntermediateGraph,
        layers: &[Vec<NodeIndex>],
    ) -> Result<()> {
        let layer_spacing = 150.0;
        let node_spacing = self.options.spacing_node_node;

        // Position layers
        for (layer_idx, layer) in layers.iter().enumerate() {
            let x = layer_idx as f64 * layer_spacing;

            // Calculate total height needed for this layer
            let total_height: f64 = layer.iter().map(|&idx| igr.graph[idx].height).sum::<f64>()
                + (layer.len().saturating_sub(1)) as f64 * node_spacing;

            let mut y = -total_height / 2.0;

            for &node_idx in layer {
                let node = &mut igr.graph[node_idx];

                match self.options.direction {
                    ElkDirection::Right => {
                        node.x = x;
                        node.y = y + node.height / 2.0;
                    }
                    ElkDirection::Down => {
                        node.x = y + node.width / 2.0;
                        node.y = x;
                    }
                    ElkDirection::Left => {
                        node.x = -x;
                        node.y = y + node.height / 2.0;
                    }
                    ElkDirection::Up => {
                        node.x = y + node.width / 2.0;
                        node.y = -x;
                    }
                }

                y += node.height + node_spacing;
            }
        }

        Ok(())
    }

    fn initialize_circular(&self, igr: &mut IntermediateGraph) {
        use std::f64::consts::PI;
        let node_count = igr.graph.node_count();
        let radius = (node_count as f64 * 30.0).max(100.0);

        for (i, node_idx) in igr.graph.node_indices().enumerate() {
            let angle = 2.0 * PI * i as f64 / node_count as f64;
            let node = &mut igr.graph[node_idx];
            node.x = radius * angle.cos();
            node.y = radius * angle.sin();
        }
    }

    fn initialize_random(&self, igr: &mut IntermediateGraph) {
        let bounds = 200.0;
        for node_idx in igr.graph.node_indices() {
            let node = &mut igr.graph[node_idx];
            node.x = (rand::random::<f64>() - 0.5) * bounds;
            node.y = (rand::random::<f64>() - 0.5) * bounds;
        }
    }

    fn apply_stress_forces(&self, igr: &mut IntermediateGraph, temperature: f64) {
        let nodes: Vec<NodeIndex> = igr.graph.node_indices().collect();
        let mut displacements: HashMap<NodeIndex, (f64, f64)> = HashMap::new();

        // Calculate ideal distances (shortest path distances)
        let ideal_distances = self.calculate_shortest_paths(igr);

        for &node_i in &nodes {
            let mut dx = 0.0;
            let mut dy = 0.0;
            let pos_i = (igr.graph[node_i].x, igr.graph[node_i].y);

            for &node_j in &nodes {
                if node_i == node_j {
                    continue;
                }

                let pos_j = (igr.graph[node_j].x, igr.graph[node_j].y);
                let current_dist =
                    ((pos_i.0 - pos_j.0).powi(2) + (pos_i.1 - pos_j.1).powi(2)).sqrt();
                let ideal_dist = ideal_distances
                    .get(&(node_i, node_j))
                    .copied()
                    .unwrap_or(100.0);

                if current_dist > 0.0 {
                    let force = (current_dist - ideal_dist) / current_dist;
                    dx += force * (pos_i.0 - pos_j.0);
                    dy += force * (pos_i.1 - pos_j.1);
                }
            }

            displacements.insert(node_i, (dx * temperature, dy * temperature));
        }

        // Apply displacements
        for (node_idx, (dx, dy)) in displacements {
            let node = &mut igr.graph[node_idx];
            node.x += dx;
            node.y += dy;
        }
    }

    fn apply_elk_forces(&self, igr: &mut IntermediateGraph, temperature: f64) {
        let nodes: Vec<NodeIndex> = igr.graph.node_indices().collect();
        let mut forces: HashMap<NodeIndex, (f64, f64)> = HashMap::new();

        // Initialize forces
        for &node in &nodes {
            forces.insert(node, (0.0, 0.0));
        }

        // Repulsive forces
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let node_i = nodes[i];
                let node_j = nodes[j];

                let pos_i = (igr.graph[node_i].x, igr.graph[node_i].y);
                let pos_j = (igr.graph[node_j].x, igr.graph[node_j].y);

                let dx = pos_i.0 - pos_j.0;
                let dy = pos_i.1 - pos_j.1;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                // Stronger repulsive force for better spacing
                let force = 5000.0 / (distance * distance);
                let fx = force * dx / distance;
                let fy = force * dy / distance;

                let force_i = forces.get_mut(&node_i).unwrap();
                force_i.0 += fx;
                force_i.1 += fy;

                let force_j = forces.get_mut(&node_j).unwrap();
                force_j.0 -= fx;
                force_j.1 -= fy;
            }
        }

        // Attractive forces along edges
        for edge in igr.graph.edge_indices() {
            let (source, target) = igr.graph.edge_endpoints(edge).unwrap();

            let pos_source = (igr.graph[source].x, igr.graph[source].y);
            let pos_target = (igr.graph[target].x, igr.graph[target].y);

            let dx = pos_target.0 - pos_source.0;
            let dy = pos_target.1 - pos_source.1;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);

            let force = distance * 0.01;
            let fx = force * dx / distance;
            let fy = force * dy / distance;

            let force_source = forces.get_mut(&source).unwrap();
            force_source.0 += fx;
            force_source.1 += fy;

            let force_target = forces.get_mut(&target).unwrap();
            force_target.0 -= fx;
            force_target.1 -= fy;
        }

        // Apply forces with temperature
        for (node_idx, (fx, fy)) in forces {
            let node = &mut igr.graph[node_idx];
            let displacement = (fx * fx + fy * fy).sqrt();
            if displacement > 0.0 {
                let limited_displacement = displacement.min(temperature);
                node.x += (fx / displacement) * limited_displacement;
                node.y += (fy / displacement) * limited_displacement;
            }
        }
    }

    fn find_roots(&self, igr: &IntermediateGraph) -> Vec<NodeIndex> {
        igr.graph
            .node_indices()
            .filter(|&node| {
                igr.graph
                    .edges_directed(node, PetDirection::Incoming)
                    .count()
                    == 0
            })
            .collect()
    }

    fn layout_tree_recursive(
        &self,
        igr: &mut IntermediateGraph,
        node: NodeIndex,
        x: f64,
        y: f64,
        depth: i32,
    ) -> Result<f64> {
        // Position this node (center position)
        let node_data = &igr.graph[node];
        let node_width = node_data.width;
        let node_height = node_data.height;

        igr.graph[node].x = x + node_width / 2.0;
        igr.graph[node].y = y + node_height / 2.0;

        // Get children
        let children: Vec<NodeIndex> = igr
            .graph
            .edges_directed(node, PetDirection::Outgoing)
            .map(|e| e.target())
            .collect();

        if children.is_empty() {
            return Ok(node_width);
        }

        // Layout children
        let child_spacing = self.options.spacing_node_node;
        // Use depth to calculate level spacing - can be adjusted based on tree depth
        let base_level_spacing = 100.0;
        let level_spacing = base_level_spacing + (depth as f64 * 10.0).min(50.0); // Slightly increase spacing for deeper levels
        let mut child_x = x;
        let child_y = y + node_height + level_spacing;
        let mut total_width = 0.0;

        for &child in &children {
            let child_width =
                self.layout_tree_recursive(igr, child, child_x, child_y, depth + 1)?;
            child_x += child_width + child_spacing;
            total_width += child_width + child_spacing;
        }

        // Remove extra spacing from last child
        if !children.is_empty() {
            total_width -= child_spacing;
        }

        total_width = total_width.max(node_width);

        // Center this node over its children
        if total_width > node_width {
            igr.graph[node].x = x + total_width / 2.0;
        }

        Ok(total_width)
    }

    fn calculate_shortest_paths(
        &self,
        igr: &IntermediateGraph,
    ) -> HashMap<(NodeIndex, NodeIndex), f64> {
        // Simple Floyd-Warshall for shortest paths
        let nodes: Vec<NodeIndex> = igr.graph.node_indices().collect();
        let mut distances = HashMap::new();
        let inf = 1000.0;

        // Initialize distances
        for &i in &nodes {
            for &j in &nodes {
                if i == j {
                    distances.insert((i, j), 0.0);
                } else {
                    distances.insert((i, j), inf);
                }
            }
        }

        // Set edge distances
        for edge in igr.graph.edge_indices() {
            let (source, target) = igr.graph.edge_endpoints(edge).unwrap();
            distances.insert((source, target), 100.0); // Ideal edge length
        }

        // Floyd-Warshall
        for &k in &nodes {
            for &i in &nodes {
                for &j in &nodes {
                    let current = distances[&(i, j)];
                    let via_k = distances[&(i, k)] + distances[&(k, j)];
                    if via_k < current {
                        distances.insert((i, j), via_k);
                    }
                }
            }
        }

        distances
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

            let padding = 30.0;
            container.bounds = Some(BoundingBox {
                x: min_x - padding,
                y: min_y - padding,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding,
            });
        }
    }

    fn calculate_group_bounds(&self, igr: &mut IntermediateGraph) {
        for group in &mut igr.groups {
            if group.children.is_empty() {
                continue;
            }

            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for &child_idx in &group.children {
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

            let padding = match &group.group_type {
                GroupType::FlowGroup => 35.0,
                GroupType::BasicGroup => 30.0,
                GroupType::SemanticGroup(_) => 40.0,
            };

            group.bounds = Some(BoundingBox {
                x: min_x - padding,
                y: min_y - padding,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding,
            });
        }
    }

    fn normalize_positions(&self, igr: &mut IntermediateGraph) {
        // Find minimum x and y coordinates
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;

        for node_idx in igr.graph.node_indices() {
            let node = &igr.graph[node_idx];
            min_x = min_x.min(node.x - node.width / 2.0);
            min_y = min_y.min(node.y - node.height / 2.0);
        }

        // Shift all nodes to ensure non-negative positions
        if min_x < 50.0 || min_y < 50.0 {
            let shift_x = 50.0 - min_x.min(50.0);
            let shift_y = 50.0 - min_y.min(50.0);

            for node_idx in igr.graph.node_indices() {
                let node = &mut igr.graph[node_idx];
                node.x += shift_x;
                node.y += shift_y;
            }
        }
    }
}
