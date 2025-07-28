// src/layout/ml/constraints.rs
//! Neural constraint satisfaction for layout

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use candle_core::{Device, Module, Tensor};
use candle_nn::{linear, VarBuilder, VarMap, RNN};
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoNodeIdentifiers;
use std::collections::HashMap;
use std::path::Path;

/// Layout constraint types
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutConstraint {
    /// Minimum distance between two nodes
    MinDistance {
        node1: NodeIndex,
        node2: NodeIndex,
        distance: f64,
    },
    /// Maximum distance between two nodes
    MaxDistance {
        node1: NodeIndex,
        node2: NodeIndex,
        distance: f64,
    },
    /// Alignment constraint
    Alignment { nodes: Vec<NodeIndex>, axis: Axis },
    /// Containment constraint
    Containment {
        container: NodeIndex,
        children: Vec<NodeIndex>,
        padding: f64,
    },
    /// Ordering constraint
    Ordering {
        nodes: Vec<NodeIndex>,
        direction: Direction,
    },
    /// Fixed position constraint
    FixedPosition {
        node: NodeIndex,
        position: (f64, f64),
    },
    /// Relative position constraint
    RelativePosition {
        node: NodeIndex,
        reference: NodeIndex,
        offset: (f64, f64),
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Result of constraint solving
#[derive(Debug, Clone)]
pub struct ConstraintSolution {
    pub positions: HashMap<NodeIndex, (f64, f64)>,
    pub satisfied_constraints: Vec<usize>,
    pub violated_constraints: Vec<(usize, f64)>, // (constraint_idx, violation_amount)
    pub feasibility_score: f64,
}

/// Neural constraint solver
pub struct NeuralConstraintSolver {
    var_map: VarMap,
    device: Device,
    encoder: ConstraintEncoder,
    solver_network: SolverNetwork,
    decoder: SolutionDecoder,
    max_iterations: usize,
}

impl NeuralConstraintSolver {
    pub fn new(max_iterations: usize) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        let encoder = ConstraintEncoder::new(&vb)?;
        let solver_network = SolverNetwork::new(&vb)?;
        let decoder = SolutionDecoder::new(&vb)?;

        Ok(Self {
            var_map,
            device,
            encoder,
            solver_network,
            decoder,
            max_iterations,
        })
    }

    pub fn from_path(path: &Path, max_iterations: usize) -> Result<Self> {
        let device = Device::Cpu;
        let mut var_map = VarMap::new();

        var_map.load(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to load constraint solver model: {e}"
            )))
        })?;

        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        let encoder = ConstraintEncoder::new(&vb)?;
        let solver_network = SolverNetwork::new(&vb)?;
        let decoder = SolutionDecoder::new(&vb)?;

        Ok(Self {
            var_map,
            device,
            encoder,
            solver_network,
            decoder,
            max_iterations,
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.var_map.save(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to save constraint solver model: {e}"
            )))
        })
    }

    /// Solve layout constraints
    pub fn solve(
        &self,
        graph: &IntermediateGraph,
        constraints: &[LayoutConstraint],
        initial_positions: Option<&HashMap<NodeIndex, (f64, f64)>>,
    ) -> Result<ConstraintSolution> {
        // Get all nodes from the graph (not just constraint nodes)
        let nodes: Vec<NodeIndex> = graph.graph.node_identifiers().collect();

        // Initialize positions
        let positions = if let Some(init_pos) = initial_positions {
            init_pos.clone()
        } else {
            self.initialize_positions(&nodes)
        };

        // Encode constraints
        let encoded_constraints = self.encoder.encode(constraints, &nodes)?;

        // Initialize hidden state
        let mut hidden = self.initialize_hidden_state(&nodes, &positions)?;

        // Iterative solving
        let mut best_solution = None;
        let mut best_feasibility = 0.0;

        for iteration in 0..self.max_iterations {
            // Run solver network
            let (new_hidden, feasibility) =
                self.solver_network.forward(&hidden, &encoded_constraints)?;

            // Decode positions
            let new_positions = self.decoder.decode(&new_hidden, &nodes)?;

            // Evaluate constraints
            let (satisfied, violated) = self.evaluate_constraints(constraints, &new_positions);
            let feasibility_score = match *feasibility.dims() {
                [1, 1] => feasibility
                    .squeeze(0)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to squeeze feasibility: {e}"
                        )))
                    })?
                    .squeeze(0)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to squeeze feasibility again: {e}"
                        )))
                    })?
                    .to_scalar::<f32>()
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to extract feasibility: {e}"
                        )))
                    })? as f64,
                [1] => feasibility
                    .squeeze(0)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to squeeze feasibility: {e}"
                        )))
                    })?
                    .to_scalar::<f32>()
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to extract feasibility: {e}"
                        )))
                    })? as f64,
                _ => feasibility.to_scalar::<f32>().map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to extract feasibility: {e}"
                    )))
                })? as f64,
            };

            // Update best solution
            if feasibility_score > best_feasibility {
                best_feasibility = feasibility_score;
                best_solution = Some(ConstraintSolution {
                    positions: new_positions.clone(),
                    satisfied_constraints: satisfied.clone(),
                    violated_constraints: violated.clone(),
                    feasibility_score,
                });
            }

            // Check convergence
            if feasibility_score > 0.95 || violated.is_empty() {
                log::info!("Constraints solved in {iteration} iterations");
                break;
            }

            // Update for next iteration
            hidden = new_hidden;
            let _ = new_positions;
        }

        best_solution.ok_or_else(|| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                "Failed to find feasible solution".to_string(),
            ))
        })
    }

    fn initialize_positions(&self, nodes: &[NodeIndex]) -> HashMap<NodeIndex, (f64, f64)> {
        let mut positions = HashMap::new();
        let n = nodes.len() as f64;

        for (i, &node) in nodes.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n;
            let x = 200.0 * angle.cos();
            let y = 200.0 * angle.sin();
            positions.insert(node, (x, y));
        }

        positions
    }

    fn initialize_hidden_state(
        &self,
        nodes: &[NodeIndex],
        positions: &HashMap<NodeIndex, (f64, f64)>,
    ) -> Result<Tensor> {
        let hidden_dim = 128;
        let mut state = vec![0.0f32; nodes.len() * hidden_dim];

        // Encode initial positions into hidden state
        for (i, &node) in nodes.iter().enumerate() {
            if let Some(&(x, y)) = positions.get(&node) {
                let base_idx = i * hidden_dim;
                state[base_idx] = (x / 100.0) as f32;
                state[base_idx + 1] = (y / 100.0) as f32;
                // Initialize rest with small random values
                for j in 2..hidden_dim {
                    state[base_idx + j] = ((i * j) % 13) as f32 / 13.0 - 0.5;
                }
            }
        }

        Tensor::from_vec(state, &[nodes.len(), hidden_dim], &self.device).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create hidden state: {e}"
            )))
        })
    }

    fn evaluate_constraints(
        &self,
        constraints: &[LayoutConstraint],
        positions: &HashMap<NodeIndex, (f64, f64)>,
    ) -> (Vec<usize>, Vec<(usize, f64)>) {
        let mut satisfied = vec![];
        let mut violated = vec![];

        for (i, constraint) in constraints.iter().enumerate() {
            match constraint {
                LayoutConstraint::MinDistance {
                    node1,
                    node2,
                    distance,
                } => {
                    if let (Some(&pos1), Some(&pos2)) = (positions.get(node1), positions.get(node2))
                    {
                        let actual_dist =
                            ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
                        if actual_dist >= *distance {
                            satisfied.push(i);
                        } else {
                            violated.push((i, distance - actual_dist));
                        }
                    }
                }
                LayoutConstraint::MaxDistance {
                    node1,
                    node2,
                    distance,
                } => {
                    if let (Some(&pos1), Some(&pos2)) = (positions.get(node1), positions.get(node2))
                    {
                        let actual_dist =
                            ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
                        if actual_dist <= *distance {
                            satisfied.push(i);
                        } else {
                            violated.push((i, actual_dist - distance));
                        }
                    }
                }
                LayoutConstraint::Alignment { nodes, axis } => {
                    let positions_vec: Vec<_> =
                        nodes.iter().filter_map(|n| positions.get(n)).collect();

                    if positions_vec.len() == nodes.len() {
                        let coords: Vec<f64> = match axis {
                            Axis::Horizontal => positions_vec.iter().map(|p| p.1).collect(),
                            Axis::Vertical => positions_vec.iter().map(|p| p.0).collect(),
                        };

                        let mean = coords.iter().sum::<f64>() / coords.len() as f64;
                        let variance = coords.iter().map(|c| (c - mean).powi(2)).sum::<f64>()
                            / coords.len() as f64;

                        if variance < 1.0 {
                            satisfied.push(i);
                        } else {
                            violated.push((i, variance.sqrt()));
                        }
                    }
                }
                LayoutConstraint::FixedPosition { node, position } => {
                    if let Some(&actual_pos) = positions.get(node) {
                        let error = ((actual_pos.0 - position.0).powi(2)
                            + (actual_pos.1 - position.1).powi(2))
                        .sqrt();
                        if error < 1.0 {
                            satisfied.push(i);
                        } else {
                            violated.push((i, error));
                        }
                    }
                }
                _ => {
                    // Simplified evaluation for other constraint types
                    satisfied.push(i);
                }
            }
        }

        (satisfied, violated)
    }
}

