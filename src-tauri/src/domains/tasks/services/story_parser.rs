use serde::{Deserialize, Serialize};

/// Parsed information from any story/description text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStory {
    /// Story title/summary
    pub title: Option<String>,
    /// Story description
    pub description: Option<String>,
    /// Acceptance criteria or requirements (extracted from description)
    pub requirements: Vec<String>,
    /// Story type (Story, Bug, Epic, Task, Feature, etc.)
    pub story_type: Option<String>,
    /// Priority mentioned in text
    pub priority: Option<String>,
    /// Labels/tags mentioned
    pub labels: Vec<String>,
    /// Project name mentioned
    pub project: Option<String>,
    /// Assignee mentioned
    pub assignee: Option<String>,
    /// Estimated story points or time
    pub estimate: Option<String>,
    /// Dependencies or linked issues
    pub dependencies: Vec<String>,
    /// Original full text (preserved for reference)
    pub original_text: String,
    /// Detected format/type (jira, github, plain, etc.)
    pub detected_format: Option<String>,
}

impl Default for ParsedStory {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            requirements: Vec::new(),
            story_type: None,
            priority: None,
            labels: Vec::new(),
            project: None,
            assignee: None,
            estimate: None,
            dependencies: Vec::new(),
            original_text: String::new(),
            detected_format: None,
        }
    }
}

/// Generic parser for story/description text from any source
pub struct StoryParser;

impl StoryParser {
    /// Parse story text and extract structured information
    /// Works with any format: Jira, GitHub issues, plain text, etc.
    pub fn parse(story_text: &str) -> ParsedStory {
        let mut parsed = ParsedStory {
            original_text: story_text.to_string(),
            ..Default::default()
        };

        if story_text.trim().is_empty() {
            return parsed;
        }

        // Detect format
        parsed.detected_format = Self::detect_format(story_text);

        let lines: Vec<&str> = story_text.lines().collect();
        
        // Try to detect if it's structured format (with labels/fields)
        if Self::is_structured_format(&lines) {
            Self::parse_structured(&lines, &mut parsed);
        } else {
            Self::parse_freeform(story_text, &mut parsed);
        }

        parsed
    }

    /// Detect the source format (Jira, GitHub, plain text, etc.)
    fn detect_format(text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        
        // Check for Jira patterns
        if lower.contains("jira") || text.contains("PROJ-") || text.contains("[PROJ-") {
            return Some("jira".to_string());
        }
        
        // Check for GitHub issue patterns
        if lower.contains("github") || text.contains("#") && text.contains("issue") {
            return Some("github".to_string());
        }
        
        // Check for markdown format
        if text.contains("```") || text.starts_with("#") {
            return Some("markdown".to_string());
        }
        
        // Check for structured format
        if text.contains(":") && text.lines().any(|l| l.contains(":")) {
            return Some("structured".to_string());
        }
        
        Some("plain".to_string())
    }

    /// Check if text appears to be in structured format (with labels/fields)
    fn is_structured_format(lines: &[&str]) -> bool {
        lines.iter().any(|line| {
            let lower = line.to_lowercase();
            lower.contains("summary:") || lower.contains("title:")
                || lower.contains("description:")
                || lower.contains("type:")
                || lower.contains("priority:")
                || lower.contains("labels:")
                || lower.contains("project:")
                || lower.contains("status:")
                || lower.contains("assignee:")
        })
    }

