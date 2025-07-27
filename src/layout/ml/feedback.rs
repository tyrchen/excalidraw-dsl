// src/layout/ml/feedback.rs
//! Online learning from user feedback

use crate::error::{EDSLError, Result};
use crate::igr::IntermediateGraph;
use crate::layout::ml::{GraphFeatureExtractor, GraphFeatures};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// User feedback types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// User accepted the layout as-is
    Accept,
    /// User made minor adjustments
    MinorAdjust,
    /// User made major adjustments
    MajorAdjust,
    /// User rejected and requested different strategy
    Reject,
    /// Explicit rating (1-5)
    Rating(u8),
}

impl FeedbackType {
    /// Convert feedback to satisfaction score (0.0 - 1.0)
    pub fn to_satisfaction_score(&self) -> f64 {
        match self {
            FeedbackType::Accept => 1.0,
            FeedbackType::MinorAdjust => 0.75,
            FeedbackType::MajorAdjust => 0.5,
            FeedbackType::Reject => 0.0,
            FeedbackType::Rating(r) => (*r as f64 - 1.0) / 4.0,
        }
    }
}

/// Feedback session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSession {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub graph_features: GraphFeatures,
    pub selected_strategy: String,
    pub alternative_strategies: Vec<String>,
    pub initial_quality_score: f64,
    pub final_quality_score: Option<f64>,
    pub user_feedback: Option<FeedbackType>,
    pub time_spent_ms: u64,
    pub adjustments_made: Vec<LayoutAdjustment>,
}

/// Layout adjustment made by user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutAdjustment {
    pub timestamp: DateTime<Utc>,
    pub adjustment_type: AdjustmentType,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    NodeMove { node_id: String, dx: f64, dy: f64 },
    NodeAlign { nodes: Vec<String>, axis: String },
    SpacingChange { scale: f64 },
    StrategySwitch { from: String, to: String },
}

/// Collects and processes user feedback for online learning
pub struct FeedbackCollector {
    sessions: Arc<Mutex<VecDeque<FeedbackSession>>>,
    max_sessions: usize,
    feature_extractor: GraphFeatureExtractor,
    anonymizer: DataAnonymizer,
}

impl FeedbackCollector {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(VecDeque::new())),
            max_sessions,
            feature_extractor: GraphFeatureExtractor::new(),
            anonymizer: DataAnonymizer::new(),
        }
    }

    /// Start a new feedback session
    pub fn start_session(
        &self,
        graph: &IntermediateGraph,
        selected_strategy: String,
        alternative_strategies: Vec<String>,
        initial_quality_score: f64,
    ) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let graph_features = self.feature_extractor.extract(graph)?;

        let session = FeedbackSession {
            session_id: session_id.clone(),
            timestamp: Utc::now(),
            graph_features,
            selected_strategy,
            alternative_strategies,
            initial_quality_score,
            final_quality_score: None,
            user_feedback: None,
            time_spent_ms: 0,
            adjustments_made: vec![],
        };

        let mut sessions = self.sessions.lock().unwrap();
        if sessions.len() >= self.max_sessions {
            sessions.pop_front();
        }
        sessions.push_back(session);

        Ok(session_id)
    }

    /// Record a user adjustment
    pub fn record_adjustment(&self, session_id: &str, adjustment: LayoutAdjustment) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.adjustments_made.push(adjustment);
        }
        Ok(())
    }

    /// Complete a feedback session
    pub fn complete_session(
        &self,
        session_id: &str,
        feedback: FeedbackType,
        final_quality_score: f64,
        time_spent_ms: u64,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.user_feedback = Some(feedback);
            session.final_quality_score = Some(final_quality_score);
            session.time_spent_ms = time_spent_ms;
        }
        Ok(())
    }

    /// Get recent feedback statistics
    pub fn get_feedback_stats(&self) -> FeedbackStatistics {
        let sessions = self.sessions.lock().unwrap();

        let total_sessions = sessions.len();
        let mut strategy_satisfaction: HashMap<String, Vec<f64>> = HashMap::new();
        let mut feedback_distribution: HashMap<String, usize> = HashMap::new();

        for session in sessions.iter() {
            if let Some(feedback) = &session.user_feedback {
                let satisfaction = feedback.to_satisfaction_score();
                strategy_satisfaction
                    .entry(session.selected_strategy.clone())
                    .or_default()
                    .push(satisfaction);

                let feedback_key = format!("{feedback:?}");
                *feedback_distribution.entry(feedback_key).or_insert(0) += 1;
            }
        }

        // Calculate average satisfaction per strategy
        let avg_satisfaction: HashMap<String, f64> = strategy_satisfaction
            .iter()
            .map(|(strategy, scores)| {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                (strategy.clone(), avg)
            })
            .collect();

        FeedbackStatistics {
            total_sessions,
            avg_satisfaction_by_strategy: avg_satisfaction,
            feedback_distribution,
            recent_trend: self.calculate_trend(&sessions),
        }
    }

    /// Export anonymized feedback data for training
    pub fn export_training_data(&self, path: &Path) -> Result<()> {
        let sessions = self.sessions.lock().unwrap();
        let anonymized_sessions: Vec<_> = sessions
            .iter()
            .map(|s| self.anonymizer.anonymize_session(s))
            .collect();

        let json = serde_json::to_string_pretty(&anonymized_sessions).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to serialize feedback data: {e}"
            )))
        })?;

        std::fs::write(path, json).map_err(|e| {
            EDSLError::Layout(crate::error::LayoutError::CalculationFailed(format!(
                "Failed to write feedback data: {e}"
            )))
        })?;

        Ok(())
    }

    fn calculate_trend(&self, sessions: &VecDeque<FeedbackSession>) -> f64 {
        if sessions.len() < 2 {
            return 0.0;
        }

        // Calculate trend over last 10 sessions
        let recent_sessions: Vec<_> = sessions.iter().rev().take(10).collect();
        if recent_sessions.len() < 2 {
            return 0.0;
        }

        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        let mut xy_sum = 0.0;
        let mut x2_sum = 0.0;
        let n = recent_sessions.len() as f64;

        for (i, session) in recent_sessions.iter().enumerate() {
            if let Some(feedback) = &session.user_feedback {
                let x = i as f64;
                let y = feedback.to_satisfaction_score();
                x_sum += x;
                y_sum += y;
                xy_sum += x * y;
                x2_sum += x * x;
            }
        }

        // Calculate slope of trend line
        
        (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum * x_sum)
    }
}