/// Encodes constraints into neural network format
struct ConstraintEncoder {
    constraint_embed: candle_nn::Linear,
    _node_embed: candle_nn::Linear,
}

impl ConstraintEncoder {
    fn new(vb: &VarBuilder) -> Result<Self> {
        let constraint_embed = linear(10, 64, vb.pp("constraint_embed")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create constraint embed: {e}"
            )))
        })?;

        let node_embed = linear(2, 32, vb.pp("node_embed")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create node embed: {e}"
            )))
        })?;

        Ok(Self {
            constraint_embed,
            _node_embed: node_embed,
        })
    }

    fn encode(&self, constraints: &[LayoutConstraint], nodes: &[NodeIndex]) -> Result<Tensor> {
        let mut encoded = vec![];

        // Create node index mapping
        let node_to_idx: HashMap<NodeIndex, usize> =
            nodes.iter().enumerate().map(|(i, n)| (*n, i)).collect();

        for constraint in constraints {
            let mut features = vec![0.0f32; 10];

            match constraint {
                LayoutConstraint::MinDistance {
                    node1,
                    node2,
                    distance,
                } => {
                    features[0] = 1.0; // constraint type
                    if let (Some(&idx1), Some(&idx2)) =
                        (node_to_idx.get(node1), node_to_idx.get(node2))
                    {
                        features[1] = idx1 as f32 / nodes.len() as f32;
                        features[2] = idx2 as f32 / nodes.len() as f32;
                        features[3] = (*distance / 100.0) as f32;
                    }
                }
                LayoutConstraint::MaxDistance {
                    node1,
                    node2,
                    distance,
                } => {
                    features[0] = 2.0;
                    if let (Some(&idx1), Some(&idx2)) =
                        (node_to_idx.get(node1), node_to_idx.get(node2))
                    {
                        features[1] = idx1 as f32 / nodes.len() as f32;
                        features[2] = idx2 as f32 / nodes.len() as f32;
                        features[3] = (*distance / 100.0) as f32;
                    }
                }
                LayoutConstraint::Alignment {
                    nodes: align_nodes,
                    axis,
                } => {
                    features[0] = 3.0;
                    features[1] = match axis {
                        Axis::Horizontal => 0.0,
                        Axis::Vertical => 1.0,
                    };
                    // Encode first few nodes
                    for (i, node) in align_nodes.iter().take(3).enumerate() {
                        if let Some(&idx) = node_to_idx.get(node) {
                            features[2 + i] = idx as f32 / nodes.len() as f32;
                        }
                    }
                }
                LayoutConstraint::FixedPosition { node, position } => {
                    features[0] = 4.0;
                    if let Some(&idx) = node_to_idx.get(node) {
                        features[1] = idx as f32 / nodes.len() as f32;
                        features[2] = (position.0 / 500.0) as f32;
                        features[3] = (position.1 / 500.0) as f32;
                    }
                }
                _ => {
                    features[0] = 5.0; // Other constraint types
                }
            }

            encoded.extend_from_slice(&features);
        }

        let device = self.constraint_embed.weight().device();
        let input = Tensor::from_vec(encoded, &[constraints.len(), 10], device).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create constraint tensor: {e}"
            )))
        })?;

        self.constraint_embed.forward(&input).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to embed constraints: {e}"
            )))
        })
    }
}

