// src/layout/ml/tests.rs
//! Tests for ML layout functionality

// Include Phase 2 tests inline
include!("phase2_tests.rs");

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::igr::IntermediateGraph;
    use crate::layout::ml::{
        training::LayoutSession, GraphFeatureExtractor, MLLayoutStrategy, MLStrategySelector,
        TrainingDataCollector,
    };
    use crate::layout::{
        AdaptiveStrategy, DagreLayout, LayoutContext, LayoutEngineAdapter, LayoutStrategy,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;
    use tempfile::tempdir;

    fn create_simple_graph() -> IntermediateGraph {
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

        IntermediateGraph::from_ast(document).unwrap()
    }

    fn create_complex_graph() -> IntermediateGraph {
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: HashMap::new(),
            templates: HashMap::new(),
            diagram: None,
            nodes: vec![
                NodeDefinition {
                    id: "server".to_string(),
                    label: Some("Server".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "db".to_string(),
                    label: Some("Database".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "cache".to_string(),
                    label: Some("Cache".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "client1".to_string(),
                    label: Some("Client 1".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
                NodeDefinition {
                    id: "client2".to_string(),
                    label: Some("Client 2".to_string()),
                    component_type: None,
                    attributes: HashMap::new(),
                },
            ],
            edges: vec![
                EdgeDefinition {
                    from: "client1".to_string(),
                    to: "server".to_string(),
                    label: Some("HTTP".to_string()),
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "client2".to_string(),
                    to: "server".to_string(),
                    label: Some("HTTP".to_string()),
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "server".to_string(),
                    to: "db".to_string(),
                    label: Some("SQL".to_string()),
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "server".to_string(),
                    to: "cache".to_string(),
                    label: Some("Redis".to_string()),
                    arrow_type: ArrowType::DoubleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
            ],
            containers: vec![ContainerDefinition {
                id: Some("backend".to_string()),
                label: Some("Backend Services".to_string()),
                children: vec!["server".to_string(), "db".to_string(), "cache".to_string()],
                attributes: HashMap::new(),
                internal_statements: vec![],
            }],
            groups: vec![GroupDefinition {
                id: "clients".to_string(),
                label: Some("Client Applications".to_string()),
                group_type: GroupType::BasicGroup,
                children: vec!["client1".to_string(), "client2".to_string()],
                attributes: HashMap::new(),
                internal_statements: vec![],
            }],
            connections: vec![],
        };

        IntermediateGraph::from_ast(document).unwrap()
    }

    #[test]
    fn test_ml_layout_strategy_creation() {
        let adaptive_fallback = Arc::new(
            AdaptiveStrategy::new()
                .add_strategy(Arc::new(LayoutEngineAdapter::new(DagreLayout::new()))),
        );

        let ml_strategy = MLLayoutStrategy::new(adaptive_fallback);
        assert!(ml_strategy.is_ok());
    }

    #[test]
    fn test_ml_layout_simple_graph() {
        let mut igr = create_simple_graph();
        let context = LayoutContext::default();

        let adaptive_fallback = Arc::new(
            AdaptiveStrategy::new()
                .add_strategy(Arc::new(LayoutEngineAdapter::new(DagreLayout::new()))),
        );

        let ml_strategy = MLLayoutStrategy::new(adaptive_fallback).unwrap();
        let result = ml_strategy.apply(&mut igr, &context);

        assert!(result.is_ok());

        // Check that nodes have been positioned
        let (_, node_a) = igr.get_node_by_id("a").unwrap();
        let (_, node_b) = igr.get_node_by_id("b").unwrap();
        let (_, node_c) = igr.get_node_by_id("c").unwrap();

        // Nodes should have non-zero positions
        assert!(node_a.x != 0.0 || node_a.y != 0.0);
        assert!(node_b.x != 0.0 || node_b.y != 0.0);
        assert!(node_c.x != 0.0 || node_c.y != 0.0);

        // Nodes should not overlap
        assert!(node_a.x != node_b.x || node_a.y != node_b.y);
        assert!(node_b.x != node_c.x || node_b.y != node_c.y);
    }

    #[test]
    fn test_ml_strategy_selector_prediction() {
        let selector = MLStrategySelector::new().unwrap();
        let igr = create_complex_graph();
        let context = LayoutContext::default();

        let prediction = selector.select_strategy(&igr, &context);
        assert!(prediction.is_ok());

        let pred = prediction.unwrap();
        assert!(!pred.strategy_name.is_empty());
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
        assert!(pred.strategy.is_some());
        assert!(pred.performance_estimate.expected_time_ms >= 0.0);
        assert!(
            pred.quality_estimate.aesthetic_score >= 0.0
                && pred.quality_estimate.aesthetic_score <= 1.0
        );
    }

    #[test]
    fn test_feature_extraction() {
        let igr = create_complex_graph();
        let extractor = GraphFeatureExtractor::new();

        let features = extractor.extract(&igr);
        assert!(features.is_ok());

        let feat = features.unwrap();
        // 5 regular nodes + 1 virtual container node
        assert_eq!(feat.node_count, 6);
        assert_eq!(feat.edge_count, 4);
        assert!(feat.density > 0.0);
        // The graph might have 2 connected components due to groups
        assert!(feat.connected_components >= 1);
        assert!(feat.is_dag);
        assert!(!feat.has_cycles);
        // The container is a flat container, so hierarchy depth is 1
        // assert!(feat.hierarchy_depth >= 0);
        assert!(feat.container_ratio > 0.0);
    }

    #[test]
    fn test_training_data_collector() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("ml_training.jsonl");

        let collector = TrainingDataCollector::new()
            .with_output_path(output_path.to_str().unwrap().to_string())
            .with_buffer_size(2);

        // Create sessions and collect data
        for i in 0..3 {
            let igr = if i % 2 == 0 {
                create_simple_graph()
            } else {
                create_complex_graph()
            };

            let extractor = GraphFeatureExtractor::new();
            let features = extractor.extract(&igr).unwrap();

            let session = LayoutSession {
                id: format!("session-{i}"),
                igr,
                context: LayoutContext::default(),
                selected_strategy: if i % 2 == 0 { "dagre" } else { "force" }.to_string(),
                start_time: Instant::now(),
                features,
            };

            let performance = crate::layout::ml::PerformanceMetrics {
                expected_time_ms: 50.0 + i as f64 * 10.0,
                memory_usage_mb: 10.0 + i as f64 * 2.0,
                cpu_utilization: 0.3 + i as f64 * 0.1,
            };

            let quality = crate::layout::ml::QualityMetrics {
                edge_crossing_score: 0.9 - i as f64 * 0.1,
                node_overlap_score: 1.0,
                space_utilization: 0.8,
                symmetry_score: 0.7 + i as f64 * 0.05,
                aesthetic_score: 0.85,
            };

            collector.collect(session, performance, quality).unwrap();
        }

        // Force flush
        drop(collector);

        // Verify data was written
        let loaded = TrainingDataCollector::load_from_file(&output_path).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].selected_strategy, "dagre");
        assert_eq!(loaded[1].selected_strategy, "force");
    }

    #[test]
    fn test_ml_strategy_all_evaluations() {
        let selector = MLStrategySelector::new().unwrap();
        let igr = create_complex_graph();
        let context = LayoutContext::default();

        let predictions = selector.evaluate_all_strategies(&igr, &context);
        assert!(predictions.is_ok());

        let preds = predictions.unwrap();
        assert_eq!(preds.len(), 4); // dagre, force, elk, adaptive

        // Check that predictions are sorted by confidence
        for i in 1..preds.len() {
            assert!(preds[i - 1].confidence >= preds[i].confidence);
        }

        // Sum of confidences should be approximately 1.0 (softmax output)
        let total_confidence: f64 = preds.iter().map(|p| p.confidence).sum();
        assert!((total_confidence - 1.0).abs() < 0.01);
    }
}
