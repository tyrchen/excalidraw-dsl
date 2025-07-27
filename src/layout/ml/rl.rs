// src/layout/ml/rl.rs
//! Reinforcement Learning for layout optimization

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use candle_core::{Device, Module, Tensor};
use candle_nn::{linear, VarBuilder, VarMap};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::Path;

/// Actions that can be taken to optimize layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAction {
    /// Move a node by delta x, y
    MoveNode { node_idx: usize, dx: f64, dy: f64 },
    /// Align nodes horizontally
    AlignHorizontal { nodes: [usize; 2] },
    /// Align nodes vertically
    AlignVertical { nodes: [usize; 2] },
    /// Adjust spacing between nodes
    AdjustSpacing { scale: f64 },
    /// Rotate a subgraph
    RotateSubgraph { center_idx: usize, angle: f64 },
    /// Swap positions of two nodes
    SwapNodes { node1: usize, node2: usize },
}

/// Current state of the layout
#[derive(Clone)]
pub struct LayoutState {
    pub positions: HashMap<NodeIndex, (f64, f64)>,
    pub edge_crossings: usize,
    pub node_overlaps: usize,
    pub bounding_box: (f64, f64, f64, f64), // min_x, min_y, max_x, max_y
    pub symmetry_score: f64,
}

impl LayoutState {
    /// Convert state to feature vector for neural network
    pub fn to_features(&self) -> Vec<f32> {
        let mut features = vec![];

        // Basic metrics
        features.push(self.edge_crossings as f32 / 100.0);
        features.push(self.node_overlaps as f32 / 50.0);

        // Bounding box features
        let width = self.bounding_box.2 - self.bounding_box.0;
        let height = self.bounding_box.3 - self.bounding_box.1;
        features.push((width / 1000.0) as f32);
        features.push((height / 1000.0) as f32);
        features.push((width / height.max(1.0)) as f32); // aspect ratio

        // Space utilization
        let area = width * height;
        let node_count = self.positions.len() as f64;
        features.push((node_count / area.max(1.0) * 10000.0) as f32);

        // Symmetry score
        features.push(self.symmetry_score as f32);

        // Position statistics
        let x_coords: Vec<f64> = self.positions.values().map(|(x, _)| *x).collect();
        let y_coords: Vec<f64> = self.positions.values().map(|(_, y)| *y).collect();

        let x_mean = x_coords.iter().sum::<f64>() / x_coords.len().max(1) as f64;
        let y_mean = y_coords.iter().sum::<f64>() / y_coords.len().max(1) as f64;

        features.push((x_mean / 500.0) as f32);
        features.push((y_mean / 500.0) as f32);

        // Pad to fixed size
        while features.len() < 20 {
            features.push(0.0);
        }

        features
    }
}

/// Environment for RL-based layout optimization
pub struct LayoutEnvironment {
    graph: IntermediateGraph,
    current_state: LayoutState,
    initial_state: LayoutState,
    quality_evaluator: QualityEvaluator,
    step_count: usize,
    max_steps: usize,
}

impl LayoutEnvironment {
    pub fn new(
        graph: IntermediateGraph,
        initial_positions: HashMap<NodeIndex, (f64, f64)>,
    ) -> Result<Self> {
        let quality_evaluator = QualityEvaluator::new();
        let initial_state = quality_evaluator.evaluate_state(&graph, &initial_positions)?;

        Ok(Self {
            graph,
            current_state: initial_state.clone(),
            initial_state,
            quality_evaluator,
            step_count: 0,
            max_steps: 100,
        })
    }

    /// Apply an action and return new state, reward, and done flag
    pub fn step(&mut self, action: LayoutAction) -> Result<(LayoutState, f64, bool)> {
        // Apply action to current positions
        let mut new_positions = self.current_state.positions.clone();
        self.apply_action(action, &mut new_positions)?;

        // Evaluate new state
        let new_state = self
            .quality_evaluator
            .evaluate_state(&self.graph, &new_positions)?;

        // Calculate reward
        let reward = self.calculate_reward(&self.current_state, &new_state);

        // Update state
        self.current_state = new_state.clone();
        self.step_count += 1;

        // Check if done
        let done = self.step_count >= self.max_steps || self.is_converged();

        Ok((new_state, reward, done))
    }