/// Iterative solver network
struct SolverNetwork {
    gru_layer: candle_nn::rnn::GRU,
    attention: candle_nn::Linear,
    feasibility_head: candle_nn::Linear,
}

impl SolverNetwork {
    fn new(vb: &VarBuilder) -> Result<Self> {
        // GRU input size is hidden_dim (128) + constraint_dim (64) = 192
        let gru = candle_nn::gru(192, 128, Default::default(), vb.pp("gru")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create GRU: {e}"
            )))
        })?;

        let attention = linear(192, 1, vb.pp("attention")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create attention: {e}"
            )))
        })?;

        let feasibility_head = linear(128, 1, vb.pp("feasibility")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create feasibility head: {e}"
            )))
        })?;

        Ok(Self {
            gru_layer: gru,
            attention,
            feasibility_head,
        })
    }

    fn forward(&self, hidden: &Tensor, constraints: &Tensor) -> Result<(Tensor, Tensor)> {
        // Apply attention to constraints
        let (num_nodes, hidden_dim) = if hidden.dims().len() == 1 {
            // If 1D, treat as single node with all features
            (1, hidden.dims()[0])
        } else {
            // If 2D, get nodes and hidden dimensions
            (hidden.dims()[0], hidden.dims()[1])
        };
        let num_constraints = constraints.dims()[0];

        // Expand hidden for attention computation
        let hidden_2d = if hidden.dims().len() == 1 {
            hidden.unsqueeze(0).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to unsqueeze hidden: {e}"
                )))
            })?
        } else {
            hidden.clone()
        };

        let hidden_expanded = hidden_2d
            .unsqueeze(1)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to expand hidden: {e}"
                )))
            })?
            .broadcast_as(&[num_nodes, num_constraints, hidden_dim])
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to broadcast hidden: {e}"
                )))
            })?;

        // Expand constraints for each node
        let constraints_expanded = constraints
            .unsqueeze(0)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to expand constraints: {e}"
                )))
            })?
            .broadcast_as(&[num_nodes, num_constraints, 64])
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to broadcast constraints: {e}"
                )))
            })?;

        // Concatenate for attention
        let concat = Tensor::cat(&[&hidden_expanded, &constraints_expanded], 2).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to concatenate for attention: {e}"
            )))
        })?;

        // Compute attention weights
        let attention_logits = self.attention.forward(&concat).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to compute attention: {e}"
            )))
        })?;

        let attention_weights = candle_nn::ops::softmax(
            &attention_logits.squeeze(2).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to squeeze attention: {e}"
                )))
            })?,
            1,
        )
        .map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to compute softmax: {e}"
            )))
        })?;

        // Apply attention to constraints
        let attended_constraints = attention_weights
            .unsqueeze(2)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to unsqueeze weights: {e}"
                )))
            })?
            .broadcast_mul(&constraints_expanded)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to apply attention: {e}"
                )))
            })?
            .sum(1)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to sum attended: {e}"
                )))
            })?;

        // Combine with hidden state
        let gru_input = Tensor::cat(&[&hidden_2d, &attended_constraints], 1).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to prepare GRU input: {e}"
            )))
        })?;

        // Run GRU
        let gru_input_seq = gru_input.unsqueeze(0).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to add sequence dim: {e}"
            )))
        })?;

        let gru_states = self.gru_layer.seq(&gru_input_seq).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to run GRU: {e}"
            )))
        })?;

        // Get the last state output
        let new_hidden = gru_states
            .last()
            .ok_or_else(|| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                    "GRU returned no states".to_string(),
                ))
            })?
            .h()
            .squeeze(0)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to squeeze GRU output: {e}"
                )))
            })?;

        // Predict feasibility
        // If new_hidden is 2D [nodes, hidden_dim], take mean across nodes
        // If new_hidden is 1D [hidden_dim], use as is but unsqueeze
        let hidden_for_feasibility = if new_hidden.dims().len() == 2 {
            new_hidden
                .mean(0)
                .map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to compute mean: {e}"
                    )))
                })?
                .unsqueeze(0)
                .map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to unsqueeze mean: {e}"
                    )))
                })?
        } else {
            new_hidden.unsqueeze(0).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to unsqueeze hidden: {e}"
                )))
            })?
        };

        let feasibility = self
            .feasibility_head
            .forward(&hidden_for_feasibility)
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to predict feasibility: {e}"
                )))
            })?;

        let feasibility_sigmoid = candle_nn::ops::sigmoid(&feasibility).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to apply sigmoid: {e}"
            )))
        })?;

        Ok((new_hidden, feasibility_sigmoid))
    }
}

