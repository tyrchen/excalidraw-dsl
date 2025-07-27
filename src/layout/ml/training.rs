// src/layout/ml/training.rs
//! Training data collection and management for ML models

use crate::error::Result;
use crate::igr::IntermediateGraph;
use crate::layout::LayoutContext;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{GraphFeatures, PerformanceMetrics, QualityMetrics};

/// Training data sample for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    /// Unique identifier for this sample
    pub id: String,
    /// Timestamp when the data was collected
    pub timestamp: u64,
    /// Graph features extracted
    pub features: TrainingGraphFeatures,
    /// Strategy that was used
    pub selected_strategy: String,
    /// Actual performance metrics
    pub performance_metrics: TrainingPerformanceMetrics,
    /// Actual quality metrics (may be from user feedback)
    pub quality_metrics: TrainingQualityMetrics,
    /// User satisfaction score (0-1)
    pub user_satisfaction: Option<f64>,
}

/// Serializable version of GraphFeatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingGraphFeatures {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub clustering_coefficient: f64,
    pub max_degree: usize,
    pub avg_degree: f64,
    pub diameter: usize,
    pub connected_components: usize,
    pub hierarchy_depth: usize,
    pub avg_children_per_container: f64,
    pub nesting_complexity: f64,
    pub component_ratio: f64,
    pub container_ratio: f64,
    pub group_ratio: f64,
    pub avg_edge_length_estimate: f64,
    pub bidirectional_edge_ratio: f64,
    pub has_cycles: bool,
    pub is_dag: bool,
    pub is_tree: bool,
}

impl From<&GraphFeatures> for TrainingGraphFeatures {
    fn from(features: &GraphFeatures) -> Self {
        Self {
            node_count: features.node_count,
            edge_count: features.edge_count,
            density: features.density,
            clustering_coefficient: features.clustering_coefficient,
            max_degree: features.max_degree,
            avg_degree: features.avg_degree,
            diameter: features.diameter,
            connected_components: features.connected_components,
            hierarchy_depth: features.hierarchy_depth,
            avg_children_per_container: features.avg_children_per_container,
            nesting_complexity: features.nesting_complexity,
            component_ratio: features.component_ratio,
            container_ratio: features.container_ratio,
            group_ratio: features.group_ratio,
            avg_edge_length_estimate: features.avg_edge_length_estimate,
            bidirectional_edge_ratio: features.bidirectional_edge_ratio,
            has_cycles: features.has_cycles,
            is_dag: features.is_dag,
            is_tree: features.is_tree,
        }
    }
}

/// Serializable version of PerformanceMetrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingPerformanceMetrics {
    pub actual_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
}

impl From<&PerformanceMetrics> for TrainingPerformanceMetrics {
    fn from(metrics: &PerformanceMetrics) -> Self {
        Self {
            actual_time_ms: metrics.expected_time_ms,
            memory_usage_mb: metrics.memory_usage_mb,
            cpu_utilization: metrics.cpu_utilization,
        }
    }
}

/// Serializable version of QualityMetrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingQualityMetrics {
    pub edge_crossing_count: Option<u32>,
    pub node_overlap_count: Option<u32>,
    pub space_utilization: f64,
    pub symmetry_score: f64,
    pub aesthetic_score: f64,
}

impl From<&QualityMetrics> for TrainingQualityMetrics {
    fn from(metrics: &QualityMetrics) -> Self {
        Self {
            edge_crossing_count: None, // To be filled by actual measurement
            node_overlap_count: None,  // To be filled by actual measurement
            space_utilization: metrics.space_utilization,
            symmetry_score: metrics.symmetry_score,
            aesthetic_score: metrics.aesthetic_score,
        }
    }
}

/// Session data for layout operations
pub struct LayoutSession {
    pub id: String,
    pub igr: IntermediateGraph,
    pub context: LayoutContext,
    pub selected_strategy: String,
    pub start_time: Instant,
    pub features: GraphFeatures,
}

/// Training data collector that gathers data during layout operations
pub struct TrainingDataCollector {
    buffer: Arc<Mutex<Vec<TrainingData>>>,
    buffer_size: usize,
    output_path: Option<String>,
}

