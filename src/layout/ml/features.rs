// src/layout/ml/features.rs
//! Feature extraction for graph structures to feed into ML models

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use ndarray::Array1;
use petgraph::algo::{connected_components, dijkstra};
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoNodeIdentifiers};
use std::collections::HashMap;

/// Features extracted from a graph for ML processing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphFeatures {
    // Structural features
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub clustering_coefficient: f64,

    // Topological features
    pub max_degree: usize,
    pub avg_degree: f64,
    pub diameter: usize,
    pub connected_components: usize,

    // Hierarchical features
    pub hierarchy_depth: usize,
    pub avg_children_per_container: f64,
    pub nesting_complexity: f64,

    // Node type distribution
    pub component_ratio: f64,
    pub container_ratio: f64,
    pub group_ratio: f64,

    // Edge characteristics
    pub avg_edge_length_estimate: f64,
    pub bidirectional_edge_ratio: f64,

    // Complexity indicators
    pub has_cycles: bool,
    pub is_dag: bool,
    pub is_tree: bool,
}

impl GraphFeatures {
    /// Convert features to a numerical vector for ML models
    pub fn to_vector(&self) -> Array1<f64> {
        Array1::from_vec(vec![
            self.node_count as f64,
            self.edge_count as f64,
            self.density,
            self.clustering_coefficient,
            self.max_degree as f64,
            self.avg_degree,
            self.diameter as f64,
            self.connected_components as f64,
            self.hierarchy_depth as f64,
            self.avg_children_per_container,
            self.nesting_complexity,
            self.component_ratio,
            self.container_ratio,
            self.group_ratio,
            self.avg_edge_length_estimate,
            self.bidirectional_edge_ratio,
            self.has_cycles as i32 as f64,
            self.is_dag as i32 as f64,
            self.is_tree as i32 as f64,
        ])
    }

    /// Get feature names for interpretability
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "node_count",
            "edge_count",
            "density",
            "clustering_coefficient",
            "max_degree",
            "avg_degree",
            "diameter",
            "connected_components",
            "hierarchy_depth",
            "avg_children_per_container",
            "nesting_complexity",
            "component_ratio",
            "container_ratio",
            "group_ratio",
            "avg_edge_length_estimate",
            "bidirectional_edge_ratio",
            "has_cycles",
            "is_dag",
            "is_tree",
        ]
    }
}

/// Extractor for graph features
pub struct GraphFeatureExtractor;