    /// Reset environment to initial state
    pub fn reset(&mut self) -> LayoutState {
        self.current_state = self.initial_state.clone();
        self.step_count = 0;
        self.current_state.clone()
    }

    /// Apply action to positions
    fn apply_action(
        &self,
        action: LayoutAction,
        positions: &mut HashMap<NodeIndex, (f64, f64)>,
    ) -> Result<()> {
        let node_indices: Vec<NodeIndex> = self.graph.graph.node_indices().collect();

        match action {
            LayoutAction::MoveNode { node_idx, dx, dy } => {
                if let Some(&node) = node_indices.get(node_idx) {
                    if let Some(pos) = positions.get_mut(&node) {
                        pos.0 += dx;
                        pos.1 += dy;
                    }
                }
            }
            LayoutAction::AlignHorizontal { nodes } => {
                if let (Some(&node1), Some(&node2)) =
                    (node_indices.get(nodes[0]), node_indices.get(nodes[1]))
                {
                    if let (Some(pos1), Some(pos2)) = (positions.get(&node1), positions.get(&node2))
                    {
                        let avg_y = (pos1.1 + pos2.1) / 2.0;
                        if let Some(pos1) = positions.get_mut(&node1) {
                            pos1.1 = avg_y;
                        }
                        if let Some(pos2) = positions.get_mut(&node2) {
                            pos2.1 = avg_y;
                        }
                    }
                }
            }
            LayoutAction::AlignVertical { nodes } => {
                if let (Some(&node1), Some(&node2)) =
                    (node_indices.get(nodes[0]), node_indices.get(nodes[1]))
                {
                    if let (Some(pos1), Some(pos2)) = (positions.get(&node1), positions.get(&node2))
                    {
                        let avg_x = (pos1.0 + pos2.0) / 2.0;
                        if let Some(pos1) = positions.get_mut(&node1) {
                            pos1.0 = avg_x;
                        }
                        if let Some(pos2) = positions.get_mut(&node2) {
                            pos2.0 = avg_x;
                        }
                    }
                }
            }
            LayoutAction::AdjustSpacing { scale } => {
                // Calculate center of mass
                let (cx, cy) = self.calculate_center_of_mass(positions);

                // Scale positions relative to center
                for pos in positions.values_mut() {
                    pos.0 = cx + (pos.0 - cx) * scale;
                    pos.1 = cy + (pos.1 - cy) * scale;
                }
            }
            LayoutAction::RotateSubgraph { center_idx, angle } => {
                if let Some(&center_node) = node_indices.get(center_idx) {
                    if let Some(&(cx, cy)) = positions.get(&center_node) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();

                        for (&node, pos) in positions.iter_mut() {
                            if node != center_node {
                                let dx = pos.0 - cx;
                                let dy = pos.1 - cy;
                                pos.0 = cx + dx * cos_a - dy * sin_a;
                                pos.1 = cy + dx * sin_a + dy * cos_a;
                            }
                        }
                    }
                }
            }
            LayoutAction::SwapNodes { node1, node2 } => {
                if let (Some(&n1), Some(&n2)) = (node_indices.get(node1), node_indices.get(node2)) {
                    if let (Some(pos1), Some(pos2)) =
                        (positions.get(&n1).cloned(), positions.get(&n2).cloned())
                    {
                        positions.insert(n1, pos2);
                        positions.insert(n2, pos1);
                    }
                }
            }
        }

