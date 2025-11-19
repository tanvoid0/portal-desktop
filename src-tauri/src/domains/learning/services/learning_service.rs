use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use crate::domains::learning::repositories::{
    LearnedPatternRepository, UserPreferenceRepository, LearningEventRepository,
};
use crate::domains::learning::services::{
    MLIntensityManager, MLIntensity,
};
// FUTURE: PreferenceEngine is imported directly for now, will be re-exported when advanced learning features are fully implemented
use crate::domains::learning::services::preference_engine::PreferenceEngine;

/// Core learning orchestrator
pub struct LearningService {
    intensity_manager: MLIntensityManager,
}

impl LearningService {
    pub fn new(intensity: MLIntensity) -> Self {
        Self {
            intensity_manager: MLIntensityManager::new(intensity),
        }
    }

    pub fn with_default() -> Self {
        Self {
            intensity_manager: MLIntensityManager::with_default(),
        }
    }

    fn should_analyze_pattern(&self, pattern_type: &str) -> bool {
        self.intensity_manager.should_analyze_pattern(pattern_type)
    }

    fn max_patterns_per_context(&self) -> usize {
        self.intensity_manager.get_intensity().max_patterns_per_context()
    }

    /// Record a learning event
    pub async fn record_event(
        &self,
        db: &DatabaseConnection,
        event_type: String,
        event_data: Value,
        outcome: Option<String>,
        context: Option<String>,
    ) -> Result<i32, String> {
        let event = LearningEventRepository::create(
            db,
            event_type,
            serde_json::to_string(&event_data)
                .map_err(|e| format!("Failed to serialize event data: {}", e))?,
            outcome,
            context,
        )
        .await
        .map_err(|e| format!("Failed to create learning event: {}", e))?;

        Ok(event.id)
    }

    /// Learn a pattern from user behavior
    pub async fn learn_pattern(
        &self,
        db: &DatabaseConnection,
        pattern_type: String,
        pattern_data: Value,
        context: Option<String>,
    ) -> Result<i32, String> {
        if !self.should_analyze_pattern(&pattern_type) {
            return Err("Pattern type not analyzed at current intensity level".to_string());
        }

        let pattern_str = serde_json::to_string(&pattern_data)
            .map_err(|e| format!("Failed to serialize pattern data: {}", e))?;

        let pattern = LearnedPatternRepository::find_or_create(
            db,
            pattern_type,
            pattern_str,
            context,
        )
        .await
        .map_err(|e| format!("Failed to create pattern: {}", e))?;

        // Increment frequency if pattern already existed
        if pattern.frequency == 1 {
            Ok(pattern.id)
        } else {
            let updated = LearnedPatternRepository::increment_frequency(db, pattern.id, true)
                .await
                .map_err(|e| format!("Failed to update pattern frequency: {}", e))?;
            Ok(updated.id)
        }
    }

    /// Record pattern success/failure
    pub async fn record_pattern_outcome(
        &self,
        db: &DatabaseConnection,
        pattern_id: i32,
        success: bool,
    ) -> Result<(), String> {
        LearnedPatternRepository::increment_frequency(db, pattern_id, success)
            .await
            .map_err(|e| format!("Failed to update pattern outcome: {}", e))?;
        Ok(())
    }

    /// Get suggestions based on context
    /// Uses pattern matching to rank and filter suggestions
    pub async fn get_suggestions(
        &self,
        db: &DatabaseConnection,
        pattern_type: &str,
        context: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        let patterns = LearnedPatternRepository::get_by_type_and_context(
            db,
            pattern_type,
            context,
        )
        .await
        .map_err(|e| format!("Failed to get patterns: {}", e))?;

        // Use pattern matcher to score and rank patterns
        let mut scored_patterns: Vec<(f64, _)> = patterns
            .iter()
            .map(|pattern| {
                // Calculate score: frequency * success_rate with context matching bonus
                let base_score = pattern.frequency as f64 * pattern.success_rate;
                
                // Context matching bonus (higher score for exact context match)
                let context_bonus = if let (Some(ctx), Some(pattern_ctx)) = (context, &pattern.context) {
                    if ctx == pattern_ctx.as_str() {
                        1.5 // Exact match
                    } else if pattern_ctx.starts_with(ctx) || ctx.starts_with(pattern_ctx.as_str()) {
                        1.2 // Partial match
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };
                
                let final_score = base_score * context_bonus;
                (final_score, pattern)
            })
            .collect();

        // Sort by score (highest first)
        scored_patterns.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let max_patterns = self.max_patterns_per_context();
        let top_patterns: Vec<_> = scored_patterns
            .into_iter()
            .take(max_patterns)
            .map(|(_, pattern)| pattern.clone())
            .collect();

        let mut suggestions = Vec::new();
        for pattern in top_patterns {
            if let Ok(data) = serde_json::from_str::<Value>(&pattern.pattern_data) {
                suggestions.push(json!({
                    "pattern_id": pattern.id,
                    "pattern_data": data,
                    "frequency": pattern.frequency,
                    "success_rate": pattern.success_rate,
                    "context": pattern.context,
                }));
            }
        }

        Ok(suggestions)
    }

    /// Learn user preference
    /// Uses preference engine to calculate confidence based on context and frequency
    pub async fn learn_preference(
        &self,
        db: &DatabaseConnection,
        preference_type: String,
        context: Option<String>,
        preference_value: Value,
        learned_from: Option<String>,
    ) -> Result<i32, String> {
        let value_str = serde_json::to_string(&preference_value)
            .map_err(|e| format!("Failed to serialize preference value: {}", e))?;

        // Calculate initial confidence using preference engine
        // Check if preference already exists to aggregate
        let existing_pref = UserPreferenceRepository::get_by_type_and_context(
            db,
            &preference_type,
            context.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to check existing preference: {}", e))?;

        let confidence = if let Some(existing) = existing_pref {
            // Check if the values match - if they do, increase confidence; if not, decrease
            let existing_value: Value = serde_json::from_str(&existing.preference_value)
                .unwrap_or_else(|_| json!(null));
            
            let values_match = existing_value == preference_value;
            
            if values_match {
                // Same value: increase confidence (capped at 1.0)
                PreferenceEngine::update_confidence_from_feedback(existing.confidence, true)
            } else {
                // Different value: decrease confidence
                // This means user preference changed, so lower confidence
                (existing.confidence * 0.8).max(0.3)
            }
        } else {
            // New preference: start with base confidence based on source
            match learned_from.as_deref() {
                Some("user_selection") => 0.8, // High confidence for explicit user choices
                Some("pattern_analysis") => 0.6, // Medium for inferred patterns
                Some("user_setting") => 0.9, // Very high for explicit settings
                _ => 0.5, // Default for unknown sources
            }
        };

        let preference = UserPreferenceRepository::find_or_create(
            db,
            preference_type,
            context,
            value_str,
            confidence,
            learned_from,
        )
        .await
        .map_err(|e| format!("Failed to create preference: {}", e))?;

        Ok(preference.id)
    }

    /// Get user preference
    pub async fn get_preference(
        &self,
        db: &DatabaseConnection,
        preference_type: &str,
        context: Option<&str>,
    ) -> Result<Option<Value>, String> {
        let preference = UserPreferenceRepository::get_by_type_and_context(
            db,
            preference_type,
            context,
        )
        .await
        .map_err(|e| format!("Failed to get preference: {}", e))?;

        if let Some(pref) = preference {
            let value: Value = serde_json::from_str(&pref.preference_value)
                .map_err(|e| format!("Failed to parse preference value: {}", e))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

