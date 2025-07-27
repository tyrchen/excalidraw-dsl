// src/layout/ml/gnn.rs
//! Graph Neural Network for layout prediction

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use candle_core::{Device, Module, Tensor};
use candle_nn::{linear, VarBuilder, VarMap};
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoNodeIdentifiers};
use std::collections::HashMap;
use std::path::Path;

/// GNN-based layout position prediction
#[derive(Clone, Debug)]
pub struct LayoutPrediction {
    pub positions: HashMap<NodeIndex, (f64, f64)>,
    pub confidence: f64,
}

/// Graph Attention Network layer for node embedding
struct GATLayer {
    attention_weights: candle_nn::Linear,
    value_transform: candle_nn::Linear,
    output_dim: usize,
}

impl GATLayer {
    fn new(input_dim: usize, output_dim: usize, vb: VarBuilder) -> Result<Self> {
        let attention_weights = linear(input_dim * 2, 1, vb.pp("attention")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create attention weights: {e}"
            )))
        })?;

        let value_transform = linear(input_dim, output_dim, vb.pp("value")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create value transform: {e}"
            )))
        })?;

        Ok(Self {
            attention_weights,
            value_transform,
            output_dim,
        })
    }

    fn forward(&self, node_features: &Tensor, edge_indices: &Tensor) -> Result<Tensor> {
        // Get number of nodes
        let num_nodes = node_features.dims()[0];
        let feature_dim = node_features.dims()[1];

        // Initialize output tensor
        let device = node_features.device();
        let mut output = Tensor::zeros(
            &[num_nodes, self.output_dim],
            candle_core::DType::F32,
            device,
        )
        .map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create output tensor: {e}"
            )))
        })?;

        // Get edge source and target indices
        let edges_vec: Vec<i64> = edge_indices.to_vec2().map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to extract edge indices: {e}"
            )))
        })?[0]
            .clone();

        if edges_vec.len() % 2 != 0 {
            return Err(EDSLError::Layout(
                crate::error::LayoutError::CalculationFailed(
                    "Invalid edge indices format".to_string(),
                ),
            ));
        }

        // Process each node's neighbors
        for node_idx in 0..num_nodes {
            let node_feature = node_features.get(node_idx).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to get node features: {e}"
                )))
            })?;

            // Find neighbors
            let mut neighbor_features = vec![];
            for i in (0..edges_vec.len()).step_by(2) {
                if edges_vec[i + 1] as usize == node_idx {
                    let neighbor_idx = edges_vec[i] as usize;
                    let neighbor_feature = node_features.get(neighbor_idx).map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to get neighbor features: {e}"
                        )))
                    })?;
                    neighbor_features.push(neighbor_feature);
                }
            }

            if neighbor_features.is_empty() {
                // No neighbors, use self-attention only
                let node_feature_2d = node_feature.unsqueeze(0).map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to unsqueeze node feature: {e}"
                    )))
                })?;
                let transformed = self
                    .value_transform
                    .forward(&node_feature_2d)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to transform features: {e}"
                        )))
                    })?;
                output = output
                    .slice_assign(&[node_idx..node_idx + 1, 0..self.output_dim], &transformed)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to assign output: {e}"
                        )))
                    })?;
            } else {
                // Compute attention scores for neighbors
                let mut attention_scores = vec![];

                for neighbor_feature in &neighbor_features {
                    // Concatenate node and neighbor features
                    let concat =
                        Tensor::cat(&[&node_feature, neighbor_feature], 0).map_err(|e| {
                            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                                format!("Failed to concatenate features: {e}"),
                            ))
                        })?;

                    let score = self
                        .attention_weights
                        .forward(&concat.unsqueeze(0).map_err(|e| {
                            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                                format!("Failed to unsqueeze concat: {e}"),
                            ))
                        })?)
                        .map_err(|e| {
                            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                                format!("Failed to compute attention score: {e}"),
                            ))
                        })?
                        .squeeze(0)
                        .map_err(|e| {
                            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                                format!("Failed to squeeze attention score: {e}"),
                            ))
                        })?;

                    attention_scores.push(score);
                }

                // Softmax over attention scores
                let scores_tensor = Tensor::cat(&attention_scores, 0).map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to concatenate scores: {e}"
                    )))
                })?;

                let attention_probs = candle_nn::ops::softmax(&scores_tensor, 0).map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to compute softmax: {e}"
                    )))
                })?;

                // Weighted sum of neighbor features
                let mut aggregated = Tensor::zeros(&[feature_dim], candle_core::DType::F32, device)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to create aggregated tensor: {e}"
                        )))
                    })?;

                let probs_vec: Vec<f32> = attention_probs.to_vec1().map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to extract attention probs: {e}"
                    )))
                })?;

                for (i, neighbor_feature) in neighbor_features.iter().enumerate() {
                    let weighted =
                        neighbor_feature
                            .affine(probs_vec[i] as f64, 0.0)
                            .map_err(|e| {
                                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                                    format!("Failed to weight neighbor feature: {e}"),
                                ))
                            })?;
                    aggregated = (aggregated + weighted).map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to aggregate features: {e}"
                        )))
                    })?;
                }

                // Transform aggregated features
                let aggregated_2d = aggregated.unsqueeze(0).map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to unsqueeze aggregated features: {e}"
                    )))
                })?;
                let transformed = self.value_transform.forward(&aggregated_2d).map_err(|e| {
                    EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                        "Failed to transform aggregated features: {e}"
                    )))
                })?;

                output = output
                    .slice_assign(&[node_idx..node_idx + 1, 0..self.output_dim], &transformed)
                    .map_err(|e| {
                        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                            "Failed to assign output: {e}"
                        )))
                    })?;
            }
        }

        Ok(output)
    }
}