        Ok(())
    }

    fn calculate_center_of_mass(&self, positions: &HashMap<NodeIndex, (f64, f64)>) -> (f64, f64) {
        let count = positions.len() as f64;
        let sum_x: f64 = positions.values().map(|(x, _)| x).sum();
        let sum_y: f64 = positions.values().map(|(_, y)| y).sum();
        (sum_x / count, sum_y / count)
    }

    fn calculate_reward(&self, old_state: &LayoutState, new_state: &LayoutState) -> f64 {
        let mut reward = 0.0;

        // Edge crossing improvement
        let crossing_improvement =
            (old_state.edge_crossings as f64 - new_state.edge_crossings as f64) * 0.2;
        reward += crossing_improvement;

        // Node overlap improvement
        let overlap_improvement =
            (old_state.node_overlaps as f64 - new_state.node_overlaps as f64) * 0.5;
        reward += overlap_improvement;

        // Symmetry improvement
        let symmetry_improvement = (new_state.symmetry_score - old_state.symmetry_score) * 0.15;
        reward += symmetry_improvement;

        // Space utilization (prefer compact layouts)
        let old_area = (old_state.bounding_box.2 - old_state.bounding_box.0)
            * (old_state.bounding_box.3 - old_state.bounding_box.1);
        let new_area = (new_state.bounding_box.2 - new_state.bounding_box.0)
            * (new_state.bounding_box.3 - new_state.bounding_box.1);
        let area_improvement = (old_area - new_area) / old_area.max(1.0) * 0.1;
        reward += area_improvement;

        // Small penalty for each step to encourage efficiency
        reward -= 0.01;

        reward
    }

    fn is_converged(&self) -> bool {
        // Simple convergence check
        self.current_state.edge_crossings == 0 && self.current_state.node_overlaps == 0
    }
}

/// Quality evaluator for layout states
struct QualityEvaluator;

impl QualityEvaluator {
    fn new() -> Self {
        Self
    }

    fn evaluate_state(
        &self,
        graph: &IntermediateGraph,
        positions: &HashMap<NodeIndex, (f64, f64)>,
    ) -> Result<LayoutState> {
        let edge_crossings = self.count_edge_crossings(graph, positions);
        let node_overlaps = self.count_node_overlaps(positions);
        let bounding_box = self.calculate_bounding_box(positions);
        let symmetry_score = self.calculate_symmetry_score(positions);

        Ok(LayoutState {
            positions: positions.clone(),
            edge_crossings,
            node_overlaps,
            bounding_box,
            symmetry_score,
        })
    }

    fn count_edge_crossings(
        &self,
        graph: &IntermediateGraph,
        positions: &HashMap<NodeIndex, (f64, f64)>,
    ) -> usize {
        // Simplified edge crossing detection
        let mut crossings = 0;
        let edges: Vec<_> = graph.graph.edge_references().collect();

        for i in 0..edges.len() {
            for j in (i + 1)..edges.len() {
                if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
                    positions.get(&edges[i].source()),
                    positions.get(&edges[i].target()),
                    positions.get(&edges[j].source()),
                    positions.get(&edges[j].target()),
                ) {
                    if self.lines_intersect(p1, p2, p3, p4) {
                        crossings += 1;
                    }
                }
            }
        }

        crossings
    }

    fn lines_intersect(
        &self,
        p1: &(f64, f64),
        p2: &(f64, f64),
        p3: &(f64, f64),
        p4: &(f64, f64),
    ) -> bool {
        // Simple line intersection test
        let d = (p4.1 - p3.1) * (p2.0 - p1.0) - (p4.0 - p3.0) * (p2.1 - p1.1);
        if d.abs() < 1e-10 {
            return false;
        }

        let ua = ((p4.0 - p3.0) * (p1.1 - p3.1) - (p4.1 - p3.1) * (p1.0 - p3.0)) / d;
        let ub = ((p2.0 - p1.0) * (p1.1 - p3.1) - (p2.1 - p1.1) * (p1.0 - p3.0)) / d;

        ua > 0.0 && ua < 1.0 && ub > 0.0 && ub < 1.0
    }

    fn count_node_overlaps(&self, positions: &HashMap<NodeIndex, (f64, f64)>) -> usize {
        // Simplified node overlap detection
        let mut overlaps = 0;
        let node_size = 50.0; // Assume fixed node size

        let nodes: Vec<_> = positions.iter().collect();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let (_, pos1) = nodes[i];
                let (_, pos2) = nodes[j];

                let dx = pos1.0 - pos2.0;
                let dy = pos1.1 - pos2.1;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < node_size {
                    overlaps += 1;
                }
            }
        }

        overlaps
    }

    fn calculate_bounding_box(
        &self,
        positions: &HashMap<NodeIndex, (f64, f64)>,
    ) -> (f64, f64, f64, f64) {
        if positions.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for (x, y) in positions.values() {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        (min_x, min_y, max_x, max_y)
    }

    fn calculate_symmetry_score(&self, positions: &HashMap<NodeIndex, (f64, f64)>) -> f64 {
        // Simple symmetry score based on position distribution
        if positions.is_empty() {
            return 0.0;
        }

        let (cx, cy) = positions
            .values()
            .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));
        let cx = cx / positions.len() as f64;
        let cy = cy / positions.len() as f64;

        // Calculate variance in distances from center
        let distances: Vec<f64> = positions
            .values()
            .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
            .collect();

        let mean_dist = distances.iter().sum::<f64>() / distances.len() as f64;
        let variance = distances
            .iter()
            .map(|d| (d - mean_dist).powi(2))
            .sum::<f64>()
            / distances.len() as f64;

        // Lower variance means better symmetry
        1.0 / (1.0 + variance / 100.0)
    }
}

