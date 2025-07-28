// src/layout/ml/model.rs
//! ML model abstractions for layout prediction and quality estimation

use super::Network;
use crate::error::{EDSLError, Result};
use candle_core::{Device, Module, Tensor};
use candle_nn::{VarBuilder, VarMap};
use ndarray::Array1;
use std::path::Path;

/// Type of ML model used for predictions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    StrategySelector,
    PerformancePredictor,
    QualityPredictor,
}

/// Performance metrics predicted by the model
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub expected_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
}

/// Quality metrics predicted by the model
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub edge_crossing_score: f64,
    pub node_overlap_score: f64,
    pub space_utilization: f64,
    pub symmetry_score: f64,
    pub aesthetic_score: f64,
}

/// Neural network model for layout predictions
pub struct LayoutPredictionModel {
    #[allow(dead_code)]
    model_type: ModelType,
    network: Network,
    device: Device,
    var_map: VarMap,
}

impl LayoutPredictionModel {
    /// Create a new model with default architecture
    pub fn new(model_type: ModelType) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        let network = match model_type {
            ModelType::StrategySelector => Self::build_strategy_selector_network(&vb)?,
            ModelType::PerformancePredictor => Self::build_performance_predictor_network(&vb)?,
            ModelType::QualityPredictor => Self::build_quality_predictor_network(&vb)?,
        };

        Ok(Self {
            model_type,
            network,
            device,
            var_map,
        })
    }

    /// Load a model from a file
    pub fn from_path(model_type: ModelType, path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(EDSLError::Layout(
                crate::error::LayoutError::CalculationFailed(format!(
                    "Model file not found: {path:?}"
                )),
            ));
        }

        let device = Device::Cpu;
        let mut var_map = VarMap::new();

        // Load weights from file
        var_map.load(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to load model weights: {e}"
            )))
        })?;

        let vb = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        let network = match model_type {
            ModelType::StrategySelector => Self::build_strategy_selector_network(&vb)?,
            ModelType::PerformancePredictor => Self::build_performance_predictor_network(&vb)?,
            ModelType::QualityPredictor => Self::build_quality_predictor_network(&vb)?,
        };

        Ok(Self {
            model_type,
            network,
            device,
            var_map,
        })
    }

    /// Save the model to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        self.var_map.save(path).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to save model: {e}"
            )))
        })
    }

    /// Predict using the model
    pub fn predict(&self, features: &Array1<f64>) -> Result<Vec<f64>> {
        // Convert features to tensor (as f32 to match model weights)
        let features_f32: Vec<f32> = features.iter().map(|&x| x as f32).collect();
        let input =
            Tensor::from_vec(features_f32, &[1, features.len()], &self.device).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to create input tensor: {e}"
                )))
            })?;

        // Forward pass
        let output = (self.network)(&input).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Model forward pass failed: {e}"
            )))
        })?;

        // Convert output to vector (from f32 back to f64)
        // First flatten the tensor if it's 2D [1, n] to 1D [n]
        let output_flat = if output.dims().len() == 2 && output.dims()[0] == 1 {
            output.squeeze(0).map_err(|e| {
                EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                    "Failed to squeeze output tensor: {e}"
                )))
            })?
        } else {
            output
        };

        let output_vec_f32: Vec<f32> = output_flat.to_vec1().map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to convert output tensor: {e}"
            )))
        })?;

        // Convert back to f64
        let output_vec: Vec<f64> = output_vec_f32.iter().map(|&x| x as f64).collect();

        Ok(output_vec)
    }

    /// Build the strategy selector network
    fn build_strategy_selector_network(vb: &VarBuilder) -> Result<Network> {
        // Simple feedforward network for strategy selection
        // Input: 19 features
        // Hidden layers: 64 -> 32
        // Output: 4 strategies (dagre, force, elk, adaptive)

        use candle_nn::linear;

        let hidden1 = linear(19, 64, vb.pp("fc1")).map_err(Self::candle_error)?;
        let hidden2 = linear(64, 32, vb.pp("fc2")).map_err(Self::candle_error)?;
        let output = linear(32, 4, vb.pp("fc3")).map_err(Self::candle_error)?;

        let network = move |x: &Tensor| -> candle_core::Result<Tensor> {
            let x = hidden1.forward(x)?;
            let x = x.relu()?;
            let x = hidden2.forward(&x)?;
            let x = x.relu()?;
            let x = output.forward(&x)?;
            // Softmax over dimension 1
            let exp_x = x.exp()?;
            let sum_exp = exp_x.sum_keepdim(1)?;
            exp_x.broadcast_div(&sum_exp)
        };

        Ok(Box::new(network))
    }

    /// Build the performance predictor network
    fn build_performance_predictor_network(vb: &VarBuilder) -> Result<Network> {
        // Network for predicting performance metrics
        // Input: 19 features
        // Hidden layers: 64 -> 32
        // Output: 3 metrics (time, memory, cpu)

        use candle_nn::linear;

        let hidden1 = linear(19, 64, vb.pp("fc1")).map_err(Self::candle_error)?;
        let hidden2 = linear(64, 32, vb.pp("fc2")).map_err(Self::candle_error)?;
        let output = linear(32, 3, vb.pp("fc3")).map_err(Self::candle_error)?;

        let network = move |x: &Tensor| -> candle_core::Result<Tensor> {
            let x = hidden1.forward(x)?;
            let x = x.relu()?;
            let x = hidden2.forward(&x)?;
            let x = x.relu()?;
            output.forward(&x)
        };

        Ok(Box::new(network))
    }

    /// Build the quality predictor network
    fn build_quality_predictor_network(vb: &VarBuilder) -> Result<Network> {
        // Network for predicting quality metrics
        // Input: 19 features
        // Hidden layers: 64 -> 32
        // Output: 5 metrics (edge_crossing, overlap, space, symmetry, aesthetic)

        use candle_nn::linear;

        let hidden1 = linear(19, 64, vb.pp("fc1")).map_err(Self::candle_error)?;
        let hidden2 = linear(64, 32, vb.pp("fc2")).map_err(Self::candle_error)?;
        let output = linear(32, 5, vb.pp("fc3")).map_err(Self::candle_error)?;

        let network = move |x: &Tensor| -> candle_core::Result<Tensor> {
            let x = hidden1.forward(x)?;
            let x = x.relu()?;
            let x = hidden2.forward(&x)?;
            let x = x.relu()?;
            let x = output.forward(&x)?;
            // Sigmoid activation for quality scores
            (x.neg()?.exp()? + 1.0)?.recip()
        };

        Ok(Box::new(network))
    }

    fn candle_error(e: candle_core::Error) -> EDSLError {
        EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
            "Candle error: {e}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = LayoutPredictionModel::new(ModelType::StrategySelector).unwrap();
        assert_eq!(model.model_type, ModelType::StrategySelector);
    }

    #[test]
    fn test_model_prediction() {
        let model = LayoutPredictionModel::new(ModelType::StrategySelector).unwrap();

        // Create dummy features
        let features = Array1::from_vec(vec![0.5; 19]);

        let prediction = model.predict(&features).unwrap();

        // Should output 4 probabilities that sum to 1
        assert_eq!(prediction.len(), 4);
        let sum: f64 = prediction.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }
}
