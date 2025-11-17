use sea_orm::DatabaseConnection;
use crate::domains::autonomy::services::{ActionClassifier, ApprovalManager};
use crate::domains::learning::services::LearningService;
use crate::domains::learning::repositories::LearnedPatternRepository;
use serde::{Deserialize, Serialize};

/// Autonomous action request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousActionRequest {
    pub action_type: String,
    pub action_data: serde_json::Value,
    pub context: String,
    pub user_id: Option<String>,
}

/// Autonomous action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousActionResult {
    pub action_id: String,
    pub executed: bool,
    pub requires_approval: bool,
    pub classification: crate::domains::autonomy::services::action_classifier::ActionClassification,
    pub message: String,
}

/// Core service for managing autonomous actions
pub struct AutonomyService {
    classifier: ActionClassifier,
    approval_manager: ApprovalManager,
    learning_service: LearningService,
    autonomy_enabled: bool,
    autonomy_level: AutonomyLevel,
}

/// Autonomy level determines how aggressive the system is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyLevel {
    /// Observation only - no autonomous actions
    Observation,
    /// Safe actions only
    Conservative,
    /// Safe and low-risk actions
    Balanced,
    /// Safe, low-risk, and medium-risk (with high confidence)
    Aggressive,
}

impl Default for AutonomyLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

impl Default for AutonomyService {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomyService {
    pub fn new() -> Self {
        Self {
            classifier: ActionClassifier::new(),
            approval_manager: ApprovalManager::new(),
            learning_service: LearningService::with_default(),
            autonomy_enabled: true,
            autonomy_level: AutonomyLevel::default(),
        }
    }

    /// Evaluate if an action should be executed autonomously
    pub async fn evaluate_action(
        &mut self,
        db: &DatabaseConnection,
        request: AutonomousActionRequest,
    ) -> Result<AutonomousActionResult, String> {
        if !self.autonomy_enabled {
            return Ok(AutonomousActionResult {
                action_id: uuid::Uuid::new_v4().to_string(),
                executed: false,
                requires_approval: true,
                classification: self.classifier.classify(&request.action_type, &request.context, 0.0),
                message: "Autonomy is disabled".to_string(),
            });
        }

        // Get success rate from learned patterns
        let success_rate = self.get_action_success_rate(db, &request.action_type, &request.context).await?;

        // Classify the action
        let classification = self.classifier.classify(&request.action_type, &request.context, success_rate);

        // Check if auto-approval is allowed
        let safety_level = classification.safety_level;
        let should_auto_approve = self.approval_manager.should_auto_approve(
            &request.action_type,
            &request.context,
            safety_level,
        ) && self.is_allowed_by_level(safety_level);

        let action_id = uuid::Uuid::new_v4().to_string();
        let requires_approval_val = classification.requires_approval;

        Ok(AutonomousActionResult {
            action_id,
            executed: should_auto_approve && !requires_approval_val,
            requires_approval: !should_auto_approve || requires_approval_val,
            classification,
            message: if should_auto_approve && !requires_approval_val {
                "Action can be executed autonomously".to_string()
            } else {
                "Action requires user approval".to_string()
            },
        })
    }

    /// Check if action is allowed by current autonomy level
    fn is_allowed_by_level(&self, safety_level: crate::domains::autonomy::services::action_classifier::ActionSafetyLevel) -> bool {
        match (self.autonomy_level, safety_level) {
            (AutonomyLevel::Observation, _) => false,
            (AutonomyLevel::Conservative, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::Safe) => true,
            (AutonomyLevel::Balanced, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::Safe) => true,
            (AutonomyLevel::Balanced, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::LowRisk) => true,
            (AutonomyLevel::Aggressive, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::Safe) => true,
            (AutonomyLevel::Aggressive, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::LowRisk) => true,
            (AutonomyLevel::Aggressive, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::MediumRisk) => true,
            _ => false,
        }
    }

    /// Get success rate for an action type from learned patterns
    async fn get_action_success_rate(
        &self,
        db: &DatabaseConnection,
        action_type: &str,
        context: &str,
    ) -> Result<f64, String> {
        // Query learned patterns for this action type
        let patterns = LearnedPatternRepository::get_by_type_and_context(db, action_type, Some(context))
            .await
            .map_err(|e| format!("Failed to query patterns: {}", e))?;

        if patterns.is_empty() {
            return Ok(0.5); // Default moderate confidence
        }

        // Calculate weighted average success rate
        let total_weight = patterns.iter()
            .map(|p| p.frequency as f64)
            .sum::<f64>();

        if total_weight == 0.0 {
            return Ok(0.5);
        }

        let weighted_sum = patterns.iter()
            .map(|p| p.success_rate * p.frequency as f64)
            .sum::<f64>();

        Ok(weighted_sum / total_weight)
    }

    /// Record action outcome for learning
    pub async fn record_action_outcome(
        &mut self,
        db: &DatabaseConnection,
        action_id: &str,
        action_type: &str,
        context: &str,
        success: bool,
        feedback: Option<String>,
    ) -> Result<(), String> {
        // Record in approval manager
        self.approval_manager.record_decision(
            action_id.to_string(),
            action_type.to_string(),
            context.to_string(),
            if success {
                crate::domains::autonomy::services::approval_manager::ApprovalDecision::Approved
            } else {
                crate::domains::autonomy::services::approval_manager::ApprovalDecision::Rejected
            },
            feedback,
        );

        // Learn pattern
        self.learning_service
            .learn_pattern(
                db,
                action_type.to_string(),
                serde_json::json!({ "action_type": action_type, "success": success }),
                Some(context.to_string()),
            )
            .await?;

        Ok(())
    }

    /// Set autonomy level
    pub fn set_autonomy_level(&mut self, level: AutonomyLevel) {
        self.autonomy_level = level;
    }

    /// Get autonomy level
    pub fn get_autonomy_level(&self) -> AutonomyLevel {
        self.autonomy_level
    }

    /// Enable/disable autonomy
    pub fn set_enabled(&mut self, enabled: bool) {
        self.autonomy_enabled = enabled;
    }

    /// Check if autonomy is enabled
    pub fn is_enabled(&self) -> bool {
        self.autonomy_enabled
    }

    /// Get approval statistics
    pub fn get_approval_stats(&self) -> std::collections::HashMap<String, (u32, u32)> {
        self.approval_manager.get_approval_stats()
    }
}
