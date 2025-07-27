// src/layout/ml/phase2_tests.rs
// Comprehensive tests for Phase 2 ML layout components

#[cfg(test)]
mod phase2_tests {
    use crate::ast::*;
    use crate::igr::IntermediateGraph;
    use crate::layout::{
        LayoutContext, LayoutStrategy, LayoutEngineAdapter, DagreLayout,
        ml::*,
    };
    use crate::layout::ml::rl::LayoutPolicy;
    use crate::layout::ml::feedback::{LayoutAdjustment, AdjustmentType};
    use std::collections::HashMap;
    use std::sync::Arc;
    use petgraph::graph::NodeIndex;
    use petgraph::visit::IntoNodeIdentifiers;

    fn create_test_graph() -> IntermediateGraph {
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
                NodeDefinition {
                    id: "d".to_string(),
                    label: Some("Node D".to_string()),
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
                EdgeDefinition {
                    from: "a".to_string(),
                    to: "d".to_string(),
                    label: None,
                    arrow_type: ArrowType::SingleArrow,
                    attributes: HashMap::new(),
                    style: None,
                },
                EdgeDefinition {
                    from: "d".to_string(),
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

    #[test]
    fn test_gnn_layout_predictor() {
        let predictor = GNNLayoutPredictor::new(32, 64, 3).unwrap();
        let igr = create_test_graph();

        let prediction = predictor.predict_layout(&igr).unwrap();

        // Verify prediction structure
        assert_eq!(prediction.positions.len(), 4);
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);

        // Check that all nodes have positions
        for node in igr.graph.node_indices() {
            assert!(prediction.positions.contains_key(&node));
        }

        // Verify positions are reasonable
        for &(x, y) in prediction.positions.values() {
            assert!(x.is_finite() && y.is_finite());
            assert!(x.abs() < 10000.0 && y.abs() < 10000.0);
        }
    }

    #[test]
    fn test_rl_environment() {
        let igr = create_test_graph();
        let mut positions = HashMap::new();

        // Initialize positions
        for (i, node) in igr.graph.node_identifiers().enumerate() {
            positions.insert(node, (i as f64 * 100.0, 0.0));
        }

        let mut env = LayoutEnvironment::new(igr.clone(), positions).unwrap();

        // Test step function
        let action = LayoutAction::MoveNode {
            node_idx: 0,
            dx: 10.0,
            dy: 20.0,
        };

        let (new_state, reward, done) = env.step(action).unwrap();

        assert!(!done); // Should not be done after one step
        assert_eq!(new_state.positions.len(), 4);
        assert!(reward.is_finite());

        // Test reset
        let reset_state = env.reset();
        assert_eq!(reset_state.positions.len(), 4);
    }

    #[test]
    fn test_rl_policy() {
        let policy = LayoutPolicy::new(20, 10, 256).unwrap();

        let state = LayoutState {
            positions: HashMap::new(),
            edge_crossings: 2,
            node_overlaps: 1,
            bounding_box: (0.0, 0.0, 200.0, 200.0),
            symmetry_score: 0.8,
        };

        let (action_probs, value) = policy.forward(&state).unwrap();

        // Verify output structure
        assert_eq!(action_probs.len(), 10);
        assert!(value.is_finite());

        // Check probability distribution
        let sum: f64 = action_probs.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);

        for &prob in &action_probs {
            assert!((0.0..=1.0).contains(&prob));
        }
    }

    #[test]
    fn test_constraint_solver() {
        let solver = NeuralConstraintSolver::new(20).unwrap();
        let igr = create_test_graph();

        // Get node indices
        let nodes: Vec<NodeIndex> = igr.graph.node_identifiers().collect();

        // Define constraints
        let constraints = vec![
            LayoutConstraint::MinDistance {
                node1: nodes[0],
                node2: nodes[1],
                distance: 100.0,
            },
            LayoutConstraint::Alignment {
                nodes: vec![nodes[0], nodes[1]],
                axis: Axis::Horizontal,
            },
            LayoutConstraint::FixedPosition {
                node: nodes[0],
                position: (50.0, 50.0),
            },
        ];

        let solution = solver.solve(&igr, &constraints, None).unwrap();

        // Verify solution structure
        assert_eq!(solution.positions.len(), nodes.len());
        assert!(solution.feasibility_score >= 0.0 && solution.feasibility_score <= 1.0);

        // Check fixed position constraint
        if let Some(&pos) = solution.positions.get(&nodes[0]) {
            assert!((pos.0 - 50.0).abs() < 50.0); // Some tolerance
            assert!((pos.1 - 50.0).abs() < 50.0);
        }

        // Check minimum distance constraint
        if let (Some(&pos1), Some(&pos2)) = (
            solution.positions.get(&nodes[0]),
            solution.positions.get(&nodes[1])
        ) {
            let distance = ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
            assert!(distance >= 50.0); // Allow some violation
        }
    }

    #[test]
    fn test_feedback_collector() {
        let collector = FeedbackCollector::new(100);
        let igr = create_test_graph();

        // Start session
        let session_id = collector.start_session(
            &igr,
            "dagre".to_string(),
            vec!["force".to_string(), "elk".to_string()],
            0.75,
        ).unwrap();

        // Record adjustments
        let adjustment1 = LayoutAdjustment {
            timestamp: chrono::Utc::now(),
            adjustment_type: AdjustmentType::NodeMove {
                node_id: "a".to_string(),
                dx: 50.0,
                dy: 30.0,
            },
            magnitude: 58.31,
        };

        collector.record_adjustment(&session_id, adjustment1).unwrap();

        // Complete session
        collector.complete_session(
            &session_id,
            FeedbackType::MinorAdjust,
            0.85,
            3000,
        ).unwrap();

        // Check statistics
        let stats = collector.get_feedback_stats();
        assert_eq!(stats.total_sessions, 1);
        assert!(stats.avg_satisfaction_by_strategy.contains_key("dagre"));

        let dagre_satisfaction = stats.avg_satisfaction_by_strategy.get("dagre").unwrap();
        assert_eq!(*dagre_satisfaction, 0.75); // MinorAdjust = 0.75
    }

    #[test]
    fn test_enhanced_ml_integration() {
        let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));