/// PPO Policy network for RL optimization
pub struct LayoutPolicy {
    var_map: VarMap,
    device: Device,
    actor_net: Box<dyn Fn(&Tensor) -> candle_core::Result<Tensor> + Send + Sync>,
    critic_net: Box<dyn Fn(&Tensor) -> candle_core::Result<Tensor> + Send + Sync>,
}

impl LayoutPolicy {
    pub fn new(state_dim: usize, action_dim: usize, hidden_dim: usize) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        // Build actor network
        let actor_fc1 = linear(state_dim, hidden_dim, vb.pp("actor_fc1")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create actor fc1: {e}"
            )))
        })?;
        let actor_fc2 = linear(hidden_dim, hidden_dim, vb.pp("actor_fc2")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create actor fc2: {e}"
            )))
        })?;
        let actor_out = linear(hidden_dim, action_dim, vb.pp("actor_out")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create actor output: {e}"
            )))
        })?;

        let actor_net = move |x: &Tensor| -> candle_core::Result<Tensor> {
            let x = actor_fc1.forward(x)?;
            let x = x.relu()?;
            let x = actor_fc2.forward(&x)?;
            let x = x.relu()?;
            let x = actor_out.forward(&x)?;
            candle_nn::ops::softmax(&x, 1)
        };

        // Build critic network
        let critic_fc1 = linear(state_dim, hidden_dim, vb.pp("critic_fc1")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create critic fc1: {e}"
            )))
        })?;
        let critic_fc2 = linear(hidden_dim, hidden_dim, vb.pp("critic_fc2")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create critic fc2: {e}"
            )))
        })?;
        let critic_out = linear(hidden_dim, 1, vb.pp("critic_out")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create critic output: {e}"
            )))
        })?;

        let critic_net = move |x: &Tensor| -> candle_core::Result<Tensor> {
            let x = critic_fc1.forward(x)?;
            let x = x.relu()?;
            let x = critic_fc2.forward(&x)?;
            let x = x.relu()?;
            critic_out.forward(&x)
        };

        Ok(Self {
            var_map,
            device,
            actor_net: Box::new(actor_net),
            critic_net: Box::new(critic_net),
        })
    }

    pub fn forward(&self, state: &LayoutState) -> Result<(Vec<f64>, f64)> {
        let features = state.to_features();
        let features_len = features.len();
        let state_tensor =
            Tensor::from_vec(features, &[1, features_len], &self.device).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to create state tensor: {e}"
                )))
            })?;

        // Get action probabilities
        let action_probs = (self.actor_net)(&state_tensor).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to compute action probs: {e}"
            )))
        })?;

        let probs_vec: Vec<f32> = action_probs
            .squeeze(0)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to squeeze action probs: {e}"
                )))
            })?
            .to_vec1()
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to extract action probs: {e}"
                )))
            })?;

        // Get state value
        let value = (self.critic_net)(&state_tensor).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to compute value: {e}"
            )))
        })?;

        let value_scalar = value.to_scalar::<f32>().map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to extract value scalar: {e}"
            )))
        })?;

        Ok((
            probs_vec.iter().map(|&x| x as f64).collect(),
            value_scalar as f64,
        ))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.var_map.save(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to save RL policy: {e}"
            )))
        })
    }

    pub fn load(path: &Path) -> Result<Self> {
        let _device = Device::Cpu;
        let mut var_map = VarMap::new();

        var_map.load(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to load RL policy: {e}"
            )))
        })?;

        // Recreate with default dimensions
        Self::new(20, 10, 256)
    }
}

/// RL-based layout optimizer
pub struct RLLayoutOptimizer {
    policy: LayoutPolicy,
    learning_rate: f64,
    discount_factor: f64,
}

