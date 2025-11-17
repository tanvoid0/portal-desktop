use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Pattern structure for learning adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: String,
    pub pattern_data: Value,
    pub context: String,
    pub frequency: i32,
}

/// Pattern matching algorithms for learning system
pub struct PatternMatcher;

impl PatternMatcher {
    /// Simple frequency analysis for patterns
    pub fn frequency_analysis(patterns: &[String]) -> HashMap<String, usize> {
        let mut frequency: HashMap<String, usize> = HashMap::new();
        
        for pattern in patterns {
            *frequency.entry(pattern.clone()).or_insert(0) += 1;
        }
        
        frequency
    }

    /// Sequence pattern matching (Markov-like chains for command sequences)
    pub fn sequence_pattern_matching(sequences: &[Vec<String>]) -> HashMap<String, Vec<String>> {
        let mut transitions: HashMap<String, Vec<String>> = HashMap::new();
        
        for sequence in sequences {
            for i in 0..sequence.len().saturating_sub(1) {
                let current = &sequence[i];
                let next = &sequence[i + 1];
                
                transitions
                    .entry(current.clone())
                    .or_insert_with(Vec::new)
                    .push(next.clone());
            }
        }
        
        transitions
    }

    /// Template matching for project structures
    pub fn template_matching(structure: &Value) -> Option<String> {
        // Basic template matching based on key patterns
        if let Some(obj) = structure.as_object() {
            // Check for common project templates
            if obj.contains_key("package.json") {
                return Some("nodejs_project".to_string());
            }
            if obj.contains_key("Cargo.toml") {
                return Some("rust_project".to_string());
            }
            if obj.contains_key("requirements.txt") {
                return Some("python_project".to_string());
            }
            if obj.contains_key("go.mod") {
                return Some("go_project".to_string());
            }
        }
        None
    }

    /// Weighted scoring for context matching
    pub fn weighted_context_score(
        pattern_context: Option<&str>,
        target_context: Option<&str>,
        pattern_frequency: i32,
        success_rate: f64,
    ) -> f64 {
        let mut score = 0.0;
        
        // Context match bonus (50% weight)
        if pattern_context == target_context {
            score += 0.5;
        } else if pattern_context.is_none() || target_context.is_none() {
            // Partial match for global patterns
            score += 0.25;
        }
        
        // Frequency normalization (30% weight)
        // Normalize frequency to 0-1 range (assuming max frequency of 100)
        let freq_score = (pattern_frequency.min(100) as f64 / 100.0) * 0.3;
        score += freq_score;
        
        // Success rate (20% weight)
        score += success_rate * 0.2;
        
        score.min(1.0)
    }

    /// Find best matching pattern
    pub fn find_best_match(
        patterns: &[(String, Option<String>, i32, f64)], // (pattern_data, context, frequency, success_rate)
        target_context: Option<&str>,
    ) -> Option<(String, f64)> {
        let mut best_match: Option<(String, f64)> = None;
        let mut best_score = 0.0;
        
        for (pattern_data, pattern_context, frequency, success_rate) in patterns {
            let score = Self::weighted_context_score(
                pattern_context.as_deref(),
                target_context,
                *frequency,
                *success_rate,
            );
            
            if score > best_score {
                best_score = score;
                best_match = Some((pattern_data.clone(), score));
            }
        }
        
        best_match
    }
}

