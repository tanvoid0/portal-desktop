use serde::{Deserialize, Serialize};

/// Resource types that can be linked to tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "deployment")]
    Deployment,
    #[serde(rename = "pipeline")]
    Pipeline,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Document => "document",
            ResourceType::Project => "project",
            ResourceType::Deployment => "deployment",
            ResourceType::Pipeline => "pipeline",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "document" => Some(ResourceType::Document),
            "project" => Some(ResourceType::Project),
            "deployment" => Some(ResourceType::Deployment),
            "pipeline" => Some(ResourceType::Pipeline),
            _ => None,
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