/// Statistics about collected feedback
#[derive(Debug, Clone)]
pub struct FeedbackStatistics {
    pub total_sessions: usize,
    pub avg_satisfaction_by_strategy: HashMap<String, f64>,
    pub feedback_distribution: HashMap<String, usize>,
    pub recent_trend: f64,
}

/// Anonymizes session data for privacy
struct DataAnonymizer;

impl DataAnonymizer {
    fn new() -> Self {
        Self
    }

    fn anonymize_session(&self, session: &FeedbackSession) -> AnonymizedSession {
        AnonymizedSession {
            timestamp: session.timestamp,
            graph_features: session.graph_features.clone(),
            selected_strategy: session.selected_strategy.clone(),
            user_satisfaction: session.user_feedback.map(|f| f.to_satisfaction_score()),
            quality_improvement: match (session.initial_quality_score, session.final_quality_score)
            {
                (initial, Some(final_score)) => Some(final_score - initial),
                _ => None,
            },
            adjustment_count: session.adjustments_made.len(),
            time_spent_ms: session.time_spent_ms,
        }
    }
}

/// Anonymized session data for training
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnonymizedSession {
    timestamp: DateTime<Utc>,
    graph_features: GraphFeatures,
    selected_strategy: String,
    user_satisfaction: Option<f64>,
    quality_improvement: Option<f64>,
    adjustment_count: usize,
    time_spent_ms: u64,
}

/// Online model updater that learns from feedback
pub struct OnlineModelUpdater {
    learning_rate: f64,
    batch_size: usize,
    update_threshold: usize,
    pending_updates: Arc<Mutex<Vec<AnonymizedSession>>>,
}