/// GNN model for layout prediction
pub struct GNNLayoutPredictor {
    var_map: VarMap,
    device: Device,
    gat_layers: Vec<GATLayer>,
    position_head: candle_nn::Linear,
    confidence_head: candle_nn::Linear,
    _hidden_dim: usize,
}

impl GNNLayoutPredictor {
    /// Create a new GNN layout predictor
    pub fn new(input_dim: usize, hidden_dim: usize, num_layers: usize) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        // Create GAT layers
        let mut gat_layers = vec![];
        for i in 0..num_layers {
            let in_dim = if i == 0 { input_dim } else { hidden_dim };
            let layer = GATLayer::new(in_dim, hidden_dim, vb.pp(format!("gat_layer_{i}")))?;
            gat_layers.push(layer);
        }

        // Position prediction head (outputs x, y coordinates)
        let position_head = linear(hidden_dim, 2, vb.pp("position_head")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create position head: {e}"
            )))
        })?;

        // Confidence prediction head
        let confidence_head = linear(hidden_dim, 1, vb.pp("confidence_head")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create confidence head: {e}"
            )))
        })?;

        Ok(Self {
            var_map,
            device,
            gat_layers,
            position_head,
            confidence_head,
            _hidden_dim: hidden_dim,
        })
    }

    /// Load model from file
    pub fn from_path(path: &Path) -> Result<Self> {
        let device = Device::Cpu;
        let mut var_map = VarMap::new();

        // Load weights
        var_map.load(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to load GNN model weights: {e}"
            )))
        })?;

        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        // Recreate model architecture (assuming default dimensions)
        let input_dim = 32;
        let hidden_dim = 64;
        let num_layers = 3;

        let mut gat_layers = vec![];
        for i in 0..num_layers {
            let in_dim = if i == 0 { input_dim } else { hidden_dim };
            let layer = GATLayer::new(in_dim, hidden_dim, vb.pp(format!("gat_layer_{i}")))?;
            gat_layers.push(layer);
        }

        let position_head = linear(hidden_dim, 2, vb.pp("position_head")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create position head: {e}"
            )))
        })?;

        let confidence_head = linear(hidden_dim, 1, vb.pp("confidence_head")).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create confidence head: {e}"
            )))
        })?;

        Ok(Self {
            var_map,
            device,
            gat_layers,
            position_head,
            confidence_head,
            _hidden_dim: hidden_dim,
        })
    }

    /// Save model to file
    pub fn save(&self, path: &Path) -> Result<()> {
        self.var_map.save(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to save GNN model: {e}"
            )))
        })
    }

    /// Predict layout for a graph
    pub fn predict_layout(&self, igr: &IntermediateGraph) -> Result<LayoutPrediction> {
        // Extract node features
        let node_features = self.extract_node_features(igr)?;

        // Extract edge indices
        let edge_indices = self.extract_edge_indices(igr)?;

        // Forward pass through GAT layers
        let mut hidden = node_features;
        for layer in &self.gat_layers {
            hidden = layer.forward(&hidden, &edge_indices)?;
            hidden = hidden.relu().map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to apply ReLU: {e}"
                )))
            })?;
        }

        // Predict positions
        let positions_tensor = self.position_head.forward(&hidden).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to predict positions: {e}"
            )))
        })?;

        // Predict confidence
        let confidence_tensor = self.confidence_head.forward(&hidden).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to predict confidence: {e}"
            )))
        })?;

        // Convert to HashMap
        let positions_vec: Vec<Vec<f32>> = positions_tensor.to_vec2().map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to extract positions: {e}"
            )))
        })?;

        let confidence_vec: Vec<f32> = confidence_tensor
            .flatten_all()
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to flatten confidence: {e}"
                )))
            })?
            .to_vec1()
            .map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to extract confidence: {e}"
                )))
            })?;

        let mut positions = HashMap::new();
        for (i, node) in igr.graph.node_identifiers().enumerate() {
            if i < positions_vec.len() {
                positions.insert(
                    node,
                    (positions_vec[i][0] as f64, positions_vec[i][1] as f64),
                );
            }
        }

        // Average confidence across all nodes
        let confidence = if !confidence_vec.is_empty() {
            let sum: f32 = confidence_vec.iter().map(|c| c.tanh()).sum();
            (sum / confidence_vec.len() as f32) as f64
        } else {
            0.5
        };

        Ok(LayoutPrediction {
            positions,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    /// Extract node features for GNN
    fn extract_node_features(&self, igr: &IntermediateGraph) -> Result<Tensor> {
        let num_nodes = igr.graph.node_count();
        let feature_dim = 32; // Simple feature dimension

        let mut features = vec![0.0f32; num_nodes * feature_dim];

        for (i, node_idx) in igr.graph.node_identifiers().enumerate() {
            let base_idx = i * feature_dim;

            // Node type features
            if let Some(node) = igr.graph.node_weight(node_idx) {
                // Basic node type encoding
                features[base_idx] = if node.is_virtual_container { 1.0 } else { 0.0 };

                // Degree features
                let in_degree = igr
                    .graph
                    .edges_directed(node_idx, petgraph::Direction::Incoming)
                    .count();
                let out_degree = igr
                    .graph
                    .edges_directed(node_idx, petgraph::Direction::Outgoing)
                    .count();
                features[base_idx + 1] = in_degree as f32 / 10.0;
                features[base_idx + 2] = out_degree as f32 / 10.0;

                // Label length as feature
                if !node.label.is_empty() {
                    features[base_idx + 3] = node.label.len() as f32 / 50.0;
                }

                // Random features for now (would be replaced with actual embeddings)
                for j in 4..feature_dim {
                    features[base_idx + j] = ((i * j) % 7) as f32 / 7.0;
                }
            }
        }

        Tensor::from_vec(features, &[num_nodes, feature_dim], &self.device).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to create node features tensor: {e}"
            )))
        })
    }

    /// Extract edge indices for GNN
    fn extract_edge_indices(&self, igr: &IntermediateGraph) -> Result<Tensor> {
        let mut edge_list = vec![];

        // Create node index mapping
        let node_to_idx: HashMap<NodeIndex, usize> = igr
            .graph
            .node_identifiers()
            .enumerate()
            .map(|(i, node)| (node, i))
            .collect();

        // Extract edges
        for edge in igr.graph.edge_references() {
            if let (Some(&src_idx), Some(&dst_idx)) = (
                node_to_idx.get(&edge.source()),
                node_to_idx.get(&edge.target()),
            ) {
                edge_list.push(src_idx as i64);
                edge_list.push(dst_idx as i64);
            }
        }

        if edge_list.is_empty() {
            // No edges, return empty tensor
            Tensor::zeros(&[2, 0], candle_core::DType::I64, &self.device).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to create empty edge indices: {e}"
                )))
            })
        } else {
            let edge_count = edge_list.len() / 2;
            Tensor::from_vec(edge_list, &[2, edge_count], &self.device).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to create edge indices tensor: {e}"
                )))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap;

    #[test]
    fn test_gnn_predictor_creation() {
        let predictor = GNNLayoutPredictor::new(32, 64, 3).unwrap();
        assert_eq!(predictor._hidden_dim, 64);
        assert_eq!(predictor.gat_layers.len(), 3);
    }

    #[test]
    fn test_gnn_layout_prediction() {
        let predictor = GNNLayoutPredictor::new(32, 64, 2).unwrap();

        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "c".to_string(),
                    label: Some("Node C".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![
                EdgeDefinition {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "b".to_string(),
                    to: "c".to_string(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
            ],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let prediction = predictor.predict_layout(&igr).unwrap();

        assert_eq!(prediction.positions.len(), 3);
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
    }
}
