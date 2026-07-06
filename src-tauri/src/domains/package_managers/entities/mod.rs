use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedPackageManager {
    pub name: String,
    pub icon: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerGroup {
    pub category: String,
    pub package_managers: Vec<SuggestedPackageManager>,
}
