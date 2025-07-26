// src/layout.rs
use crate::ast::GroupType;
use crate::error::{LayoutError, Result};
use crate::igr::{BoundingBox, IntermediateGraph};
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::Direction as PetDirection;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub trait LayoutEngine: Send + Sync {
    fn layout(&self, igr: &mut IntermediateGraph) -> Result<()>;
    fn name(&self) -> &'static str;
}

/// Cache key for layout results
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct LayoutCacheKey {
    graph_hash: u64,
    engine: String,
}

impl LayoutCacheKey {
    fn from_igr(igr: &IntermediateGraph, engine: &str) -> Self {
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
struct CachedLayout {
    positions: HashMap<String, (f64, f64)>,
}

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

// Basic Dagre-like hierarchical layout
pub struct DagreLayout {
    options: DagreLayoutOptions,
}

#[derive(Debug, Clone)]
pub struct DagreLayoutOptions {
    pub node_sep: f64,
    pub rank_sep: f64,
    pub edge_sep: f64,
    pub direction: Direction,
    pub ranker: RankingAlgorithm,
}

#[derive(Debug, Clone)]
pub enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

#[derive(Debug, Clone)]
pub enum RankingAlgorithm {
    LongestPath,
    TightTree,
    NetworkSimplex,
}

impl Default for DagreLayoutOptions {
    fn default() -> Self {
        Self {
            node_sep: 80.0,                  // Increased separation between nodes in same layer
            rank_sep: 150.0,                 // Increased separation between layers
            edge_sep: 20.0,                  // Separation between edges
            direction: Direction::LeftRight, // Changed default to left-right
            ranker: RankingAlgorithm::LongestPath,
        }
    }
}

impl Default for DagreLayout {
    fn default() -> Self {
        Self::new()
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

        // Group-aware layout
        if !igr.groups.is_empty() {
            self.layout_with_groups(igr)?;
        } else {
            // Standard layout without groups
            self.layout_standard(igr)?;
        }

        // Calculate bounds for containers and groups
        self.calculate_container_bounds(igr);
        self.calculate_group_bounds(igr);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "dagre"
    }
}

impl DagreLayout {
    fn layout_standard(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Standard layout algorithm
        let node_ranks = self.assign_ranks(igr)?;
        let layers = self.build_layers(igr, &node_ranks);
        let ordered_layers = self.minimize_crossings(igr, layers);
        self.position_nodes(igr, &ordered_layers)?;
        Ok(())
    }

    fn layout_with_groups(&self, igr: &mut IntermediateGraph) -> Result<()> {
        // Create a map of nodes to their group
        let mut node_to_group: HashMap<NodeIndex, usize> = HashMap::new();
        for (group_idx, group) in igr.groups.iter().enumerate() {
            for &node_idx in &group.children {
                node_to_group.insert(node_idx, group_idx);
            }
        }

        // Layout each group independently
        for group in igr.groups.iter() {
            if group.children.is_empty() {
                continue;
            }

            // Create a subgraph for this group
            let positions = self.layout_group_subgraph(igr, &group.children, &group.group_type)?;

            // Apply positions from subgraph layout
            for (&node_idx, &(x, y)) in &positions {
                let node = &mut igr.graph[node_idx];
                node.x = x;
                node.y = y;
            }
        }

        // Layout ungrouped nodes
        let ungrouped_nodes: Vec<NodeIndex> = igr
            .graph
            .node_indices()
            .filter(|idx| !node_to_group.contains_key(idx))
            .collect();

        if !ungrouped_nodes.is_empty() {
            self.layout_ungrouped_nodes(igr, &ungrouped_nodes)?;
        }

        // Adjust positions to ensure groups don't overlap
        self.adjust_group_positions(igr);

        Ok(())
    }

    fn layout_group_subgraph(
        &self,
        igr: &IntermediateGraph,
        group_nodes: &[NodeIndex],
        group_type: &GroupType,
    ) -> Result<HashMap<NodeIndex, (f64, f64)>> {
        let mut positions = HashMap::new();

        match group_type {
            GroupType::FlowGroup => {
                // Linear flow layout for flow groups
                let mut x = 0.0;
                let y = 0.0;

                for &node_idx in group_nodes {
                    let node = &igr.graph[node_idx];
                    positions.insert(node_idx, (x, y));
                    x += node.width + self.options.node_sep * 1.5; // Extra spacing for flow
                }
            }
            GroupType::BasicGroup | GroupType::SemanticGroup(_) => {
                // Hierarchical layout for other groups
                // Create internal edges only
                let node_set: HashSet<NodeIndex> = group_nodes.iter().copied().collect();
                let mut internal_edges = Vec::new();

                for &node_idx in group_nodes {
                    for edge in igr.graph.edges_directed(node_idx, PetDirection::Outgoing) {
                        if node_set.contains(&edge.target()) {
                            internal_edges.push((node_idx, edge.target()));
                        }
                    }
                }

                // Simple grid layout if no internal structure
                if internal_edges.is_empty() {
                    let cols = (group_nodes.len() as f64).sqrt().ceil() as usize;
                    for (i, &node_idx) in group_nodes.iter().enumerate() {
                        let row = i / cols;
                        let col = i % cols;
                        let node = &igr.graph[node_idx];

                        let x = col as f64 * (node.width + self.options.node_sep);
                        let y = row as f64 * (node.height + self.options.rank_sep);
                        positions.insert(node_idx, (x, y));
                    }
                } else {
                    // Use standard dagre for internal structure
                    // This is simplified - in a full implementation, you'd run
                    // the dagre algorithm on just this subgraph
                    let mut x = 0.0;
                    let mut y = 0.0;

                    for &node_idx in group_nodes {
                        let node = &igr.graph[node_idx];
                        positions.insert(node_idx, (x, y));
                        x += node.width + self.options.node_sep;
                        if x > 400.0 {
                            // Wrap after certain width
                            x = 0.0;
                            y += node.height + self.options.rank_sep;
                        }
                    }
                }
            }
        }

        Ok(positions)
    }

    fn layout_ungrouped_nodes(
        &self,
        igr: &mut IntermediateGraph,
        ungrouped_nodes: &[NodeIndex],
    ) -> Result<()> {
        // Simple vertical layout for ungrouped nodes
        // In a full implementation, this would consider connections to groups
        let mut y = 0.0;

        for &node_idx in ungrouped_nodes {
            let node = &mut igr.graph[node_idx];
            node.x = -200.0; // Place to the left of groups
            node.y = y;
            y += node.height + self.options.node_sep;
        }

        Ok(())
    }

    fn adjust_group_positions(&self, igr: &mut IntermediateGraph) {
        // Calculate bounds for each group
        let mut group_bounds = Vec::new();

        for group in &igr.groups {
            if group.children.is_empty() {
                continue;
            }

            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for &child_idx in &group.children {
                let node = &igr.graph[child_idx];
                min_x = min_x.min(node.x - node.width / 2.0);
                max_x = max_x.max(node.x + node.width / 2.0);
                min_y = min_y.min(node.y - node.height / 2.0);
                max_y = max_y.max(node.y + node.height / 2.0);
            }

            group_bounds.push((min_x, min_y, max_x, max_y));
        }

        // Arrange groups to prevent overlap
        let group_padding = 100.0;
        let mut x_offset = 0.0;

        for (group_idx, (min_x, _min_y, max_x, _max_y)) in group_bounds.iter().enumerate() {
            let width = max_x - min_x;
            let dx = x_offset - min_x;

            // Move all nodes in this group
            for &child_idx in &igr.groups[group_idx].children {
                igr.graph[child_idx].x += dx;
            }

            x_offset += width + group_padding;
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

            // Add padding based on group type
            let padding = match &group.group_type {
                GroupType::FlowGroup => 30.0,
                GroupType::BasicGroup => 25.0,
                GroupType::SemanticGroup(_) => 35.0,
            };

            group.bounds = Some(BoundingBox {
                x: min_x - padding,
                y: min_y - padding,
                width: (max_x - min_x) + 2.0 * padding,
                height: (max_y - min_y) + 2.0 * padding,
            });
        }
    }

    // Improved ranking algorithm based on layout-rust's longest path
    fn assign_ranks(&self, igr: &IntermediateGraph) -> Result<HashMap<NodeIndex, i32>> {
        use petgraph::algo::toposort;

        // First check for cycles
        let _ = toposort(&igr.graph, None).map_err(|cycle| {
            let node_in_cycle = &igr.graph[cycle.node_id()];
            LayoutError::CalculationFailed(format!(
                "The 'dagre' layout requires a directed acyclic graph (DAG) but found a cycle involving node '{}'. \
                Consider using 'layout: force' in your configuration instead, which supports cycles.",
                node_in_cycle.id
            ))
        })?;

        match self.options.ranker {
            RankingAlgorithm::LongestPath => self.longest_path_ranking(igr),
            RankingAlgorithm::TightTree => {
                // For now, fall back to longest path
                self.longest_path_ranking(igr)
            }
            RankingAlgorithm::NetworkSimplex => {
                // For now, fall back to longest path
                self.longest_path_ranking(igr)
            }
        }
    }

    // Longest path ranking algorithm from layout-rust
    fn longest_path_ranking(&self, igr: &IntermediateGraph) -> Result<HashMap<NodeIndex, i32>> {
        let mut ranks = HashMap::new();
        let mut visited = HashMap::new();

        // Find all source nodes (nodes with no incoming edges)
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

        // If no sources found (shouldn't happen after cycle check), use all nodes
        let starting_nodes = if sources.is_empty() {
            igr.graph.node_indices().collect()
        } else {
            sources
        };

        // DFS to calculate ranks
        for node in starting_nodes {
            Self::dfs_rank(igr, node, &mut ranks, &mut visited);
        }

        Ok(ranks)
    }

    fn dfs_rank(
        igr: &IntermediateGraph,
        node: NodeIndex,
        ranks: &mut HashMap<NodeIndex, i32>,
        visited: &mut HashMap<NodeIndex, bool>,
    ) -> i32 {
        if visited.contains_key(&node) {
            return ranks.get(&node).copied().unwrap_or(0);
        }

        visited.insert(node, true);

        // Get ranks of all successors
        let successor_ranks: Vec<i32> = igr
            .graph
            .edges_directed(node, PetDirection::Outgoing)
            .map(|edge| {
                let target = edge.target();
                // Edge weight (min length) is 1 by default
                let edge_weight = 1;
                Self::dfs_rank(igr, target, ranks, visited) - edge_weight
            })
            .collect();

        // The rank is the minimum of successor ranks
        let rank = successor_ranks.into_iter().min().unwrap_or(0);
        ranks.insert(node, rank);

        rank
    }

    // Build layers from ranks
    fn build_layers(
        &self,
        _igr: &IntermediateGraph,
        node_ranks: &HashMap<NodeIndex, i32>,
    ) -> Vec<Vec<NodeIndex>> {
        let mut layers_map: HashMap<i32, Vec<NodeIndex>> = HashMap::new();

        // Group nodes by rank
        for (node, &rank) in node_ranks.iter() {
            layers_map.entry(rank).or_insert_with(Vec::new).push(*node);
        }

        // Convert to sorted vector of layers
        let mut sorted_ranks: Vec<i32> = layers_map.keys().copied().collect();
        sorted_ranks.sort();

        sorted_ranks
            .into_iter()
            .map(|rank| layers_map.remove(&rank).unwrap())
            .collect()
    }

    // Crossing minimization using barycenter method inspired by layout-rust
    fn minimize_crossings(
        &self,
        igr: &IntermediateGraph,
        mut layers: Vec<Vec<NodeIndex>>,
    ) -> Vec<Vec<NodeIndex>> {
        // Multiple passes to improve crossing reduction
        for _ in 0..4 {
            // Forward pass (top to bottom)
            for i in 1..layers.len() {
                let (prev_part, curr_part) = layers.split_at_mut(i);
                let prev_layer = &prev_part[i - 1];
                let current_layer = &mut curr_part[0];
                self.sort_layer_by_barycenter(igr, current_layer, prev_layer, true);
            }

            // Backward pass (bottom to top)
            for i in (0..layers.len() - 1).rev() {
                let (curr_part, next_part) = layers.split_at_mut(i + 1);
                let next_layer = &next_part[0];
                let current_layer = &mut curr_part[i];
                self.sort_layer_by_barycenter(igr, current_layer, next_layer, false);
            }
        }

        layers
    }

    // Sort nodes in a layer based on barycenter of connected nodes
    fn sort_layer_by_barycenter(
        &self,
        igr: &IntermediateGraph,
        layer: &mut [NodeIndex],
        reference_layer: &[NodeIndex],
        forward: bool,
    ) {
        // Create position map for reference layer
        let positions: HashMap<NodeIndex, usize> = reference_layer
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        // Calculate barycenter for each node
        let barycenters: Vec<(NodeIndex, Option<f64>)> = layer
            .iter()
            .map(|&node| {
                let edges = if forward {
                    // Look at incoming edges from previous layer
                    igr.graph.edges_directed(node, PetDirection::Incoming)
                } else {
                    // Look at outgoing edges to next layer
                    igr.graph.edges_directed(node, PetDirection::Outgoing)
                };

                let mut sum = 0.0;
                let mut count = 0;

                for edge in edges {
                    let other_node = if forward {
                        edge.source()
                    } else {
                        edge.target()
                    };
                    if let Some(&pos) = positions.get(&other_node) {
                        sum += pos as f64;
                        count += 1;
                    }
                }

                let barycenter = if count > 0 {
                    Some(sum / count as f64)
                } else {
                    None
                };

                (node, barycenter)
            })
            .collect();

        // Sort by barycenter (nodes without connections stay in place)
        let mut sorted_indices: Vec<usize> = (0..layer.len()).collect();
        sorted_indices.sort_by(|&a, &b| match (barycenters[a].1, barycenters[b].1) {
            (Some(bc_a), Some(bc_b)) => bc_a.partial_cmp(&bc_b).unwrap(),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(&b),
        });

        // Apply the new ordering
        let original_layer = layer.to_owned();
        for (i, &idx) in sorted_indices.iter().enumerate() {
            layer[i] = original_layer[idx];
        }
    }

    fn position_nodes(&self, igr: &mut IntermediateGraph, layers: &[Vec<NodeIndex>]) -> Result<()> {
        // First assign Y positions (or X for horizontal layouts) based on layers
        self.assign_layer_positions(igr, layers);

        // Then assign X positions (or Y for horizontal) within each layer
        self.assign_node_positions_within_layers(igr, layers);

        Ok(())
    }

    // Assign positions to layers (Y for TB/BT, X for LR/RL)
    fn assign_layer_positions(&self, igr: &mut IntermediateGraph, layers: &[Vec<NodeIndex>]) {
        let mut layer_positions = Vec::new();
        let mut current_pos = 0.0;

        // Calculate position for each layer
        for layer in layers {
            if layer.is_empty() {
                continue;
            }

            // Find maximum dimension in this layer
            let max_dimension = match self.options.direction {
                Direction::LeftRight | Direction::RightLeft => layer
                    .iter()
                    .map(|&idx| igr.graph[idx].width)
                    .fold(0.0, f64::max),
                Direction::TopBottom | Direction::BottomTop => layer
                    .iter()
                    .map(|&idx| igr.graph[idx].height)
                    .fold(0.0, f64::max),
            };

            layer_positions.push(current_pos + max_dimension / 2.0);
            current_pos += max_dimension + self.options.rank_sep;
        }

        // Apply positions to nodes
        for (layer_idx, layer) in layers.iter().enumerate() {
            if layer_idx >= layer_positions.len() {
                continue;
            }

            let pos = layer_positions[layer_idx];

            for &node_idx in layer {
                let node = &mut igr.graph[node_idx];
                match self.options.direction {
                    Direction::LeftRight => node.x = pos,
                    Direction::RightLeft => node.x = -pos,
                    Direction::TopBottom => node.y = pos,
                    Direction::BottomTop => node.y = -pos,
                }
            }
        }
    }

    // Assign positions within each layer
    fn assign_node_positions_within_layers(
        &self,
        igr: &mut IntermediateGraph,
        layers: &[Vec<NodeIndex>],
    ) {
        // Track which paths nodes belong to for better separation
        let mut path_groups: HashMap<NodeIndex, usize> = HashMap::new();
        let mut next_path_id = 0;

        // Assign path IDs based on connectivity
        for layer in layers.iter() {
            for &node_idx in layer {
                // Check if this node has an assigned path from a parent
                let incoming_paths: HashSet<usize> = igr
                    .graph
                    .edges_directed(node_idx, PetDirection::Incoming)
                    .filter_map(|edge| path_groups.get(&edge.source()).copied())
                    .collect();

                let path_id = if incoming_paths.is_empty() {
                    // New path starting from this node
                    let id = next_path_id;
                    next_path_id += 1;
                    id
                } else if incoming_paths.len() == 1 {
                    // Continue on the same path
                    *incoming_paths.iter().next().unwrap()
                } else {
                    // Multiple paths converging - take the smallest path ID
                    *incoming_paths.iter().min().unwrap()
                };

                path_groups.insert(node_idx, path_id);
            }
        }

        for layer in layers.iter() {
            if layer.is_empty() {
                continue;
            }

            // Group nodes by their path
            let mut nodes_by_path: HashMap<usize, Vec<(NodeIndex, f64)>> = HashMap::new();

            for &node_idx in layer {
                let node = &igr.graph[node_idx];
                let size = match self.options.direction {
                    Direction::LeftRight | Direction::RightLeft => node.height,
                    Direction::TopBottom | Direction::BottomTop => node.width,
                };

                let path_id = path_groups.get(&node_idx).copied().unwrap_or(0);
                nodes_by_path
                    .entry(path_id)
                    .or_default()
                    .push((node_idx, size));
            }

            // Sort paths by ID for consistent ordering
            let mut paths: Vec<_> = nodes_by_path.into_iter().collect();
            paths.sort_by_key(|(path_id, _)| *path_id);

            // Calculate total size needed for this layer with extra spacing between paths
            let path_separation = self.options.node_sep * 2.0; // Extra space between different paths
            let mut total_size = 0.0;

            for (_, nodes) in &paths {
                total_size += nodes.iter().map(|(_, size)| size).sum::<f64>();
                total_size += (nodes.len().saturating_sub(1)) as f64 * self.options.node_sep;
            }
            total_size += (paths.len().saturating_sub(1)) as f64 * path_separation;

            // Start positioning from the center
            let mut current_pos = -total_size / 2.0;

            // Position each path group
            for (path_idx, (_path_id, nodes)) in paths.iter().enumerate() {
                if path_idx > 0 {
                    current_pos += path_separation;
                }

                // Position nodes within this path
                for (i, &(node_idx, size)) in nodes.iter().enumerate() {
                    if i > 0 {
                        current_pos += self.options.node_sep;
                    }

                    let node = &mut igr.graph[node_idx];

                    match self.options.direction {
                        Direction::LeftRight | Direction::RightLeft => {
                            node.y = current_pos + size / 2.0;
                        }
                        Direction::TopBottom | Direction::BottomTop => {
                            node.x = current_pos + size / 2.0;
                        }
                    }

                    current_pos += size;
                }
            }
        }
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
