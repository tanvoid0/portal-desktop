pub mod learning_service;
pub mod pattern_matcher;
pub mod preference_engine;
pub mod context_analyzer;
pub mod code_pattern_analyzer;
pub mod ml_intensity_manager;
pub mod memory_manager;
pub mod context_manager;
pub mod adapters;

pub use learning_service::LearningService;
pub use pattern_matcher::PatternMatcher;
pub use pattern_matcher::Pattern;
pub use preference_engine::PreferenceEngine;
pub use context_analyzer::ContextAnalyzer;
pub use code_pattern_analyzer::CodePatternAnalyzer;
pub use ml_intensity_manager::{MLIntensityManager, MLIntensity};
pub use memory_manager::MemoryManager;
pub use context_manager::ContextManager;
pub use adapters::*;

