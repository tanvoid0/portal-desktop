use serde::{Deserialize, Serialize};

/// Safety levels for autonomous actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionSafetyLevel {
    /// Safe actions - auto-execute without approval (read-only, suggestions, UI updates)
    Safe,
    /// Low risk actions - auto-execute after threshold (common commands like npm install)
    LowRisk,
    /// Medium risk actions - suggest first (configuration changes, file creation)
    MediumRisk,
    /// High risk actions - always require confirmation (file deletion, system changes)
    HighRisk,
}

/// Action classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionClassification {
    pub safety_level: ActionSafetyLevel,
    pub confidence: f64,
    pub requires_approval: bool,
    pub reason: String,
}

/// Classifies actions based on their safety level
pub struct ActionClassifier {
    // User-defined thresholds for each safety level
    safe_threshold: f64,
    low_risk_threshold: f64,
    medium_risk_threshold: f64,
}

impl Default for ActionClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionClassifier {
    pub fn new() -> Self {
        Self {
            safe_threshold: 0.95,
            low_risk_threshold: 0.85,
            medium_risk_threshold: 0.70,
        }
    }

    /// Classify an action based on its type and context
    pub fn classify(&self, action_type: &str, context: &str, success_rate: f64) -> ActionClassification {
        // Determine base safety level from action type
        let base_level = self.get_base_safety_level(action_type);
        
        // Adjust based on success rate and context
        let requires_approval = match base_level {
            ActionSafetyLevel::Safe => false,
            ActionSafetyLevel::LowRisk => success_rate < self.low_risk_threshold,
            ActionSafetyLevel::MediumRisk => success_rate < self.medium_risk_threshold || success_rate < self.safe_threshold,
            ActionSafetyLevel::HighRisk => true,
        };

        let confidence = self.calculate_confidence(base_level, success_rate, context);
        let reason = self.generate_reason(base_level, success_rate, requires_approval);

        ActionClassification {
            safety_level: base_level,
            confidence,
            requires_approval,
            reason,
        }
    }

    /// Get base safety level from action type
    fn get_base_safety_level(&self, action_type: &str) -> ActionSafetyLevel {
        let action_lower = action_type.to_lowercase();
        
        // Safe actions
        if action_lower.contains("read") || 
           action_lower.contains("list") || 
           action_lower.contains("get") ||
           action_lower.contains("suggest") ||
           action_lower.contains("display") {
            return ActionSafetyLevel::Safe;
        }

        // High risk actions
        if action_lower.contains("delete") || 
           action_lower.contains("remove") ||
           action_lower.contains("uninstall") ||
           action_lower.contains("destroy") ||
           action_lower.contains("drop") ||
           action_lower.contains("format") {
            return ActionSafetyLevel::HighRisk;
        }

        // Medium risk actions
        if action_lower.contains("create") || 
           action_lower.contains("update") ||
           action_lower.contains("modify") ||
           action_lower.contains("configure") ||
           action_lower.contains("install") {
            return ActionSafetyLevel::MediumRisk;
        }

        // Low risk actions (default for common commands)
        ActionSafetyLevel::LowRisk
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, level: ActionSafetyLevel, success_rate: f64, _context: &str) -> f64 {
        let base_confidence = match level {
            ActionSafetyLevel::Safe => 0.95,
            ActionSafetyLevel::LowRisk => 0.80,
            ActionSafetyLevel::MediumRisk => 0.65,
            ActionSafetyLevel::HighRisk => 0.30,
        };

        // Adjust based on success rate
        (base_confidence * 0.5) + (success_rate * 0.5)
    }

    /// Generate human-readable reason
    fn generate_reason(&self, level: ActionSafetyLevel, success_rate: f64, requires_approval: bool) -> String {
        match (level, requires_approval) {
            (ActionSafetyLevel::Safe, _) => "Safe read-only operation".to_string(),
            (ActionSafetyLevel::LowRisk, true) => format!(
                "Low risk action, but success rate ({:.0}%) is below threshold",
                success_rate * 100.0
            ),
            (ActionSafetyLevel::LowRisk, false) => format!(
                "Low risk action with good success rate ({:.0}%)",
                success_rate * 100.0
            ),
            (ActionSafetyLevel::MediumRisk, true) => format!(
                "Medium risk action requiring approval (success rate: {:.0}%)",
                success_rate * 100.0
            ),
            (ActionSafetyLevel::MediumRisk, false) => format!(
                "Medium risk action, but high confidence ({:.0}%)",
                success_rate * 100.0
            ),
            (ActionSafetyLevel::HighRisk, _) => "High risk action - always requires approval".to_string(),
        }
    }

    /// Update thresholds (for user customization)
    pub fn set_thresholds(&mut self, safe: f64, low_risk: f64, medium_risk: f64) {
        self.safe_threshold = safe;
        self.low_risk_threshold = low_risk;
        self.medium_risk_threshold = medium_risk;
    }
}
