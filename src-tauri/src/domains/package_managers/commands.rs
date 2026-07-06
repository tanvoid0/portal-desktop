use crate::database::DatabaseManager;
use crate::domains::learning::repositories::learned_pattern_repository::LearnedPatternRepository;
use crate::domains::package_managers::entities::{PackageManagerGroup, SuggestedPackageManager};
use crate::domains::package_managers::repositories::PackageManagerRepository;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_suggested_package_managers(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<PackageManagerGroup>, String> {
    // Get learned package manager patterns
    let db = db_manager.get_connection();
    let learned_patterns = LearnedPatternRepository::get_by_type(db, "package_manager")
        .await
        .unwrap_or_else(|_| Vec::new());

    // Extract unique package manager names from learned patterns (sorted by frequency)
    let mut learned_package_managers: Vec<(String, i32)> = Vec::new();
    for pattern in &learned_patterns {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&pattern.pattern_data) {
            if let Some(pm) = data.get("package_manager").and_then(|v| v.as_str()) {
                // Check if we already have this package manager
                if let Some((_, freq)) = learned_package_managers
                    .iter_mut()
                    .find(|(name, _)| name == pm)
                {
                    *freq += pattern.frequency;
                } else {
                    learned_package_managers.push((pm.to_string(), pattern.frequency));
                }
            }
        }
    }

    // Sort by frequency (highest first)
    learned_package_managers.sort_by(|a, b| b.1.cmp(&a.1));

    let groups = vec![
        PackageManagerGroup {
            category: "JavaScript/TypeScript".to_string(),
            package_managers: vec![
                SuggestedPackageManager {
                    name: "npm".to_string(),
                    icon: "logos:npm-icon".to_string(),
                    category: "JavaScript/TypeScript".to_string(),
                },
                SuggestedPackageManager {
                    name: "yarn".to_string(),
                    icon: "logos:yarn".to_string(),
                    category: "JavaScript/TypeScript".to_string(),
                },
                SuggestedPackageManager {
                    name: "pnpm".to_string(),
                    icon: "logos:pnpm".to_string(),
                    category: "JavaScript/TypeScript".to_string(),
                },
                SuggestedPackageManager {
                    name: "bun".to_string(),
                    icon: "logos:bun".to_string(),
                    category: "JavaScript/TypeScript".to_string(),
                },
            ],
        },
        PackageManagerGroup {
            category: "Python".to_string(),
            package_managers: vec![
                SuggestedPackageManager {
                    name: "pip".to_string(),
                    icon: "logos:python".to_string(),
                    category: "Python".to_string(),
                },
                SuggestedPackageManager {
                    name: "pipenv".to_string(),
                    icon: "logos:python".to_string(),
                    category: "Python".to_string(),
                },
                SuggestedPackageManager {
                    name: "poetry".to_string(),
                    icon: "logos:poetry".to_string(),
                    category: "Python".to_string(),
                },
                SuggestedPackageManager {
                    name: "conda".to_string(),
                    icon: "logos:conda".to_string(),
                    category: "Python".to_string(),
                },
            ],
        },
        PackageManagerGroup {
            category: "Rust".to_string(),
            package_managers: vec![SuggestedPackageManager {
                name: "cargo".to_string(),
                icon: "logos:rust".to_string(),
                category: "Rust".to_string(),
            }],
        },
        PackageManagerGroup {
            category: "Go".to_string(),
            package_managers: vec![SuggestedPackageManager {
                name: "go mod".to_string(),
                icon: "logos:go".to_string(),
                category: "Go".to_string(),
            }],
        },
        PackageManagerGroup {
            category: "Java".to_string(),
            package_managers: vec![
                SuggestedPackageManager {
                    name: "maven".to_string(),
                    icon: "logos:maven".to_string(),
                    category: "Java".to_string(),
                },
                SuggestedPackageManager {
                    name: "gradle".to_string(),
                    icon: "logos:gradle".to_string(),
                    category: "Java".to_string(),
                },
            ],
        },
        PackageManagerGroup {
            category: "PHP".to_string(),
            package_managers: vec![SuggestedPackageManager {
                name: "composer".to_string(),
                icon: "logos:composer".to_string(),
                category: "PHP".to_string(),
            }],
        },
        PackageManagerGroup {
            category: "Ruby".to_string(),
            package_managers: vec![
                SuggestedPackageManager {
                    name: "bundler".to_string(),
                    icon: "logos:ruby".to_string(),
                    category: "Ruby".to_string(),
                },
                SuggestedPackageManager {
                    name: "gem".to_string(),
                    icon: "logos:ruby".to_string(),
                    category: "Ruby".to_string(),
                },
            ],
        },
        PackageManagerGroup {
            category: ".NET".to_string(),
            package_managers: vec![
                SuggestedPackageManager {
                    name: "nuget".to_string(),
                    icon: "logos:nuget-icon".to_string(),
                    category: ".NET".to_string(),
                },
                SuggestedPackageManager {
                    name: "dotnet".to_string(),
                    icon: "logos:dotnet".to_string(),
                    category: ".NET".to_string(),
                },
            ],
        },
    ];

    // Add a "Recommended" group if we have learned package managers
    if !learned_package_managers.is_empty() {
        let recommended_package_managers: Vec<SuggestedPackageManager> = learned_package_managers
            .iter()
            .take(10) // Top 10 most frequently used
            .map(|(name, _)| {
                // Try to find matching icon from existing groups
                let icon = groups
                    .iter()
                    .flat_map(|g| &g.package_managers)
                    .find(|pm| pm.name.eq_ignore_ascii_case(name))
                    .map(|pm| pm.icon.clone())
                    .unwrap_or_else(|| format!("logos:{}", name.to_lowercase().replace(" ", "-")));

                SuggestedPackageManager {
                    name: name.clone(),
                    icon,
                    category: "Recommended".to_string(),
                }
            })
            .collect();

        if !recommended_package_managers.is_empty() {
            let mut groups_with_recommended = groups;
            groups_with_recommended.insert(
                0,
                PackageManagerGroup {
                    category: "Recommended (Based on Your Usage)".to_string(),
                    package_managers: recommended_package_managers,
                },
            );
            return Ok(groups_with_recommended);
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn get_all_package_managers(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<crate::entities::package_manager::Model>, String> {
    let conn = db.get_connection_clone();
    PackageManagerRepository::get_all(&conn)
        .await
        .map_err(|e| format!("Failed to get package managers: {}", e))
}

#[tauri::command]
pub async fn create_package_manager(
    name: String,
    icon: String,
    icon_type: String, // 'devicon' or 'file'
    category: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::package_manager::Model, String> {
    let conn = db.get_connection_clone();

    // Check if package manager with this name already exists
    if let Ok(Some(_)) = PackageManagerRepository::get_by_name(&conn, &name).await {
        return Err(format!(
            "Package manager with name '{}' already exists",
            name
        ));
    }

    PackageManagerRepository::create(&conn, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to create package manager: {}", e))
}

#[tauri::command]
pub async fn update_package_manager(
    id: i32,
    name: Option<String>,
    icon: Option<String>,
    icon_type: Option<String>,
    category: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::package_manager::Model, String> {
    let conn = db.get_connection_clone();

    // If updating name, check if another package manager with that name exists
    if let Some(ref new_name) = name {
        if let Ok(Some(existing)) = PackageManagerRepository::get_by_name(&conn, new_name).await {
            if existing.id != id {
                return Err(format!(
                    "Package manager with name '{}' already exists",
                    new_name
                ));
            }
        }
    }

    PackageManagerRepository::update(&conn, id, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to update package manager: {}", e))
}

#[tauri::command]
pub async fn delete_package_manager(
    id: i32,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let conn = db.get_connection_clone();
    PackageManagerRepository::delete(&conn, id)
        .await
        .map_err(|e| format!("Failed to delete package manager: {}", e))
}