impl RLLayoutOptimizer {
    pub fn new() -> Result<Self> {
        let policy = LayoutPolicy::new(20, 10, 256)?;

        Ok(Self {
            policy,
            learning_rate: 0.001,
            discount_factor: 0.99,
        })
    }

    pub fn optimize_layout(
        &self,
        graph: &IntermediateGraph,
        initial_positions: HashMap<NodeIndex, (f64, f64)>,
        max_episodes: usize,
    ) -> Result<HashMap<NodeIndex, (f64, f64)>> {
        let mut best_positions = initial_positions.clone();
        let mut best_score = f64::MIN;

        for episode in 0..max_episodes {
            let graph_clone = graph.clone();
            let mut env = LayoutEnvironment::new(graph_clone, initial_positions.clone())?;
            let mut episode_score = 0.0;

            loop {
                let state = env.current_state.clone();

                // Get action from policy
                let (action_probs, _value) = self.policy.forward(&state)?;
                let action = self.sample_action(&action_probs);

                // Take action
                let (_new_state, reward, done) = env.step(action)?;
                episode_score += reward;

                if done {
                    break;
                }
            }

            // Update best if better
            if episode_score > best_score {
                best_score = episode_score;
                best_positions = env.current_state.positions.clone();
                log::info!("RL Optimizer - Episode {episode}: New best score = {best_score:.3}");
            }
        }

        Ok(best_positions)
    }

    fn sample_action(&self, action_probs: &[f64]) -> LayoutAction {
        // Sample action based on probabilities
        #[allow(deprecated)]
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let r: f64 = rng.random();
        let mut cumsum = 0.0;

        for (i, &prob) in action_probs.iter().enumerate() {
            cumsum += prob;
            if r < cumsum {
                return self.index_to_action(i);
            }
        }

        // Default action
        LayoutAction::MoveNode {
            node_idx: 0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    fn index_to_action(&self, index: usize) -> LayoutAction {
        // Map index to action type
        match index % 6 {
            0 => LayoutAction::MoveNode {
                node_idx: index / 6,
                dx: ((index % 3) as f64 - 1.0) * 10.0,
                dy: ((index % 5) as f64 - 2.0) * 10.0,
            },
            1 => LayoutAction::AlignHorizontal {
                nodes: [index % 10, (index + 1) % 10],
            },
            2 => LayoutAction::AlignVertical {
                nodes: [index % 10, (index + 1) % 10],
            },
            3 => LayoutAction::AdjustSpacing {
                scale: 0.9 + (index % 3) as f64 * 0.1,
            },
            4 => LayoutAction::RotateSubgraph {
                center_idx: index % 10,
                angle: ((index % 4) as f64 - 2.0) * 0.1,
            },
            _ => LayoutAction::SwapNodes {
                node1: index % 10,
                node2: (index + 1) % 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn test_layout_environment() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: StdHashMap::new(),
            templates: StdHashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
            ],
            edges: vec![EdgeDefinition {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                arrow_type: ArrowType::SingleArrow,
                attributes: StdHashMap::new(),
                style: None,
            }],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();

        // Create initial positions
        let mut positions = HashMap::new();
        for (i, node) in igr.graph.node_indices().enumerate() {
            positions.insert(node, (i as f64 * 100.0, 0.0));
        }

        let mut env = LayoutEnvironment::new(igr, positions).unwrap();

        // Test stepping
        let action = LayoutAction::MoveNode {
            node_idx: 0,
            dx: 10.0,
            dy: 10.0,
        };

        let (new_state, reward, done) = env.step(action).unwrap();
        assert!(!done);
        assert_eq!(new_state.positions.len(), 2);
    }

    #[test]
    fn test_rl_policy() {
        let policy = LayoutPolicy::new(20, 10, 256).unwrap();

        let state = LayoutState {
            positions: HashMap::new(),
            edge_crossings: 5,
            node_overlaps: 2,
            bounding_box: (0.0, 0.0, 100.0, 100.0),
            symmetry_score: 0.7,
        };

        let (action_probs, value) = policy.forward(&state).unwrap();

        assert_eq!(action_probs.len(), 10);
        assert!((action_probs.iter().sum::<f64>() - 1.0).abs() < 0.01);
        assert!(value.is_finite());
    }
}
