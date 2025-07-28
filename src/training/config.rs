// src/training/config.rs
//! Training configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub phase: TrainingPhase,
    pub output_dir: PathBuf,
    pub device: DeviceConfig,
    pub data_config: DataConfig,
    pub gnn_config: GNNConfig,
    pub rl_config: RLConfig,
    pub constraint_config: ConstraintConfig,
    pub enhanced_config: EnhancedConfig,
    pub evaluation_config: EvaluationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingPhase {
    DataGeneration,
    GNNTraining,
    RLTraining,
    ConstraintTraining,
    EnhancedTraining,
    Evaluation,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub use_metal: bool,
    pub device_id: usize,
    pub memory_limit_gb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub num_samples: usize,
    pub graph_sizes: Vec<usize>,
    pub graph_types: Vec<GraphType>,
    pub augmentation_factor: usize,
    pub validation_split: f32,
    pub test_split: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphType {
    Chain,
    Tree,
    Star,
    Mesh,
    Hierarchy,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
    pub num_layers: usize,
    pub dropout: f32,
    pub learning_rate: f64,
    pub batch_size: usize,
    pub max_epochs: usize,
    pub early_stopping_patience: usize,
    pub checkpoint_every: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GNNConfig {
    pub model: ModelConfig,
    pub attention_heads: usize,
    pub use_residual: bool,
    pub use_batch_norm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLConfig {
    pub model: ModelConfig,
    pub num_episodes: usize,
    pub max_steps_per_episode: usize,
    pub exploration_rate: f64,
    pub exploration_decay: f64,
    pub discount_factor: f64,
    pub target_update_frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub model: ModelConfig,
    pub max_iterations: usize,
    pub convergence_threshold: f64,
    pub constraint_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedConfig {
    pub gnn_weight: f64,
    pub rl_weight: f64,
    pub constraint_weight: f64,
    pub num_integration_epochs: usize,
    pub ensemble_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub test_graphs: Vec<String>,
    pub metrics: Vec<EvaluationMetric>,
    pub benchmark_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationMetric {
    EdgeCrossings,
    NodeOverlaps,
    SymmetryScore,
    CompactnessScore,
    AspectRatio,
    TotalEdgeLength,
    InferenceTime,
    MemoryUsage,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            phase: TrainingPhase::All,
            output_dir: PathBuf::from("ml_training_output"),
            device: DeviceConfig::default(),
            data_config: DataConfig::default(),
            gnn_config: GNNConfig::default(),
            rl_config: RLConfig::default(),
            constraint_config: ConstraintConfig::default(),
            enhanced_config: EnhancedConfig::default(),
            evaluation_config: EvaluationConfig::default(),
        }
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            use_metal: cfg!(target_os = "macos"),
            device_id: 0,
            memory_limit_gb: 16,
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            num_samples: 10000,
            graph_sizes: vec![5, 10, 20, 50, 100, 200],
            graph_types: vec![
                GraphType::Chain,
                GraphType::Tree,
                GraphType::Star,
                GraphType::Mesh,
                GraphType::Hierarchy,
            ],
            augmentation_factor: 3,
            validation_split: 0.15,
            test_split: 0.15,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            input_dim: 32,
            hidden_dim: 128,
            output_dim: 2,
            num_layers: 3,
            dropout: 0.1,
            learning_rate: 0.001,
            batch_size: 32,
            max_epochs: 100,
            early_stopping_patience: 10,
            checkpoint_every: 10,
        }
    }
}

impl Default for GNNConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                hidden_dim: 128,
                num_layers: 4,
                ..ModelConfig::default()
            },
            attention_heads: 8,
            use_residual: true,
            use_batch_norm: true,
        }
    }
}

impl Default for RLConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                hidden_dim: 256,
                output_dim: 10, // Action space size
                ..ModelConfig::default()
            },
            num_episodes: 1000,
            max_steps_per_episode: 100,
            exploration_rate: 0.1,
            exploration_decay: 0.995,
            discount_factor: 0.99,
            target_update_frequency: 100,
        }
    }
}

impl Default for ConstraintConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                hidden_dim: 192,
                num_layers: 3,
                ..ModelConfig::default()
            },
            max_iterations: 50,
            convergence_threshold: 0.001,
            constraint_weight: 1.0,
        }
    }
}

impl Default for EnhancedConfig {
    fn default() -> Self {
        Self {
            gnn_weight: 0.4,
            rl_weight: 0.4,
            constraint_weight: 0.2,
            num_integration_epochs: 50,
            ensemble_size: 3,
        }
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            test_graphs: vec![
                "a [Node A]; b [Node B]; c [Node C]; a -> b -> c;".to_string(),
                "root [Root]; left [Left]; right [Right]; l1 [L1]; l2 [L2]; r1 [R1]; r2 [R2]; root -> left; root -> right; left -> l1; left -> l2; right -> r1; right -> r2;".to_string(),
                "center [Center]; n1 [Node 1]; n2 [Node 2]; n3 [Node 3]; n4 [Node 4]; center -> n1; center -> n2; center -> n3; center -> n4;".to_string(),
                "n1 [Node 1]; n2 [Node 2]; n3 [Node 3]; n4 [Node 4]; n1 -> n2; n1 -> n3; n2 -> n3; n2 -> n4; n3 -> n4; n3 -> n1;".to_string(),
            ],
            metrics: vec![
                EvaluationMetric::EdgeCrossings,
                EvaluationMetric::NodeOverlaps,
                EvaluationMetric::SymmetryScore,
                EvaluationMetric::CompactnessScore,
                EvaluationMetric::InferenceTime,
            ],
            benchmark_strategies: vec![
                "dagre".to_string(),
                "force".to_string(),
                "ml_basic".to_string(),
            ],
        }
    }
}

impl TrainingConfig {
    /// Load configuration from file
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &std::path::Path) -> crate::Result<()> {
        let content = if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            serde_yaml::to_string(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Create M4-optimized configuration
    pub fn m4_optimized() -> Self {
        let mut config = Self::default();

        // Optimize for Apple M4
        config.device.use_metal = true;
        config.device.memory_limit_gb = 32; // Assume M4 Pro/Max

        // Larger batch sizes for unified memory
        config.gnn_config.model.batch_size = 64;
        config.rl_config.model.batch_size = 128;
        config.constraint_config.model.batch_size = 64;

        // Deeper networks for M4's compute power
        config.gnn_config.model.hidden_dim = 256;
        config.gnn_config.model.num_layers = 6;
        config.gnn_config.attention_heads = 16;

        config.rl_config.model.hidden_dim = 512;
        config.rl_config.num_episodes = 2000;

        config.constraint_config.model.hidden_dim = 384;

        // More training data for better models
        config.data_config.num_samples = 50000;
        config.data_config.augmentation_factor = 5;

        config
    }

    /// Create quick test configuration
    pub fn quick_test() -> Self {
        let mut config = Self::default();

        // Smaller for quick testing
        config.data_config.num_samples = 1000;
        config.data_config.graph_sizes = vec![5, 10, 15]; // Much smaller graphs
        config.data_config.augmentation_factor = 2; // Less augmentation
        config.gnn_config.model.max_epochs = 10;
        config.rl_config.num_episodes = 100;
        config.constraint_config.model.max_epochs = 10;
        config.enhanced_config.num_integration_epochs = 5;

        // Smaller models
        config.gnn_config.model.hidden_dim = 64;
        config.rl_config.model.hidden_dim = 128;
        config.constraint_config.model.hidden_dim = 96;

        config
    }
}
