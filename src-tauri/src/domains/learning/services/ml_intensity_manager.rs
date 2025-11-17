use serde::{Deserialize, Serialize};

/// ML Intensity levels that balance intelligence vs performance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLIntensity {
    /// Fast: Minimal learning, basic pattern recognition only
    Fast,
    /// Light: Limited pattern analysis, basic suggestions
    Light,
    /// Medium: Full pattern analysis, contextual suggestions
    Medium,
    /// Deep: Advanced analysis, code pattern learning, predictive actions
    Deep,
}

impl MLIntensity {
    /// Get the default intensity level
    pub fn default() -> Self {
        MLIntensity::Medium
    }

    /// Convert string to MLIntensity
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fast" => MLIntensity::Fast,
            "light" => MLIntensity::Light,
            "medium" => MLIntensity::Medium,
            "deep" => MLIntensity::Deep,
            _ => MLIntensity::default(),
        }
    }

    /// Convert MLIntensity to string
    pub fn to_string(&self) -> &'static str {
        match self {
            MLIntensity::Fast => "fast",
            MLIntensity::Light => "light",
            MLIntensity::Medium => "medium",
            MLIntensity::Deep => "deep",
        }
    }

    /// Get processing batch size based on intensity
    /// Lower intensity = larger batches (less frequent processing)
    pub fn batch_size(&self) -> usize {
        match self {
            MLIntensity::Fast => 100,      // Process in large batches
            MLIntensity::Light => 50,       // Medium batches
            MLIntensity::Medium => 25,      // Smaller batches
            MLIntensity::Deep => 10,        // Small batches for detailed analysis
        }
    }

    /// Get processing delay in milliseconds
    /// Higher intensity = more frequent processing
    pub fn processing_delay_ms(&self) -> u64 {
        match self {
            MLIntensity::Fast => 60000,     // Process every 60 seconds
            MLIntensity::Light => 30000,    // Process every 30 seconds
            MLIntensity::Medium => 15000,   // Process every 15 seconds
            MLIntensity::Deep => 5000,      // Process every 5 seconds
        }
    }

    /// Whether to enable code pattern analysis
    pub fn enable_code_analysis(&self) -> bool {
        matches!(self, MLIntensity::Medium | MLIntensity::Deep)
    }

    /// Whether to enable deep pattern matching
    pub fn enable_deep_matching(&self) -> bool {
        matches!(self, MLIntensity::Deep)
    }

    /// Whether to enable predictive actions
    pub fn enable_predictive_actions(&self) -> bool {
        matches!(self, MLIntensity::Deep)
    }

    /// Maximum patterns to analyze per context
    pub fn max_patterns_per_context(&self) -> usize {
        match self {
            MLIntensity::Fast => 10,
            MLIntensity::Light => 25,
            MLIntensity::Medium => 50,
            MLIntensity::Deep => 100,
        }
    }
}

pub struct MLIntensityManager {
    intensity: MLIntensity,
}

impl MLIntensityManager {
    pub fn new(intensity: MLIntensity) -> Self {
        Self { intensity }
    }

    pub fn with_default() -> Self {
        Self {
            intensity: MLIntensity::default(),
        }
    }

    pub fn get_intensity(&self) -> MLIntensity {
        self.intensity
    }

    pub fn set_intensity(&mut self, intensity: MLIntensity) {
        self.intensity = intensity;
    }

    /// Check if a pattern should be analyzed based on intensity
    pub fn should_analyze_pattern(&self, pattern_type: &str) -> bool {
        match self.intensity {
            MLIntensity::Fast => {
                // Only analyze command patterns
                pattern_type == "command"
            }
            MLIntensity::Light => {
                // Analyze commands and workflows
                matches!(pattern_type, "command" | "workflow")
            }
            MLIntensity::Medium => {
                // Analyze most patterns except code
                !matches!(pattern_type, "code")
            }
            MLIntensity::Deep => {
                // Analyze all patterns
                true
            }
        }
    }
}