impl GraphFeatureExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract features from an intermediate graph
    pub fn extract(&self, igr: &IntermediateGraph) -> Result<GraphFeatures> {
        let node_count = igr.graph.node_count();
        let edge_count = igr.graph.edge_count();

        if node_count == 0 {
            return Err(EDSLError::Layout(
                crate::error::LayoutError::CalculationFailed(
                    "Cannot extract features from empty graph".to_string(),
                ),
            ));
        }

        // Calculate density
        let max_edges = node_count * (node_count - 1);
        let density = if max_edges > 0 {
            edge_count as f64 / max_edges as f64
        } else {
            0.0
        };

        // Calculate degree statistics
        let degrees = self.calculate_degrees(igr);
        let max_degree = *degrees.values().max().unwrap_or(&0);
        let avg_degree = if node_count > 0 {
            degrees.values().sum::<usize>() as f64 / node_count as f64
        } else {
            0.0
        };

        // Calculate clustering coefficient
        let clustering_coefficient = self.calculate_clustering_coefficient(igr);

        // Calculate connected components
        let connected_components = connected_components(&igr.graph);

        // Calculate diameter (simplified - maximum shortest path)
        let diameter = self.calculate_diameter(igr);

        // Calculate hierarchical features
        let (hierarchy_depth, avg_children_per_container, nesting_complexity) =
            self.calculate_hierarchical_features(igr);

        // Calculate node type distribution
        let (component_ratio, container_ratio, group_ratio) = self.calculate_node_type_ratios(igr);

        // Calculate edge characteristics
        let (avg_edge_length_estimate, bidirectional_edge_ratio) =
            self.calculate_edge_characteristics(igr);

        // Detect graph properties
        let has_cycles = petgraph::algo::is_cyclic_directed(&igr.graph);
        let is_dag = !has_cycles;
        let is_tree = is_dag && edge_count == node_count.saturating_sub(1);

        Ok(GraphFeatures {
            node_count,
            edge_count,
            density,
            clustering_coefficient,
            max_degree,
            avg_degree,
            diameter,
            connected_components,
            hierarchy_depth,
            avg_children_per_container,
            nesting_complexity,
            component_ratio,
            container_ratio,
            group_ratio,
            avg_edge_length_estimate,
            bidirectional_edge_ratio,
            has_cycles,
            is_dag,
            is_tree,
        })
    }

    fn calculate_degrees(&self, igr: &IntermediateGraph) -> HashMap<NodeIndex, usize> {
        let mut degrees = HashMap::new();

        for node in igr.graph.node_identifiers() {
            let in_degree = igr
                .graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .count();
            let out_degree = igr
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .count();
            degrees.insert(node, in_degree + out_degree);
        }

        degrees
    }

    fn calculate_clustering_coefficient(&self, igr: &IntermediateGraph) -> f64 {
        // Simplified clustering coefficient calculation
        // For each node, calculate the ratio of existing edges between neighbors
        // to the maximum possible edges between neighbors

        let mut total_coefficient = 0.0;
        let mut node_count = 0;

        for node in igr.graph.node_identifiers() {
            let neighbors: Vec<_> = igr.graph.neighbors_undirected(node).collect();

            if neighbors.len() < 2 {
                continue;
            }

            let mut edge_count = 0;
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    if igr.graph.find_edge(neighbors[i], neighbors[j]).is_some()
                        || igr.graph.find_edge(neighbors[j], neighbors[i]).is_some()
                    {
                        edge_count += 1;
                    }
                }
            }

            let max_edges = neighbors.len() * (neighbors.len() - 1) / 2;
            if max_edges > 0 {
                total_coefficient += edge_count as f64 / max_edges as f64;
                node_count += 1;
            }
        }

        if node_count > 0 {
            total_coefficient / node_count as f64
        } else {
            0.0
        }
    }

    fn calculate_diameter(&self, igr: &IntermediateGraph) -> usize {
        // Calculate the maximum shortest path between any two nodes
        let mut max_distance = 0;

        // Sample nodes for large graphs to avoid O(n²) complexity
        let sample_size = std::cmp::min(10, igr.graph.node_count());
        let nodes: Vec<_> = igr.graph.node_identifiers().take(sample_size).collect();

        for &start in &nodes {
            let distances = dijkstra(&igr.graph, start, None, |_| 1);
            if let Some(&max_dist) = distances.values().max() {
                max_distance = std::cmp::max(max_distance, max_dist);
            }
        }

        max_distance
    }

    fn calculate_hierarchical_features(&self, igr: &IntermediateGraph) -> (usize, f64, f64) {
        let mut max_depth = 0;
        let mut total_children = 0;
        let mut container_count = 0;
        let mut depth_sum = 0;

        // Calculate depth for each container
        for container in &igr.containers {
            container_count += 1;
            total_children += container.children.len();

            // Calculate depth by checking nested containers
            let depth = if let Some(id) = &container.id {
                self.calculate_container_depth(igr, id, 0)
            } else {
                0
            };
            max_depth = std::cmp::max(max_depth, depth);
            depth_sum += depth;
        }

        let avg_children = if container_count > 0 {
            total_children as f64 / container_count as f64
        } else {
            0.0
        };

        let nesting_complexity = if container_count > 0 {
            depth_sum as f64 / container_count as f64
        } else {
            0.0
        };

        (max_depth, avg_children, nesting_complexity)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn calculate_container_depth(
        &self,
        igr: &IntermediateGraph,
        container_id: &str,
        current_depth: usize,
    ) -> usize {
        let mut max_child_depth = current_depth;

        if let Some(container) = igr
            .containers
            .iter()
            .find(|c| c.id.as_deref() == Some(container_id))
        {
            // Check nested containers
            for &nested_idx in &container.nested_containers {
                if let Some(nested_container) = igr.containers.get(nested_idx) {
                    if let Some(nested_id) = &nested_container.id {
                        let child_depth =
                            self.calculate_container_depth(igr, nested_id, current_depth + 1);
                        max_child_depth = std::cmp::max(max_child_depth, child_depth);
                    }
                }
            }
        }

        max_child_depth
    }

    fn calculate_node_type_ratios(&self, igr: &IntermediateGraph) -> (f64, f64, f64) {
        let total_nodes = igr.graph.node_count() as f64;
        if total_nodes == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let mut component_count = 0;
        let mut container_count = 0;

        // Count based on node properties
        for node_weight in igr.graph.node_weights() {
            if node_weight.is_virtual_container {
                container_count += 1;
            } else {
                component_count += 1;
            }
        }

        // Groups are stored separately
        let group_count = igr.groups.len();

        (
            component_count as f64 / total_nodes,
            container_count as f64 / total_nodes,
            group_count as f64 / total_nodes,
        )
    }

    fn calculate_edge_characteristics(&self, igr: &IntermediateGraph) -> (f64, f64) {
        let edge_count = igr.graph.edge_count();
        if edge_count == 0 {
            return (0.0, 0.0);
        }

        let mut bidirectional_pairs = 0;
        let mut checked_edges = HashMap::new();

        // Check for bidirectional edges
        for edge in igr.graph.edge_references() {
            let source = edge.source();
            let target = edge.target();
            let reverse_key = (target, source);

            if checked_edges.contains_key(&reverse_key) {
                bidirectional_pairs += 1;
            } else {
                checked_edges.insert((source, target), true);
            }
        }

        let bidirectional_ratio = (bidirectional_pairs * 2) as f64 / edge_count as f64;

        // Estimate average edge length (simplified - based on graph structure)
        let avg_edge_length_estimate = if igr.graph.node_count() > 0 {
            100.0 * (1.0 + (edge_count as f64 / igr.graph.node_count() as f64).ln())
        } else {
            100.0
        };

        (avg_edge_length_estimate, bidirectional_ratio)
    }
}

impl Default for GraphFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap;

    #[test]
    fn test_feature_extraction_simple_graph() {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
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

        let igr = IntermediateGraph::from_ast(document).unwrap();
        let extractor = GraphFeatureExtractor::new();
        let features = extractor.extract(&igr).unwrap();

        assert_eq!(features.node_count, 2);
        assert_eq!(features.edge_count, 1);
        assert!(features.is_dag);
        assert!(!features.has_cycles);
        assert_eq!(features.connected_components, 1);
    }
}
