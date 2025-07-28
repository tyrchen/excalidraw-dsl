// src/training/data.rs
//! Training data generation and management

// use crate::ast::*; // Unused import
use crate::igr::IntermediateGraph;
use crate::layout::{DagreLayout, ForceLayout, LayoutEngineAdapter, LayoutStrategy};
use crate::parser::parse_edsl;
use crate::training::config::{DataConfig, GraphType};
use crate::Result;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStats {
    pub total_samples: usize,
    pub training_samples: usize,
    pub validation_samples: usize,
    pub test_samples: usize,
    pub graph_type_distribution: HashMap<String, usize>,
    pub size_distribution: HashMap<usize, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub graph_edsl: String,
    pub graph_features: Vec<f32>,
    pub target_positions: HashMap<String, (f64, f64)>,
    pub layout_strategy: String,
    pub quality_score: f64,
    pub metadata: SampleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub graph_type: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub generation_timestamp: String,
}

pub struct TrainingDataGenerator {
    config: DataConfig,
}

impl TrainingDataGenerator {
    pub fn new(config: &DataConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub fn generate_datasets(&mut self, output_dir: &Path) -> Result<DatasetStats> {
        println!("📊 Generating training datasets...");

        // Create output directories
        let data_dir = output_dir.join("data");
        std::fs::create_dir_all(&data_dir)?;

        let mut all_samples = Vec::new();
        let mut graph_type_distribution = HashMap::new();
        let mut size_distribution = HashMap::new();

        // Generate samples for each graph type and size
        for graph_type in &self.config.graph_types {
            for &size in &self.config.graph_sizes {
                let samples_per_type_size = self.config.num_samples
                    / (self.config.graph_types.len() * self.config.graph_sizes.len());

                println!(
                    "  Generating {graph_type:?} graphs with {size} nodes ({samples_per_type_size} samples)"
                );

                for i in 0..samples_per_type_size {
                    // Generate base graph
                    let graph_edsl = self.generate_graph(graph_type, size, i)?;

                    // Generate training samples with different layout strategies
                    let samples = self.generate_samples_for_graph(&graph_edsl, graph_type, size)?;
                    all_samples.extend(samples);

                    // Update distributions
                    *graph_type_distribution
                        .entry(format!("{graph_type:?}"))
                        .or_insert(0) += 1;
                    *size_distribution.entry(size).or_insert(0) += 1;
                }
            }
        }

        // Apply data augmentation
        if self.config.augmentation_factor > 1 {
            println!(
                "  Applying data augmentation (factor: {})",
                self.config.augmentation_factor
            );
            all_samples = self.augment_data(all_samples)?;
        }

        // Split into train/validation/test
        let (train_samples, val_samples, test_samples) = self.split_data(all_samples)?;

        // Save datasets
        self.save_dataset(&train_samples, &data_dir.join("train.json"))?;
        self.save_dataset(&val_samples, &data_dir.join("validation.json"))?;
        self.save_dataset(&test_samples, &data_dir.join("test.json"))?;

        let stats = DatasetStats {
            total_samples: train_samples.len() + val_samples.len() + test_samples.len(),
            training_samples: train_samples.len(),
            validation_samples: val_samples.len(),
            test_samples: test_samples.len(),
            graph_type_distribution,
            size_distribution,
        };

        // Save statistics
        let stats_content = serde_json::to_string_pretty(&stats)?;
        std::fs::write(data_dir.join("stats.json"), stats_content)?;

        println!("✅ Dataset generation completed:");
        println!("  Training samples: {}", stats.training_samples);
        println!("  Validation samples: {}", stats.validation_samples);
        println!("  Test samples: {}", stats.test_samples);

        Ok(stats)
    }

    fn generate_graph(&self, graph_type: &GraphType, size: usize, seed: usize) -> Result<String> {
        let graph_name = format!("{graph_type:?}_{size}_{seed}");

        match graph_type {
            GraphType::Chain => self.generate_chain_graph(&graph_name, size),
            GraphType::Tree => self.generate_tree_graph(&graph_name, size),
            GraphType::Star => self.generate_star_graph(&graph_name, size),
            GraphType::Mesh => self.generate_mesh_graph(&graph_name, size),
            GraphType::Hierarchy => self.generate_hierarchy_graph(&graph_name, size),
            GraphType::Random => self.generate_random_graph(&graph_name, size, seed),
        }
    }

    fn generate_chain_graph(&self, _name: &str, size: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes
        for i in 0..size {
            edsl.push_str(&format!("n{i} [Node {i}]\n"));
        }

        edsl.push('\n');

        // Create chain connections
        for i in 0..size.saturating_sub(1) {
            edsl.push_str(&format!("n{} -> n{}\n", i, i + 1));
        }

        Ok(edsl)
    }

    fn generate_tree_graph(&self, _name: &str, size: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes
        for i in 0..size {
            edsl.push_str(&format!("n{i} [Node {i}]\n"));
        }

        edsl.push('\n');

        // Create tree structure (binary tree)
        for i in 1..size {
            let parent = (i - 1) / 2;
            edsl.push_str(&format!("n{parent} -> n{i}\n"));
        }

        Ok(edsl)
    }

    fn generate_star_graph(&self, _name: &str, size: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes
        for i in 0..size {
            edsl.push_str(&format!("n{i} [Node {i}]\n"));
        }

        edsl.push('\n');

        // Create star connections (node 0 is center)
        for i in 1..size {
            edsl.push_str(&format!("n0 -> n{i}\n"));
        }

        Ok(edsl)
    }

    fn generate_mesh_graph(&self, _name: &str, size: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes
        for i in 0..size {
            edsl.push_str(&format!("n{i} [Node {i}]\n"));
        }

        edsl.push('\n');

        // Create mesh connections (each node connects to next node and one additional)
        for i in 0..size {
            // Connect to next node
            if i + 1 < size {
                edsl.push_str(&format!("n{} -> n{}\n", i, i + 1));
            }
            // Connect to one additional node to create mesh structure
            if i + 2 < size {
                edsl.push_str(&format!("n{} -> n{}\n", i, i + 2));
            }
        }

        Ok(edsl)
    }

    fn generate_hierarchy_graph(&self, _name: &str, size: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes with hierarchy levels
        let levels = 3;
        let nodes_per_level = size / levels;

        for i in 0..size {
            let level = i / nodes_per_level;
            edsl.push_str(&format!("n{i} [L{level} Node {i}]\n"));
        }

        edsl.push('\n');

        // Create hierarchical connections
        for level in 0..levels - 1 {
            let current_start = level * nodes_per_level;
            let next_start = (level + 1) * nodes_per_level;

            for i in current_start..current_start + nodes_per_level {
                for j in next_start..next_start + nodes_per_level {
                    if j < size {
                        edsl.push_str(&format!("n{i} -> n{j}\n"));
                    }
                }
            }
        }

        Ok(edsl)
    }

    fn generate_random_graph(&self, _name: &str, size: usize, seed: usize) -> Result<String> {
        let mut edsl = String::new();

        // Create nodes
        for i in 0..size {
            edsl.push_str(&format!("n{i} [Node {i}]\n"));
        }

        edsl.push('\n');

        // Create random connections (pseudo-random based on seed)
        let edge_probability = 0.3;
        for i in 0..size {
            for j in i + 1..size {
                // Simple pseudo-random number generator
                let hash = ((i * 31 + j * 17 + seed * 13) % 100) as f64 / 100.0;
                if hash < edge_probability {
                    edsl.push_str(&format!("n{i} -> n{j}\n"));
                }
            }
        }

        Ok(edsl)
    }

    fn generate_samples_for_graph(
        &self,
        graph_edsl: &str,
        graph_type: &GraphType,
        _size: usize,
    ) -> Result<Vec<TrainingSample>> {
        let mut samples = Vec::new();

        // Parse the graph
        let parsed = parse_edsl(graph_edsl)?;
        let igr = IntermediateGraph::from_ast(parsed)?;

        // Generate samples with different layout strategies
        let strategies = vec![
            (
                "dagre",
                Box::new(LayoutEngineAdapter::new(DagreLayout::new())) as Box<dyn LayoutStrategy>,
            ),
            (
                "force",
                Box::new(LayoutEngineAdapter::new(ForceLayout::new())) as Box<dyn LayoutStrategy>,
            ),
        ];

        for (strategy_name, strategy) in strategies {
            let mut igr_copy = igr.clone();

            // Apply layout
            strategy.apply(&mut igr_copy, &Default::default())?;

            // Extract features and positions
            let graph_features = self.extract_graph_features(&igr_copy)?;
            let target_positions = self.extract_positions(&igr_copy)?;
            let quality_score = self.calculate_quality_score(&igr_copy);

            let sample = TrainingSample {
                graph_edsl: graph_edsl.to_string(),
                graph_features,
                target_positions,
                layout_strategy: strategy_name.to_string(),
                quality_score,
                metadata: SampleMetadata {
                    graph_type: format!("{graph_type:?}"),
                    node_count: igr_copy.graph.node_count(),
                    edge_count: igr_copy.graph.edge_count(),
                    density: igr_copy.graph.edge_count() as f64
                        / (igr_copy.graph.node_count() as f64
                            * (igr_copy.graph.node_count() - 1) as f64
                            / 2.0),
                    generation_timestamp: chrono::Utc::now().to_rfc3339(),
                },
            };

            samples.push(sample);
        }

        Ok(samples)
    }

    fn extract_graph_features(&self, igr: &IntermediateGraph) -> Result<Vec<f32>> {
        let mut features = Vec::new();

        // Basic graph statistics
        features.push(igr.graph.node_count() as f32);
        features.push(igr.graph.edge_count() as f32);

        // Density
        let density = if igr.graph.node_count() > 1 {
            igr.graph.edge_count() as f32
                / (igr.graph.node_count() as f32 * (igr.graph.node_count() - 1) as f32 / 2.0)
        } else {
            0.0
        };
        features.push(density);

        // Degree statistics
        let degrees: Vec<usize> = igr
            .graph
            .node_indices()
            .map(|node| igr.graph.edges(node).count())
            .collect();

        if !degrees.is_empty() {
            let max_degree = *degrees.iter().max().unwrap() as f32;
            let min_degree = *degrees.iter().min().unwrap() as f32;
            let avg_degree = degrees.iter().sum::<usize>() as f32 / degrees.len() as f32;

            features.push(max_degree);
            features.push(min_degree);
            features.push(avg_degree);
        } else {
            features.extend_from_slice(&[0.0, 0.0, 0.0]);
        }

        // Pad to fixed size (32 features)
        while features.len() < 32 {
            features.push(0.0);
        }

        Ok(features)
    }

    fn extract_positions(&self, igr: &IntermediateGraph) -> Result<HashMap<String, (f64, f64)>> {
        let mut positions = HashMap::new();

        for node_idx in igr.graph.node_indices() {
            if let Some(node_data) = igr.graph.node_weight(node_idx) {
                let pos = (node_data.x, node_data.y);
                positions.insert(node_data.id.clone(), pos);
            }
        }

        Ok(positions)
    }

    fn calculate_quality_score(&self, igr: &IntermediateGraph) -> f64 {
        // Simplified quality calculation
        let node_count = igr.graph.node_count();
        let edge_count = igr.graph.edge_count();

        if node_count == 0 {
            return 0.0;
        }

        // Calculate basic quality metrics
        let density = edge_count as f64 / (node_count as f64 * (node_count - 1) as f64 / 2.0);
        let base_score = 0.5 + density * 0.3;

        // Add some randomness based on layout (simplified)
        let layout_factor = (node_count + edge_count) as f64 % 100.0 / 100.0 * 0.2;

        (base_score + layout_factor).clamp(0.0, 1.0)
    }

    fn augment_data(&self, samples: Vec<TrainingSample>) -> Result<Vec<TrainingSample>> {
        let mut augmented = samples.clone();

        for sample in &samples {
            for i in 1..self.config.augmentation_factor {
                let mut augmented_sample = sample.clone();

                // Apply position transformations
                let transform_type = i % 4;
                match transform_type {
                    1 => self.apply_rotation(&mut augmented_sample, 90.0)?,
                    2 => self.apply_scaling(&mut augmented_sample, 1.2)?,
                    3 => self.apply_translation(&mut augmented_sample, 50.0, 50.0)?,
                    _ => self.apply_noise(&mut augmented_sample, 0.1)?,
                }

                augmented.push(augmented_sample);
            }
        }

        Ok(augmented)
    }

    fn apply_rotation(&self, sample: &mut TrainingSample, angle_degrees: f64) -> Result<()> {
        let angle_rad = angle_degrees.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        for (_node_id, pos) in sample.target_positions.iter_mut() {
            let x = pos.0;
            let y = pos.1;
            pos.0 = x * cos_a - y * sin_a;
            pos.1 = x * sin_a + y * cos_a;
        }

        Ok(())
    }

    fn apply_scaling(&self, sample: &mut TrainingSample, factor: f64) -> Result<()> {
        for (_node_id, pos) in sample.target_positions.iter_mut() {
            pos.0 *= factor;
            pos.1 *= factor;
        }
        Ok(())
    }

    fn apply_translation(&self, sample: &mut TrainingSample, dx: f64, dy: f64) -> Result<()> {
        for (_node_id, pos) in sample.target_positions.iter_mut() {
            pos.0 += dx;
            pos.1 += dy;
        }
        Ok(())
    }

    fn apply_noise(&self, sample: &mut TrainingSample, noise_factor: f64) -> Result<()> {
        for (i, (_node_id, pos)) in sample.target_positions.iter_mut().enumerate() {
            // Simple pseudo-random noise
            let noise_x = ((i * 17) % 100) as f64 / 100.0 - 0.5;
            let noise_y = ((i * 23) % 100) as f64 / 100.0 - 0.5;

            pos.0 += noise_x * noise_factor * 100.0;
            pos.1 += noise_y * noise_factor * 100.0;
        }
        Ok(())
    }

    fn split_data(
        &self,
        samples: Vec<TrainingSample>,
    ) -> Result<(
        Vec<TrainingSample>,
        Vec<TrainingSample>,
        Vec<TrainingSample>,
    )> {
        let total = samples.len();
        let val_size = (total as f32 * self.config.validation_split) as usize;
        let test_size = (total as f32 * self.config.test_split) as usize;
        let train_size = total - val_size - test_size;

        let train_samples = samples[0..train_size].to_vec();
        let val_samples = samples[train_size..train_size + val_size].to_vec();
        let test_samples = samples[train_size + val_size..].to_vec();

        Ok((train_samples, val_samples, test_samples))
    }

    fn save_dataset(&self, samples: &[TrainingSample], path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(samples)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