    /// Parse structured format (with explicit field labels)
    fn parse_structured(lines: &[&str], parsed: &mut ParsedStory) {
        let mut current_section: Option<String> = None;
        let mut description_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let lower = trimmed.to_lowercase();
            
            // Title/Summary
            if lower.starts_with("summary:") || lower.starts_with("title:") || lower.starts_with("subject:") {
                parsed.title = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Description
            else if lower.starts_with("description:") || lower.starts_with("body:") || lower.starts_with("details:") {
                current_section = Some("description".to_string());
                let desc = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                if !desc.is_empty() {
                    description_lines.push(desc.to_string());
                }
            } 
            // Type
            else if lower.starts_with("type:") || lower.starts_with("issue type:") || lower.starts_with("kind:") {
                parsed.story_type = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Priority
            else if lower.starts_with("priority:") || lower.starts_with("severity:") {
                parsed.priority = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Labels/Tags
            else if lower.starts_with("labels:") || lower.starts_with("tags:") || lower.starts_with("categories:") {
                let labels_str = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                parsed.labels = Self::parse_list(labels_str);
                current_section = None;
            } 
            // Project
            else if lower.starts_with("project:") || lower.starts_with("repo:") || lower.starts_with("repository:") {
                parsed.project = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Assignee
            else if lower.starts_with("assignee:") || lower.starts_with("assigned to:") || lower.starts_with("owner:") {
                parsed.assignee = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Estimate
            else if lower.starts_with("estimate:") || lower.starts_with("story points:") || lower.starts_with("effort:") {
                parsed.estimate = Some(trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string());
                current_section = None;
            } 
            // Status
            else if lower.starts_with("status:") || lower.starts_with("state:") {
                // Skip status, not relevant for task creation
                current_section = None;
            }
            // Description continuation
            else if current_section.as_deref() == Some("description") {
                description_lines.push(trimmed.to_string());
            } 
            // Requirements/Acceptance Criteria
            else if lower.contains("acceptance criteria") || lower.contains("requirements") || lower.contains("acceptance:") || lower.contains("definition of done") {
                current_section = Some("requirements".to_string());
                let criteria = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                if !criteria.is_empty() {
                    parsed.requirements.push(criteria.to_string());
                }
            } 
            // Requirements continuation
            else if current_section.as_deref() == Some("requirements") {
                // Check if it's a list item
                if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with("•") || trimmed.starts_with("▪") {
                    parsed.requirements.push(trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == '▪').trim().to_string());
                } else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    // Numbered list
                    parsed.requirements.push(trimmed.splitn(2, '.').nth(1).unwrap_or(trimmed).trim().to_string());
                } else if trimmed.to_lowercase().starts_with("given") 
                    || trimmed.to_lowercase().starts_with("when") 
                    || trimmed.to_lowercase().starts_with("then") {
                    parsed.requirements.push(trimmed.to_string());
                }
            } 
            // First line might be title if no label
            else if parsed.title.is_none() && !lower.contains(":") && trimmed.len() < 200 {
                parsed.title = Some(trimmed.to_string());
            }
        }

        if !description_lines.is_empty() {
            parsed.description = Some(description_lines.join("\n"));
        }
    }

    /// Parse freeform text (no explicit structure)
    fn parse_freeform(text: &str, parsed: &mut ParsedStory) {
        let lines: Vec<&str> = text.lines().collect();
        
        // First non-empty line is likely the title
        if let Some(first_line) = lines.iter().find(|l| !l.trim().is_empty()) {
            let trimmed = first_line.trim();
            // Skip markdown headers
            if !trimmed.starts_with('#') && trimmed.len() < 200 {
                parsed.title = Some(trimmed.trim_start_matches('#').trim().to_string());
            }
        }

        // Extract description (everything after title, before requirements)
        let mut description_start = 0;
        let mut requirements_start = None;

        for (i, line) in lines.iter().enumerate() {
            let lower = line.to_lowercase();
            let trimmed = line.trim();
            
            // Skip empty lines and headers
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if lower.contains("acceptance criteria") 
                || lower.contains("requirements") 
                || lower.contains("definition of done")
                || lower.contains("acceptance:")
                || lower.contains("what needs to be done")
                || lower.contains("tasks:")
                || lower.contains("checklist:") {
                requirements_start = Some(i);
                break;
            }
            if i > 0 && !trimmed.is_empty() {
                description_start = i;
            }
        }

        // Extract description
        let desc_lines: Vec<&str> = if let Some(req_start) = requirements_start {
            lines[description_start..req_start].iter()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .copied()
                .collect()
        } else {
            lines[description_start..].iter()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .copied()
                .collect()
        };

        if !desc_lines.is_empty() {
            parsed.description = Some(desc_lines.join("\n"));
        }

        // Extract requirements/acceptance criteria
        if let Some(req_start) = requirements_start {
            for line in &lines[req_start + 1..] {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                // Skip markdown headers
                if trimmed.starts_with('#') {
                    continue;
                }
                
                // List items
                if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with("•") || trimmed.starts_with("▪") {
                    parsed.requirements.push(
                        trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == '▪').trim().to_string()
                    );
                } 
                // Numbered list
                else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    parsed.requirements.push(
                        trimmed.splitn(2, '.').nth(1).unwrap_or(trimmed).trim().to_string()
                    );
                } 
                // Gherkin-style (Given/When/Then)
                else if trimmed.to_lowercase().starts_with("given") 
                    || trimmed.to_lowercase().starts_with("when") 
                    || trimmed.to_lowercase().starts_with("then") {
                    parsed.requirements.push(trimmed.to_string());
                }
                // Checkbox format
                else if trimmed.starts_with("[ ]") || trimmed.starts_with("[x]") || trimmed.starts_with("[X]") {
                    parsed.requirements.push(
                        trimmed.trim_start_matches("[ ]").trim_start_matches("[x]").trim_start_matches("[X]").trim().to_string()
                    );
                }
            }
        }

        // Extract metadata using patterns
        Self::extract_metadata(text, parsed);
    }

