use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use chrono;

/// Code pattern analyzer for learning best practices and patterns across projects
pub struct CodePatternAnalyzer;

impl CodePatternAnalyzer {
    /// Analyze file structure patterns
    pub fn analyze_file_structure(files: &[String]) -> Value {
        let mut patterns = HashMap::new();
        
        // Count file extensions
        let mut extensions: HashMap<String, usize> = HashMap::new();
        for file in files {
            if let Some(ext) = Path::new(file).extension() {
                if let Some(ext_str) = ext.to_str() {
                    *extensions.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            }
        }

        patterns.insert("extensions".to_string(), json!(extensions));
        
        // Detect common directory patterns
        let mut directory_patterns = Vec::new();
        for file in files {
            if let Some(parent) = Path::new(file).parent() {
                if let Some(parent_str) = parent.to_str() {
                    if !directory_patterns.contains(&parent_str.to_string()) {
                        directory_patterns.push(parent_str.to_string());
                    }
                }
            }
        }
        patterns.insert("directories".to_string(), json!(directory_patterns));

        json!(patterns)
    }

    /// Analyze import/require patterns
    pub fn analyze_import_patterns(imports: &[String]) -> Value {
        let mut patterns = HashMap::new();
        
        // Group by package/library
        let mut package_counts: HashMap<String, usize> = HashMap::new();
        for import in imports {
            // Extract package name (first part before '/')
            let package = import.split('/').next().unwrap_or_else(|| import.as_str());
            *package_counts.entry(package.to_string()).or_insert(0) += 1;
        }
        
        patterns.insert("packages".to_string(), json!(package_counts));
        patterns.insert("total_imports".to_string(), json!(imports.len()));

        json!(patterns)
    }

    /// Detect common code patterns
    pub fn detect_common_patterns(code_content: &str) -> Value {
        let mut patterns = HashMap::new();
        
        // Detect async/await patterns
        let async_count = code_content.matches("async").count();
        let await_count = code_content.matches("await").count();
        if async_count > 0 || await_count > 0 {
            patterns.insert("async_pattern".to_string(), json!(true));
        }

        // Detect error handling patterns
        let try_count = code_content.matches("try").count();
        let catch_count = code_content.matches("catch").count();
        let error_count = code_content.matches("error").count();
        if try_count > 0 || catch_count > 0 || error_count > 0 {
            patterns.insert("error_handling".to_string(), json!(true));
        }

        // Detect test patterns
        let test_count = code_content.matches("test").count();
        let describe_count = code_content.matches("describe").count();
        if test_count > 0 || describe_count > 0 {
            patterns.insert("testing_pattern".to_string(), json!(true));
        }

        json!(patterns)
    }

    /// Build code pattern summary
    pub fn build_pattern_summary(
        file_structure: &Value,
        imports: &Value,
        common_patterns: &Value,
    ) -> Value {
        json!({
            "file_structure": file_structure,
            "imports": imports,
            "common_patterns": common_patterns,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
    }
}

