use crate::database::DatabaseManager;
use crate::domains::languages::entities::{LanguageGroup, SuggestedLanguage};
use crate::domains::languages::repositories::LanguageRepository;
use crate::domains::learning::repositories::learned_pattern_repository::LearnedPatternRepository;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_suggested_languages(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<LanguageGroup>, String> {
    // Get learned language patterns
    let db = db_manager.get_connection();
    let learned_patterns = LearnedPatternRepository::get_by_type(db, "language")
        .await
        .unwrap_or_else(|_| Vec::new());

    // Extract unique language names from learned patterns (sorted by frequency)
    let mut learned_languages: Vec<(String, i32)> = Vec::new();
    for pattern in &learned_patterns {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&pattern.pattern_data) {
            if let Some(lang) = data.get("language").and_then(|v| v.as_str()) {
                // Check if we already have this language
                if let Some((_, freq)) = learned_languages.iter_mut().find(|(name, _)| name == lang)
                {
                    *freq += pattern.frequency;
                } else {
                    learned_languages.push((lang.to_string(), pattern.frequency));
                }
            }
        }
    }

    // Sort by frequency (highest first)
    learned_languages.sort_by(|a, b| b.1.cmp(&a.1));

    let groups = vec![
        LanguageGroup {
            category: "Web".to_string(),
            languages: vec![
                SuggestedLanguage {
                    name: "JavaScript".to_string(),
                    icon: "logos:javascript".to_string(),
                    category: "Web".to_string(),
                },
                SuggestedLanguage {
                    name: "TypeScript".to_string(),
                    icon: "logos:typescript-icon".to_string(),
                    category: "Web".to_string(),
                },
                SuggestedLanguage {
                    name: "HTML".to_string(),
                    icon: "logos:html-5".to_string(),
                    category: "Web".to_string(),
                },
                SuggestedLanguage {
                    name: "CSS".to_string(),
                    icon: "logos:css-3".to_string(),
                    category: "Web".to_string(),
                },
            ],
        },
        LanguageGroup {
            category: "Backend".to_string(),
            languages: vec![
                SuggestedLanguage {
                    name: "Python".to_string(),
                    icon: "logos:python".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "Rust".to_string(),
                    icon: "logos:rust".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "Go".to_string(),
                    icon: "logos:go".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "Java".to_string(),
                    icon: "logos:java".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "C#".to_string(),
                    icon: "logos:c-sharp".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "PHP".to_string(),
                    icon: "logos:php".to_string(),
                    category: "Backend".to_string(),
                },
                SuggestedLanguage {
                    name: "Ruby".to_string(),
                    icon: "logos:ruby".to_string(),
                    category: "Backend".to_string(),
                },
            ],
        },
        LanguageGroup {
            category: "Mobile".to_string(),
            languages: vec![
                SuggestedLanguage {
                    name: "Swift".to_string(),
                    icon: "logos:swift".to_string(),
                    category: "Mobile".to_string(),
                },
                SuggestedLanguage {
                    name: "Kotlin".to_string(),
                    icon: "logos:kotlin-icon".to_string(),
                    category: "Mobile".to_string(),
                },
                SuggestedLanguage {
                    name: "Dart".to_string(),
                    icon: "logos:dart".to_string(),
                    category: "Mobile".to_string(),
                },
            ],
        },
        LanguageGroup {
            category: "Systems".to_string(),
            languages: vec![
                SuggestedLanguage {
                    name: "C".to_string(),
                    icon: "logos:c".to_string(),
                    category: "Systems".to_string(),
                },
                SuggestedLanguage {
                    name: "C++".to_string(),
                    icon: "logos:c-plusplus".to_string(),
                    category: "Systems".to_string(),
                },
                SuggestedLanguage {
                    name: "Rust".to_string(),
                    icon: "logos:rust".to_string(),
                    category: "Systems".to_string(),
                },
            ],
        },
        LanguageGroup {
            category: "Functional".to_string(),
            languages: vec![
                SuggestedLanguage {
                    name: "Haskell".to_string(),
                    icon: "logos:haskell-icon".to_string(),
                    category: "Functional".to_string(),
                },
                SuggestedLanguage {
                    name: "Elixir".to_string(),
                    icon: "logos:elixir".to_string(),
                    category: "Functional".to_string(),
                },
                SuggestedLanguage {
                    name: "F#".to_string(),
                    icon: "logos:f-sharp".to_string(),
                    category: "Functional".to_string(),
                },
            ],
        },
    ];

    // Add a "Recommended" group if we have learned languages
    if !learned_languages.is_empty() {
        let recommended_languages: Vec<SuggestedLanguage> = learned_languages
            .iter()
            .take(10) // Top 10 most frequently used
            .map(|(name, _)| {
                // Try to find matching icon from existing groups
                let icon = groups
                    .iter()
                    .flat_map(|g| &g.languages)
                    .find(|lang| lang.name.eq_ignore_ascii_case(name))
                    .map(|lang| lang.icon.clone())
                    .unwrap_or_else(|| format!("logos:{}", name.to_lowercase().replace(" ", "-")));

                SuggestedLanguage {
                    name: name.clone(),
                    icon,
                    category: "Recommended".to_string(),
                }
            })
            .collect();

        if !recommended_languages.is_empty() {
            let mut groups_with_recommended = groups;
            groups_with_recommended.insert(
                0,
                LanguageGroup {
                    category: "Recommended (Based on Your Usage)".to_string(),
                    languages: recommended_languages,
                },
            );
            return Ok(groups_with_recommended);
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn get_all_languages(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<crate::entities::language::Model>, String> {
    let conn = db.get_connection_clone();
    LanguageRepository::get_all(&conn)
        .await
        .map_err(|e| format!("Failed to get languages: {}", e))
}

#[tauri::command]
pub async fn create_language(
    name: String,
    icon: String,
    icon_type: String, // 'devicon' or 'file' - parameter name matches frontend camelCase 'iconType' via Tauri conversion
    category: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::language::Model, String> {
    let conn = db.get_connection_clone();

    // Check if language with this name already exists
    if let Ok(Some(_)) = LanguageRepository::get_by_name(&conn, &name).await {
        return Err(format!("Language with name '{}' already exists", name));
    }

    LanguageRepository::create(&conn, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to create language: {}", e))
}

#[tauri::command]
pub async fn update_language(
    id: i32,
    name: Option<String>,
    icon: Option<String>,
    icon_type: Option<String>,
    category: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::language::Model, String> {
    let conn = db.get_connection_clone();

    // If updating name, check if another language with that name exists
    if let Some(ref new_name) = name {
        if let Ok(Some(existing)) = LanguageRepository::get_by_name(&conn, new_name).await {
            if existing.id != id {
                return Err(format!("Language with name '{}' already exists", new_name));
            }
        }
    }

    LanguageRepository::update(&conn, id, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to update language: {}", e))
}

#[tauri::command]
pub async fn delete_language(id: i32, db: State<'_, Arc<DatabaseManager>>) -> Result<(), String> {
    let conn = db.get_connection_clone();
    LanguageRepository::delete(&conn, id)
        .await
        .map_err(|e| format!("Failed to delete language: {}", e))
}
