use serde_json::{json, Value};

/// User preference learning and adaptation
pub struct PreferenceEngine;

impl PreferenceEngine {
    /// Calculate confidence based on pattern frequency and success rate
    pub fn calculate_confidence(frequency: i32, success_rate: f64, base_confidence: f64) -> f64 {
        // Confidence increases with frequency and success rate
        // Formula: base_confidence * (1 + log10(frequency + 1) / 10) * success_rate
        let freq_value = (frequency + 1).max(1);
        let frequency_factor = 1.0 + (freq_value as f64).log10() / 10.0;
        let confidence = base_confidence * frequency_factor * success_rate;
        
        confidence.min(1.0)
    }

    /// Aggregate preferences from multiple sources
    pub fn aggregate_preferences(
        preferences: &[(Value, f64)], // (value, confidence) pairs
    ) -> Option<(Value, f64)> {
        if preferences.is_empty() {
            return None;
        }

        // Find the preference with highest confidence
        let best = preferences
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        best.map(|(value, confidence)| (value.clone(), *confidence))
    }

    /// Update confidence based on user feedback
    pub fn update_confidence_from_feedback(
        current_confidence: f64,
        accepted: bool,
    ) -> f64 {
        if accepted {
            // Increase confidence (capped at 1.0)
            (current_confidence + 0.1).min(1.0)
        } else {
            // Decrease confidence (capped at 0.0)
            (current_confidence - 0.1).max(0.0)
        }
    }

    /// Merge preference values (for when we have multiple sources)
    pub fn merge_preference_values(values: &[Value]) -> Value {
        if values.is_empty() {
            return Value::Null;
        }

        if values.len() == 1 {
            return values[0].clone();
        }

        // For arrays, combine them
        if values.iter().all(|v| v.is_array()) {
            let mut merged = Vec::new();
            for val in values {
                if let Some(arr) = val.as_array() {
                    merged.extend_from_slice(arr);
                }
            }
            return json!(merged);
        }

        // For objects, merge them (last one wins for conflicts)
        if values.iter().all(|v| v.is_object()) {
            let mut merged = json!({});
            for val in values {
                if let Some(obj) = merged.as_object_mut() {
                    if let Some(val_obj) = val.as_object() {
                        for (key, value) in val_obj {
                            obj.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            return merged;
        }

        // For other types, return the most common one
        // Simple approach: return the first one
        values[0].clone()
    }
}