impl TrainingDataCollector {
    /// Create a new training data collector
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_size: 100,
            output_path: None,
        }
    }

    /// Set the output path for training data
    pub fn with_output_path(mut self, path: String) -> Self {
        self.output_path = Some(path);
        self
    }

    /// Set the buffer size before flushing to disk
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Collect training data from a layout session
    pub fn collect(
        &self,
        session: LayoutSession,
        performance: PerformanceMetrics,
        quality: QualityMetrics,
    ) -> Result<()> {
        let training_data = TrainingData {
            id: session.id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            features: TrainingGraphFeatures::from(&session.features),
            selected_strategy: session.selected_strategy,
            performance_metrics: TrainingPerformanceMetrics {
                actual_time_ms: session.start_time.elapsed().as_millis() as f64,
                memory_usage_mb: performance.memory_usage_mb,
                cpu_utilization: performance.cpu_utilization,
            },
            quality_metrics: TrainingQualityMetrics::from(&quality),
            user_satisfaction: None,
        };

        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(training_data);

        // Check if we should flush
        if buffer.len() >= self.buffer_size {
            drop(buffer); // Release lock before flushing
            self.flush()?;
        }

        Ok(())
    }

    /// Update training data with user feedback
    pub fn update_with_feedback(
        &self,
        session_id: &str,
        user_satisfaction: f64,
        edge_crossing_count: Option<u32>,
        node_overlap_count: Option<u32>,
    ) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();

        if let Some(data) = buffer.iter_mut().find(|d| d.id == session_id) {
            data.user_satisfaction = Some(user_satisfaction);
            if let Some(count) = edge_crossing_count {
                data.quality_metrics.edge_crossing_count = Some(count);
            }
            if let Some(count) = node_overlap_count {
                data.quality_metrics.node_overlap_count = Some(count);
            }
        }

        Ok(())
    }

    /// Flush buffered training data to disk
    pub fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();

        if buffer.is_empty() {
            return Ok(());
        }

        if let Some(ref path) = self.output_path {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| {
                    crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                        format!("Failed to open training data file: {e}"),
                    ))
                })?;

            let mut writer = BufWriter::new(file);

            for data in buffer.iter() {
                serde_json::to_writer(&mut writer, data).map_err(|e| {
                    crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                        format!("Failed to write training data: {e}"),
                    ))
                })?;
                writeln!(&mut writer).map_err(|e| {
                    crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                        format!("Failed to write newline: {e}"),
                    ))
                })?;
            }

            writer.flush().map_err(|e| {
                crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                    format!("Failed to flush training data: {e}"),
                ))
            })?;
        }

        buffer.clear();
        Ok(())
    }

    /// Load training data from a file
    pub fn load_from_file(path: &Path) -> Result<Vec<TrainingData>> {
        let file = File::open(path).map_err(|e| {
            crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to open training data file: {e}"
            )))
        })?;

        let reader = BufReader::new(file);
        let mut data = Vec::new();

        for line in std::io::BufRead::lines(reader) {
            let line = line.map_err(|e| {
                crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                    format!("Failed to read line: {e}"),
                ))
            })?;

            if !line.trim().is_empty() {
                let sample: TrainingData = serde_json::from_str(&line).map_err(|e| {
                    crate::error::EDSLError::Layout(crate::error::LayoutError::CalculationFailed(
                        format!("Failed to parse training data: {e}"),
                    ))
                })?;
                data.push(sample);
            }
        }

        Ok(data)
    }
}

impl Default for TrainingDataCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TrainingDataCollector {
    fn drop(&mut self) {
        // Flush any remaining data
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::layout::ml::GraphFeatureExtractor;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_training_data_collection() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("training.jsonl");

        let collector = TrainingDataCollector::new()
            .with_output_path(output_path.to_str().unwrap().to_string())
            .with_buffer_size(1);

        // Create a simple graph
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            nodes: vec![NodeDefinition {
                id: "a".to_string(),
                label: Some("A".to_string()),
                component_type: None,
                attributes: HashMap::new(),
            }],
            edges: vec![],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let extractor = GraphFeatureExtractor::new();
        let features = extractor.extract(&igr).unwrap();

        let session = LayoutSession {
            id: "test-session".to_string(),
            igr,
            context: LayoutContext::default(),
            selected_strategy: "dagre".to_string(),
            start_time: Instant::now(),
            features,
        };

        let performance = PerformanceMetrics {
            expected_time_ms: 50.0,
            memory_usage_mb: 10.0,
            cpu_utilization: 0.3,
        };

        let quality = QualityMetrics {
            edge_crossing_score: 0.9,
            node_overlap_score: 1.0,
            space_utilization: 0.8,
            symmetry_score: 0.7,
            aesthetic_score: 0.85,
        };

        collector.collect(session, performance, quality).unwrap();

        // Load and verify
        let loaded = TrainingDataCollector::load_from_file(&output_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].selected_strategy, "dagre");
        assert_eq!(loaded[0].features.node_count, 1);
    }
}