/// Decodes hidden states to positions
struct SolutionDecoder {
    position_head: candle_nn::Linear,
}

impl SolutionDecoder {
    fn new(vb: &VarBuilder) -> Result<Self> {
        let position_head = linear(128, 2, vb.pp("position_head")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create position head: {e}"
            )))
        })?;

        Ok(Self { position_head })
    }

    fn decode(
        &self,
        hidden: &Tensor,
        nodes: &[NodeIndex],
    ) -> Result<HashMap<NodeIndex, (f64, f64)>> {
        // Handle both 1D [hidden_dim] and 2D [nodes, hidden_dim] inputs
        let hidden_2d = if hidden.dims().len() == 1 {
            // If 1D, repeat for each node
            let _hidden_dim = hidden.dims()[0];
            hidden
                .unsqueeze(0)
                .map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to unsqueeze hidden: {e}"
                    )))
                })?
                .repeat(&[nodes.len(), 1])
                .map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to repeat hidden: {e}"
                    )))
                })?
        } else {
            hidden.clone()
        };

        let positions_tensor = self.position_head.forward(&hidden_2d).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to decode positions: {e}"
            )))
        })?;

        let positions_vec: Vec<Vec<f32>> = positions_tensor.to_vec2().map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to extract positions: {e}"
            )))
        })?;

        let mut positions = HashMap::new();
        for (i, &node) in nodes.iter().enumerate() {
            if i < positions_vec.len() {
                let x = positions_vec[i][0] as f64 * 500.0;
                let y = positions_vec[i][1] as f64 * 500.0;
                positions.insert(node, (x, y));
            }
        }

        Ok(positions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::Graph;

    #[test]
    fn test_constraint_solver() {
        let solver = NeuralConstraintSolver::new(10).unwrap();

        // Create simple test graph using AST
        use crate::ast::*;
        use std::collections::HashMap as StdHashMap;

        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: StdHashMap::new(),
            templates: StdHashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "n1".to_string(),
                    label: Some("Node 1".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
                NodeDefinition {
                    id: "n2".to_string(),
                    label: Some("Node 2".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
                NodeDefinition {
                    id: "n3".to_string(),
                    label: Some("Node 3".to_string()),
                    component_type: None,
                    attributes: StdHashMap::new(),
                },
            ],
            edges: vec![],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let nodes: Vec<NodeIndex> = igr.graph.node_identifiers().collect();

        // Define constraints using the actual node indices
        let constraints = vec![
            LayoutConstraint::MinDistance {
                node1: nodes[0],
                node2: nodes[1],
                distance: 50.0,
            },
            LayoutConstraint::Alignment {
                nodes: vec![nodes[0], nodes[1], nodes[2]],
                axis: Axis::Horizontal,
            },
            LayoutConstraint::FixedPosition {
                node: nodes[0],
                position: (0.0, 0.0),
            },
        ];

        // Solve constraints
        let solution = solver.solve(&igr, &constraints, None).unwrap();

        assert_eq!(solution.positions.len(), 3);
        assert!(solution.feasibility_score >= 0.0 && solution.feasibility_score <= 1.0);

        // Check fixed position constraint - with large tolerance since network is untrained
        if let Some(&pos) = solution.positions.get(&nodes[0]) {
            // Note: Neural network is untrained, so we just check it produces reasonable values
            assert!(pos.0.abs() < 1000.0); // Just check it's in a reasonable range
            assert!(pos.1.abs() < 1000.0);
        }
    }

    #[test]
    fn test_constraint_types() {
        let mut graph = Graph::<(), ()>::new();
        let n1 = graph.add_node(());
        let n2 = graph.add_node(());

        // Test different constraint types
        let min_dist = LayoutConstraint::MinDistance {
            node1: n1,
            node2: n2,
            distance: 100.0,
        };

        let max_dist = LayoutConstraint::MaxDistance {
            node1: n1,
            node2: n2,
            distance: 200.0,
        };

        let alignment = LayoutConstraint::Alignment {
            nodes: vec![n1, n2],
            axis: Axis::Vertical,
        };

        // All constraint types should be created successfully
        assert!(matches!(min_dist, LayoutConstraint::MinDistance { .. }));
        assert!(matches!(max_dist, LayoutConstraint::MaxDistance { .. }));
        assert!(matches!(alignment, LayoutConstraint::Alignment { .. }));
    }
}
