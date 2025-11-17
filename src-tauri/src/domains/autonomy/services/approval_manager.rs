use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Approval decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    Pending,
}

/// Action approval record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionApproval {
    pub action_id: String,
    pub action_type: String,
    pub context: String,
    pub decision: ApprovalDecision,
    pub timestamp: i64,
    pub user_feedback: Option<String>,
}

/// Manages action approvals and learning from user decisions
pub struct ApprovalManager {
    // Cache of recent approvals (in-memory)
    recent_approvals: HashMap<String, ActionApproval>,
    // Pattern-based auto-approval rules (learned from user behavior)
    auto_approval_rules: Vec<AutoApprovalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AutoApprovalRule {
    pattern_type: String,
    context_pattern: String,
    success_count: u32,
    total_count: u32,
    auto_approve: bool,
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            recent_approvals: HashMap::new(),
            auto_approval_rules: Vec::new(),
        }
    }

    /// Check if an action should be auto-approved based on learned patterns
    pub fn should_auto_approve(&self, action_type: &str, context: &str, safety_level: crate::domains::autonomy::services::action_classifier::ActionSafetyLevel) -> bool {
        // High risk actions never auto-approve
        if matches!(safety_level, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::HighRisk) {
            return false;
        }

        // Safe actions always auto-approve
        if matches!(safety_level, crate::domains::autonomy::services::action_classifier::ActionSafetyLevel::Safe) {
            return true;
        }

        // Check learned patterns
        for rule in &self.auto_approval_rules {
            if rule.matches(action_type, context) && rule.should_auto_approve() {
                return true;
            }
        }

        false
    }

    /// Record an approval decision for learning
    pub fn record_decision(&mut self, action_id: String, action_type: String, context: String, decision: ApprovalDecision, feedback: Option<String>) {
        let approval = ActionApproval {
            action_id: action_id.clone(),
            action_type: action_type.clone(),
            context: context.clone(),
            decision,
            timestamp: chrono::Utc::now().timestamp(),
            user_feedback: feedback,
        };

        self.recent_approvals.insert(action_id, approval.clone());

        // Update auto-approval rules based on decision
        self.update_auto_approval_rule(&action_type, &context, decision == ApprovalDecision::Approved);

        // Limit cache size
        if self.recent_approvals.len() > 1000 {
            let oldest_key = self.recent_approvals
                .iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone());
            
            if let Some(key) = oldest_key {
                self.recent_approvals.remove(&key);
            }
        }
    }

    /// Update auto-approval rule based on user decision
    fn update_auto_approval_rule(&mut self, action_type: &str, context: &str, approved: bool) {
        // Find or create rule
        let rule = self.auto_approval_rules
            .iter_mut()
            .find(|r| r.pattern_type == action_type && r.context_pattern == context);

        if let Some(rule) = rule {
            rule.total_count += 1;
            if approved {
                rule.success_count += 1;
            }
            
            // Auto-approve if success rate > 90% and at least 5 attempts
            rule.auto_approve = rule.total_count >= 5 && 
                               (rule.success_count as f64 / rule.total_count as f64) > 0.90;
        } else {
            let new_rule = AutoApprovalRule {
                pattern_type: action_type.to_string(),
                context_pattern: context.to_string(),
                success_count: if approved { 1 } else { 0 },
                total_count: 1,
                auto_approve: false,
            };
            
            // Don't auto-approve on first attempt
            self.auto_approval_rules.push(new_rule);
        }
    }

    /// Get approval history for an action type
    pub fn get_approval_history(&self, action_type: &str, limit: usize) -> Vec<&ActionApproval> {
        self.recent_approvals
            .values()
            .filter(|a| a.action_type == action_type)
            .take(limit)
            .collect()
    }

    /// Get approval statistics
    pub fn get_approval_stats(&self) -> HashMap<String, (u32, u32)> {
        let mut stats = HashMap::new();
        
        for approval in self.recent_approvals.values() {
            let entry = stats.entry(approval.action_type.clone())
                .or_insert((0, 0));
            
            if approval.decision == ApprovalDecision::Approved {
                entry.0 += 1;
            }
            entry.1 += 1;
        }
        
        stats
    }
}

impl AutoApprovalRule {
    fn matches(&self, action_type: &str, context: &str) -> bool {
        self.pattern_type == action_type && self.context_pattern == context
    }

    fn should_auto_approve(&self) -> bool {
        self.auto_approve
    }
}