impl OnlineModelUpdater {
    pub fn new(learning_rate: f64, batch_size: usize) -> Self {
        Self {
            learning_rate,
            batch_size,
            update_threshold: batch_size * 2,
            pending_updates: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue feedback for model update
    pub fn queue_feedback(&self, feedback: &FeedbackCollector) -> Result<()> {
        let stats = feedback.get_feedback_stats();

        // Only queue if we have enough recent feedback
        if stats.total_sessions < 5 {
            return Ok(());
        }

        // Export and queue anonymized data
        let sessions = feedback.sessions.lock().unwrap();
        let anonymized: Vec<_> = sessions
            .iter()
            .filter(|s| s.user_feedback.is_some())
            .map(|s| feedback.anonymizer.anonymize_session(s))
            .collect();

        let mut pending = self.pending_updates.lock().unwrap();
        pending.extend(anonymized);

        // Trigger update if threshold reached
        if pending.len() >= self.update_threshold {
            self.trigger_model_update(&pending)?;
            pending.clear();
        }

        Ok(())
    }

    /// Perform online model update
    fn trigger_model_update(&self, sessions: &[AnonymizedSession]) -> Result<()> {
        log::info!(
            "Triggering online model update with {} sessions",
            sessions.len()
        );

        // Group sessions by strategy
        let mut strategy_groups: HashMap<String, Vec<&AnonymizedSession>> = HashMap::new();
        for session in sessions {
            strategy_groups
                .entry(session.selected_strategy.clone())
                .or_default()
                .push(session);
        }

        // Calculate strategy performance adjustments
        for (strategy, group) in strategy_groups {
            let avg_satisfaction = group
                .iter()
                .filter_map(|s| s.user_satisfaction)
                .sum::<f64>()
                / group.len() as f64;

            let avg_improvement = group
                .iter()
                .filter_map(|s| s.quality_improvement)
                .sum::<f64>()
                / group.len() as f64;

            log::info!(
                "Strategy '{strategy}': avg satisfaction = {avg_satisfaction:.3}, avg improvement = {avg_improvement:.3}"
            );

            // In a real implementation, this would update the model weights
            // For now, we just log the insights
        }

        Ok(())
    }

    /// Calculate feature importance from feedback
    pub fn calculate_feature_importance(
        &self,
        sessions: &[AnonymizedSession],
    ) -> HashMap<String, f64> {
        let mut importance = HashMap::new();

        // Simple correlation analysis between features and satisfaction
        let feature_names = GraphFeatures::feature_names();

        for (i, &name) in feature_names.iter().enumerate() {
            let mut correlation = 0.0;
            let mut count = 0;

            for session in sessions {
                if let Some(satisfaction) = session.user_satisfaction {
                    let features = session.graph_features.to_vector();
                    if i < features.len() {
                        correlation += features[i] * satisfaction;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                importance.insert(name.to_string(), correlation / count as f64);
            }
        }

        importance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn test_feedback_collector() {
        let collector = FeedbackCollector::new(100);

        // Create a simple graph
        let document = ParsedDocument {
            config: GlobalConfig::default(),
            component_types: StdHashMap::new(),
            templates: StdHashMap::new(),
            diagram: None,
            nodes: vec![NodeDefinition {
                id: "a".to_string(),
                label: Some("Node A".to_string()),
                component_type: None,
                attributes: StdHashMap::new(),
            }],
            edges: vec![],
            containers: vec![],
            groups: vec![],
            connections: vec![],
        };

        let igr = IntermediateGraph::from_ast(document).unwrap();

        // Start session
        let session_id = collector
            .start_session(&igr, "dagre".to_string(), vec!["force".to_string()], 0.8)
            .unwrap();

        // Record adjustment
        let adjustment = LayoutAdjustment {
            timestamp: Utc::now(),
            adjustment_type: AdjustmentType::NodeMove {
                node_id: "a".to_string(),
                dx: 10.0,
                dy: 20.0,
            },
            magnitude: 22.36,
        };
        collector
            .record_adjustment(&session_id, adjustment)
            .unwrap();

        // Complete session
        collector
            .complete_session(&session_id, FeedbackType::MinorAdjust, 0.85, 5000)
            .unwrap();

        // Check stats
        let stats = collector.get_feedback_stats();
        assert_eq!(stats.total_sessions, 1);
        assert!(stats.avg_satisfaction_by_strategy.contains_key("dagre"));
    }

    #[test]
    fn test_feedback_types() {
        assert_eq!(FeedbackType::Accept.to_satisfaction_score(), 1.0);
        assert_eq!(FeedbackType::MinorAdjust.to_satisfaction_score(), 0.75);
        assert_eq!(FeedbackType::MajorAdjust.to_satisfaction_score(), 0.5);
        assert_eq!(FeedbackType::Reject.to_satisfaction_score(), 0.0);
        assert_eq!(FeedbackType::Rating(5).to_satisfaction_score(), 1.0);
        assert_eq!(FeedbackType::Rating(3).to_satisfaction_score(), 0.5);
        assert_eq!(FeedbackType::Rating(1).to_satisfaction_score(), 0.0);
    }
}