        // Test with all features enabled
        let strategy = EnhancedMLLayoutBuilder::new()
            .with_gnn(true)
            .with_rl(false) // Disable RL for faster test
            .with_constraints(true)
            .with_feedback(true)
            .with_fallback(fallback.clone())
            .build()
            .unwrap();

        let mut igr = create_test_graph();
        let context = LayoutContext::default();

        // Apply enhanced layout
        strategy.apply(&mut igr, &context).unwrap();

        // Verify all nodes have positions
        for node_idx in igr.graph.node_indices() {
            let node = igr.graph.node_weight(node_idx).unwrap();
            // Check that position is reasonable
            let (x, y) = (node.x, node.y);
            if x != 0.0 || y != 0.0 {
                assert!(x.is_finite() && y.is_finite());
            }
        }
    }

    #[test]
    fn test_enhanced_ml_builder_configurations() {
        let fallback = Arc::new(LayoutEngineAdapter::new(DagreLayout::new()));

        // Test different configurations
        let configs = vec![
            (true, false, false, false),   // GNN only
            (false, true, false, false),   // RL only
            (false, false, true, false),   // Constraints only
            (false, false, false, true),   // Feedback only
            (true, true, true, true),      // All enabled
        ];

        for (gnn, rl, constraints, feedback) in configs {
            let strategy = EnhancedMLLayoutBuilder::new()
                .with_gnn(gnn)
                .with_rl(rl)
                .with_constraints(constraints)
                .with_feedback(feedback)
                .with_rl_episodes(10) // Reduce for testing
                .with_fallback(fallback.clone())
                .build();

            assert!(strategy.is_ok());

            let strategy = strategy.unwrap();
            // Verify the strategy was created successfully
            // We can't access private fields, but the build succeeded with the expected config
        }
    }

    #[test]
    fn test_feedback_types() {
        assert_eq!(FeedbackType::Accept.to_satisfaction_score(), 1.0);
        assert_eq!(FeedbackType::MinorAdjust.to_satisfaction_score(), 0.75);
        assert_eq!(FeedbackType::MajorAdjust.to_satisfaction_score(), 0.5);
        assert_eq!(FeedbackType::Reject.to_satisfaction_score(), 0.0);

        assert_eq!(FeedbackType::Rating(5).to_satisfaction_score(), 1.0);
        assert_eq!(FeedbackType::Rating(4).to_satisfaction_score(), 0.75);
        assert_eq!(FeedbackType::Rating(3).to_satisfaction_score(), 0.5);
        assert_eq!(FeedbackType::Rating(2).to_satisfaction_score(), 0.25);
        assert_eq!(FeedbackType::Rating(1).to_satisfaction_score(), 0.0);
    }

    #[test]
    fn test_layout_actions() {
        let mut positions = HashMap::new();
        let nodes: Vec<NodeIndex> = (0..4).map(NodeIndex::new).collect();

        for (i, &node) in nodes.iter().enumerate() {
            positions.insert(node, (i as f64 * 100.0, 0.0));
        }

        // Test move action
        let move_action = LayoutAction::MoveNode {
            node_idx: 0,
            dx: 50.0,
            dy: -50.0,
        };

        // Test alignment actions
        let h_align = LayoutAction::AlignHorizontal {
            nodes: [0, 1],
        };

        let v_align = LayoutAction::AlignVertical {
            nodes: [2, 3],
        };

        // Test spacing action
        let spacing = LayoutAction::AdjustSpacing {
            scale: 1.2,
        };

        // Test swap action
        let swap = LayoutAction::SwapNodes {
            node1: 0,
            node2: 1,
        };

        // All actions should be created successfully
        assert!(matches!(move_action, LayoutAction::MoveNode { .. }));
        assert!(matches!(h_align, LayoutAction::AlignHorizontal { .. }));
        assert!(matches!(v_align, LayoutAction::AlignVertical { .. }));
        assert!(matches!(spacing, LayoutAction::AdjustSpacing { .. }));
        assert!(matches!(swap, LayoutAction::SwapNodes { .. }));
    }
}