    /// Extract metadata from text using patterns
    fn extract_metadata(text: &str, parsed: &mut ParsedStory) {
        let lower = text.to_lowercase();

        // Extract priority
        if parsed.priority.is_none() {
            for priority in &["critical", "high", "medium", "low", "blocker", "trivial", "minor", "major", "urgent", "normal"] {
                if lower.contains(&format!("priority: {}", priority))
                    || lower.contains(&format!("priority {}", priority))
                    || (lower.contains(priority) && (lower.contains("priority") || lower.contains("severity"))) {
                    parsed.priority = Some(priority.to_string());
                    break;
                }
            }
        }

        // Extract story type
        if parsed.story_type.is_none() {
            for story_type in &["story", "bug", "epic", "task", "subtask", "feature", "improvement", "enhancement", "chore", "refactor"] {
                if lower.contains(story_type) {
                    parsed.story_type = Some(story_type.to_string());
                    break;
                }
            }
        }

        // Extract labels (common patterns)
        // Look for [label], #label, "label", or label: value patterns
        let label_patterns = [
            (r#"\[([^\]]+)\]"#, 1), // [label]
            (r"#(\w+)", 1),         // #label
            (r#""([^"]+)""#, 1),    // "label"
            (r"labels?:\s*([^\n]+)", 1), // labels: a, b, c
        ];

        for (pattern, group) in &label_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            for cap in re.captures_iter(text) {
                if let Some(label) = cap.get(*group) {
                    let label_str = label.as_str().trim().to_string();
                    if !label_str.is_empty() && !parsed.labels.contains(&label_str) {
                        // If it's a list, split it
                        if label_str.contains(',') || label_str.contains(';') {
                            parsed.labels.extend(Self::parse_list(&label_str));
                        } else {
                            parsed.labels.push(label_str);
                        }
                    }
                }
            }
        }

        // Extract project (look for project names, project codes, or repository names)
        let project_patterns = [
            (r"(?i)(?:project|proj|repo|repository)[\s:]*([A-Z][A-Z0-9-]+)", 1), // PROJECT-123, Project Name
            (r"(?i)(?:project|proj|repo|repository)[\s:]*([\w\s]+)", 1),        // Project Name
            (r"([A-Z]{2,}-\d+)", 1), // ISSUE-123 format
        ];

        if parsed.project.is_none() {
            for (pattern, group) in &project_patterns {
                let re = regex::Regex::new(pattern).unwrap();
                if let Some(cap) = re.captures(text) {
                    if let Some(project) = cap.get(*group) {
                        let project_str = project.as_str().trim().to_string();
                        if !project_str.is_empty() && project_str.len() < 100 {
                            parsed.project = Some(project_str);
                            break;
                        }
                    }
                }
            }
        }

        // Extract assignee
        if parsed.assignee.is_none() {
            let assignee_re = regex::Regex::new(r"(?i)(?:assignee|assigned to|owner|author)[\s:]*([A-Za-z][A-Za-z0-9._-]+)").unwrap();
            if let Some(cap) = assignee_re.captures(text) {
                if let Some(assignee) = cap.get(1) {
                    parsed.assignee = Some(assignee.as_str().trim().to_string());
                }
            }
        }

        // Extract estimate/story points
        if parsed.estimate.is_none() {
            let estimate_re = regex::Regex::new(r"(?i)(?:estimate|story points?|sp|points?|effort|hours?)[\s:]*(\d+)").unwrap();
            if let Some(cap) = estimate_re.captures(text) {
                if let Some(est) = cap.get(1) {
                    parsed.estimate = Some(est.as_str().to_string());
                }
            }
        }

        // Extract dependencies (linked issues)
        let dep_re = regex::Regex::new(r"(?i)(?:depends on|blocks?|relates to|linked to|references?)[\s:]*([A-Z]+-\d+|\d+)").unwrap();
        for cap in dep_re.captures_iter(text) {
            if let Some(dep) = cap.get(1) {
                parsed.dependencies.push(dep.as_str().to_string());
            }
        }
    }

    /// Parse a comma/semicolon/pipe-separated list
    fn parse_list(list_str: &str) -> Vec<String> {
        list_str
            .split(&[',', ';', '|', '\n'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_structured_format() {
        let text = r#"
Title: Implement user authentication
Description: Add login and registration functionality
Type: Story
Priority: High
Labels: backend, security, api
Project: Portal
        "#;
        
        let parsed = StoryParser::parse(text);
        assert_eq!(parsed.title, Some("Implement user authentication".to_string()));
        assert_eq!(parsed.story_type, Some("Story".to_string()));
        assert_eq!(parsed.priority, Some("High".to_string()));
        assert!(parsed.labels.contains(&"backend".to_string()));
    }

    #[test]
    fn test_parse_freeform() {
        let text = r#"
Implement user authentication

This story is about adding login and registration functionality to the portal application.

Acceptance Criteria:
- User can register with email and password
- User can login with credentials
- Password is hashed securely
        "#;
        
        let parsed = StoryParser::parse(text);
        assert!(parsed.title.is_some());
        assert!(parsed.description.is_some());
        assert_eq!(parsed.requirements.len(), 3);
    }

    #[test]
    fn test_parse_github_style() {
        let text = r#"
## Add user authentication feature

We need to implement login and registration.

### Requirements
1. User registration
2. User login
3. Password reset

**Labels:** backend, security
**Project:** portal-app
        "#;
        
        let parsed = StoryParser::parse(text);
        assert!(parsed.title.is_some());
        assert!(parsed.requirements.len() >= 3);
    }
}

