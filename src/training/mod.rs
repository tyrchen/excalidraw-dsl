// src/training/mod.rs
//! ML Layout Training Module
//!
//! This module provides comprehensive training capabilities for ML layout models.

pub mod config;
pub mod constraints;
pub mod data;
pub mod enhanced;
pub mod gnn;
pub mod rl;
pub mod utils;

pub use config::{ModelConfig, TrainingConfig, TrainingPhase};
pub use constraints::ConstraintTrainer;
pub use data::{DatasetStats, TrainingDataGenerator};
pub use enhanced::EnhancedTrainer;
pub use gnn::GNNTrainer;
pub use rl::RLTrainer;
pub use utils::{MetricsCollector, TrainingLogger};

use crate::error::Result;
use std::path::Path;

/// Main training orchestrator
pub struct TrainingOrchestrator {
    config: TrainingConfig,
    logger: TrainingLogger,
    metrics: MetricsCollector,
}

impl TrainingOrchestrator {
    pub fn new(config: TrainingConfig) -> Result<Self> {
        let logger = TrainingLogger::new(&config.output_dir)?;
        let metrics = MetricsCollector::new();

        Ok(Self {
            config,
            logger,
            metrics,
        })
    }

    /// Run complete training pipeline
    pub fn run_training(&mut self) -> Result<()> {
        self.logger.info("🚀 Starting ML Layout Training Pipeline");

        // Phase 1: Data Generation
        if matches!(
            self.config.phase,
            TrainingPhase::DataGeneration | TrainingPhase::All
        ) {
            self.run_data_generation()?;
        }

        // Phase 2: GNN Training
        if matches!(
            self.config.phase,
            TrainingPhase::GNNTraining | TrainingPhase::All
        ) {
            self.run_gnn_training()?;
        }

        // Phase 3: RL Training
        if matches!(
            self.config.phase,
            TrainingPhase::RLTraining | TrainingPhase::All
        ) {
            self.run_rl_training()?;
        }

        // Phase 4: Constraint Training
        if matches!(
            self.config.phase,
            TrainingPhase::ConstraintTraining | TrainingPhase::All
        ) {
            self.run_constraint_training()?;
        }

        // Phase 5: Enhanced Model Training
        if matches!(
            self.config.phase,
            TrainingPhase::EnhancedTraining | TrainingPhase::All
        ) {
            self.run_enhanced_training()?;
        }

        // Phase 6: Evaluation
        if matches!(
            self.config.phase,
            TrainingPhase::Evaluation | TrainingPhase::All
        ) {
            self.run_evaluation()?;
        }

        self.logger
            .info("✅ Training pipeline completed successfully");
        self.save_final_metrics()?;

        Ok(())
    }

    pub fn run_data_generation(&mut self) -> Result<()> {
        self.logger.info("📊 Phase 1: Generating Training Data");

        let mut generator = TrainingDataGenerator::new(&self.config.data_config)?;
        let stats = generator.generate_datasets(&self.config.output_dir)?;

        self.logger.info(&format!(
            "Generated {} training samples",
            stats.total_samples
        ));
        self.metrics.record_data_generation(stats);

        Ok(())
    }

    pub fn run_gnn_training(&mut self) -> Result<()> {
        self.logger.info("🧠 Phase 2: Training GNN Model");

        let mut trainer = GNNTrainer::new(&self.config.gnn_config)?;
        let model_path = self.config.output_dir.join("models/gnn_model.bin");

        let training_results = trainer.train(&model_path)?;

        self.logger.info(&format!(
            "GNN training completed with loss: {:.4}",
            training_results.final_loss
        ));
        self.metrics.record_gnn_training(training_results);

        Ok(())
    }

    pub fn run_rl_training(&mut self) -> Result<()> {
        self.logger.info("🎮 Phase 3: Training RL Model");

        let mut trainer = RLTrainer::new(&self.config.rl_config)?;
        let model_path = self.config.output_dir.join("models/rl_model.bin");

        let training_results = trainer.train(&model_path)?;

        self.logger.info(&format!(
            "RL training completed with avg reward: {:.4}",
            training_results.avg_reward
        ));
        self.metrics.record_rl_training(training_results);

        Ok(())
    }

    pub fn run_constraint_training(&mut self) -> Result<()> {
        self.logger.info("🧩 Phase 4: Training Constraint Solver");

        let mut trainer = ConstraintTrainer::new(&self.config.constraint_config)?;
        let model_path = self.config.output_dir.join("models/constraint_model.bin");

        let training_results = trainer.train(&model_path)?;

        self.logger.info(&format!(
            "Constraint training completed with accuracy: {:.4}",
            training_results.accuracy
        ));
        self.metrics.record_constraint_training(training_results);

        Ok(())
    }

    pub fn run_enhanced_training(&mut self) -> Result<()> {
        self.logger.info("🔄 Phase 5: Training Enhanced Model");

        let mut trainer = EnhancedTrainer::new(&self.config.enhanced_config)?;
        let model_path = self.config.output_dir.join("models/enhanced_model.bin");

        let training_results = trainer.train(&model_path)?;

        self.logger.info(&format!(
            "Enhanced training completed with score: {:.4}",
            training_results.quality_score
        ));
        self.metrics.record_enhanced_training(training_results);

        Ok(())
    }

    pub fn run_evaluation(&mut self) -> Result<()> {
        self.logger.info("📊 Phase 6: Model Evaluation");

        // Load and evaluate all trained models
        let gnn_path = self.config.output_dir.join("models/gnn_model.bin");
        let rl_path = self.config.output_dir.join("models/rl_model.bin");
        let constraint_path = self.config.output_dir.join("models/constraint_model.bin");
        let enhanced_path = self.config.output_dir.join("models/enhanced_model.bin");

        let evaluation_results = utils::evaluate_all_models(
            &gnn_path,
            &rl_path,
            &constraint_path,
            &enhanced_path,
            &self.config.evaluation_config,
        )?;

        self.logger.info("Model evaluation completed");
        self.metrics.record_evaluation(evaluation_results);

        Ok(())
    }

    fn save_final_metrics(&self) -> Result<()> {
        let metrics_path = self.config.output_dir.join("training_metrics.json");
        self.metrics.save_to_file(&metrics_path)?;

        let report_path = self.config.output_dir.join("training_report.md");
        self.metrics.generate_report(&report_path)?;

        Ok(())
    }

    /// Save training metrics to file
    pub fn save_metrics(&self, path: &Path) -> Result<()> {
        self.metrics.save_to_file(path)
    }

    /// Generate training report
    pub fn generate_report(&self, path: &Path) -> Result<()> {
        self.metrics.generate_report(path)
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Option<&MetricsCollector> {
        Some(&self.metrics)
    }
}
