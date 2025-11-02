use crate::domains::ide::entities::{IdeConfig, FrameworkIdeMapping, SuggestedFramework, FrameworkGroup};
use crate::domains::ide::services::ide_service::IdeService;
use crate::domains::ide::repositories::framework_repository::FrameworkRepository;
use crate::domains::ide::repositories::ide_repository::IdeRepository;
use crate::domains::ide::repositories::framework_ide_mapping_repository::FrameworkIdeMappingRepository;
use crate::database::DatabaseManager;
use std::sync::Arc;
use tauri::{Manager, State};

// Initialize IDE service in app state
pub fn init_ide_storage<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    app.manage(Arc::new(IdeService::new()));
}

#[tauri::command]
pub async fn detect_installed_ides(
    ide_service: State<'_, Arc<IdeService>>,
) -> Result<Vec<String>, String> {
    Ok(ide_service.detect_installed_ides())
}

#[tauri::command]
pub async fn get_all_ides(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<IdeConfig>, String> {
    let conn = db.get_connection_clone();
    let ide_models = IdeRepository::get_all(&conn)
        .await
        .map_err(|e| format!("Failed to get IDEs: {}", e))?;
    
    Ok(ide_models.into_iter().map(|ide| IdeConfig {
        id: Some(ide.id as i64),
        name: ide.name,
        executable: ide.executable,
        is_default: ide.is_default,
        created_at: ide.created_at.map(|dt| dt.to_rfc3339()),
        updated_at: ide.updated_at.map(|dt| dt.to_rfc3339()),
    }).collect())
}

#[tauri::command]
pub async fn add_ide(
    name: String,
    executable: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    let ide = IdeRepository::create(&conn, name, executable, false)
        .await
        .map_err(|e| format!("Failed to add IDE: {}", e))?;
    
    Ok(ide.id as i64)
}

#[tauri::command]
pub async fn update_ide(
    id: i64,
    name: String,
    executable: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    IdeRepository::update(&conn, id as i32, Some(name), Some(executable), None)
        .await
        .map_err(|e| format!("Failed to update IDE: {}", e))?;
    
    Ok(id)
}

#[tauri::command]
pub async fn delete_ide(
    id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    
    // Framework IDE mappings will be automatically deleted via foreign key cascade
    IdeRepository::delete(&conn, id as i32)
        .await
        .map_err(|e| format!("Failed to delete IDE: {}", e))?;
    
    Ok(id)
}

#[tauri::command]
pub async fn set_default_ide(
    id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    IdeRepository::update(&conn, id as i32, None, None, Some(true))
        .await
        .map_err(|e| format!("Failed to set default IDE: {}", e))?;
    
    Ok(id)
}

#[tauri::command]
pub async fn get_default_ide(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Option<IdeConfig>, String> {
    let conn = db.get_connection_clone();
    let ide = IdeRepository::get_default(&conn)
        .await
        .map_err(|e| format!("Failed to get default IDE: {}", e))?;
    
    Ok(ide.map(|ide| IdeConfig {
        id: Some(ide.id as i64),
        name: ide.name,
        executable: ide.executable,
        is_default: ide.is_default,
        created_at: ide.created_at.map(|dt| dt.to_rfc3339()),
        updated_at: ide.updated_at.map(|dt| dt.to_rfc3339()),
    }))
}

#[tauri::command]
pub async fn get_all_framework_ide_mappings(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<FrameworkIdeMapping>, String> {
    let conn = db.get_connection_clone();
    let mappings = FrameworkIdeMappingRepository::get_all(&conn)
        .await
        .map_err(|e| format!("Failed to get framework IDE mappings: {}", e))?;
    
    Ok(mappings.into_iter().map(|mapping| FrameworkIdeMapping {
        id: Some(mapping.id as i64),
        framework: mapping.framework,
        ide_id: mapping.ide_id as i64,
        created_at: mapping.created_at.map(|dt| dt.to_rfc3339()),
        updated_at: mapping.updated_at.map(|dt| dt.to_rfc3339()),
    }).collect())
}

#[tauri::command]
pub async fn set_framework_ide_mapping(
    framework: String,
    ide_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    
    // Verify IDE exists
    let _ide = IdeRepository::get_by_id(&conn, ide_id as i32)
        .await
        .map_err(|e| format!("Failed to verify IDE: {}", e))?
        .ok_or_else(|| format!("IDE with id {} not found", ide_id))?;
    
    let mapping = FrameworkIdeMappingRepository::create_or_update(&conn, framework, ide_id as i32)
        .await
        .map_err(|e| format!("Failed to set framework IDE mapping: {}", e))?;
    
    Ok(mapping.id as i64)
}

#[tauri::command]
pub async fn get_framework_ide_mapping(
    framework: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Option<IdeConfig>, String> {
    let conn = db.get_connection_clone();
    let mapping = FrameworkIdeMappingRepository::get_by_framework(&conn, &framework)
        .await
        .map_err(|e| format!("Failed to get framework IDE mapping: {}", e))?;
    
    if let Some(mapping) = mapping {
        let ide = IdeRepository::get_by_id(&conn, mapping.ide_id)
            .await
            .map_err(|e| format!("Failed to get IDE: {}", e))?
            .ok_or_else(|| "IDE not found".to_string())?;
        
        Ok(Some(IdeConfig {
            id: Some(ide.id as i64),
            name: ide.name,
            executable: ide.executable,
            is_default: ide.is_default,
            created_at: ide.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: ide.updated_at.map(|dt| dt.to_rfc3339()),
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn delete_framework_ide_mapping(
    framework: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<i64, String> {
    let conn = db.get_connection_clone();
    
    // Get the mapping first to return its ID
    let mapping = FrameworkIdeMappingRepository::get_by_framework(&conn, &framework)
        .await
        .map_err(|e| format!("Failed to get framework IDE mapping: {}", e))?
        .ok_or_else(|| format!("Mapping for framework {} not found", framework))?;
    
    let id = mapping.id as i64;
    FrameworkIdeMappingRepository::delete(&conn, &framework)
        .await
        .map_err(|e| format!("Failed to delete framework IDE mapping: {}", e))?;
    
    Ok(id)
}

#[tauri::command]
pub async fn get_suggested_frameworks() -> Result<Vec<FrameworkGroup>, String> {
    let groups = vec![
        FrameworkGroup {
            category: "Frontend".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "React".to_string(), icon: "logos:react".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Vue".to_string(), icon: "logos:vue".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Angular".to_string(), icon: "logos:angular-icon".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Svelte".to_string(), icon: "logos:svelte-icon".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Next.js".to_string(), icon: "logos:nextjs-icon".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Nuxt".to_string(), icon: "logos:nuxt-icon".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Remix".to_string(), icon: "logos:remix-icon".to_string(), category: "Frontend".to_string() },
                SuggestedFramework { name: "Gatsby".to_string(), icon: "logos:gatsby".to_string(), category: "Frontend".to_string() },
            ],
        },
        FrameworkGroup {
            category: "Backend".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "Node.js".to_string(), icon: "logos:nodejs-icon".to_string(), category: "Backend".to_string() },
                SuggestedFramework { name: "Express".to_string(), icon: "logos:express".to_string(), category: "Backend".to_string() },
                SuggestedFramework { name: "Fastify".to_string(), icon: "logos:fastify-icon".to_string(), category: "Backend".to_string() },
                SuggestedFramework { name: "NestJS".to_string(), icon: "logos:nestjs".to_string(), category: "Backend".to_string() },
            ],
        },
        FrameworkGroup {
            category: "Python".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "Python".to_string(), icon: "logos:python".to_string(), category: "Python".to_string() },
                SuggestedFramework { name: "Django".to_string(), icon: "logos:django-icon".to_string(), category: "Python".to_string() },
                SuggestedFramework { name: "Flask".to_string(), icon: "logos:flask".to_string(), category: "Python".to_string() },
                SuggestedFramework { name: "FastAPI".to_string(), icon: "logos:fastapi-icon".to_string(), category: "Python".to_string() },
            ],
        },
        FrameworkGroup {
            category: "Ruby".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "Ruby".to_string(), icon: "logos:ruby".to_string(), category: "Ruby".to_string() },
                SuggestedFramework { name: "Rails".to_string(), icon: "logos:rails".to_string(), category: "Ruby".to_string() },
                SuggestedFramework { name: "Sinatra".to_string(), icon: "logos:sinatra".to_string(), category: "Ruby".to_string() },
            ],
        },
        FrameworkGroup {
            category: "PHP".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "PHP".to_string(), icon: "logos:php".to_string(), category: "PHP".to_string() },
                SuggestedFramework { name: "Laravel".to_string(), icon: "logos:laravel".to_string(), category: "PHP".to_string() },
                SuggestedFramework { name: "Symfony".to_string(), icon: "logos:symfony".to_string(), category: "PHP".to_string() },
            ],
        },
        FrameworkGroup {
            category: "Java".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "Java".to_string(), icon: "logos:java".to_string(), category: "Java".to_string() },
                SuggestedFramework { name: "Spring".to_string(), icon: "logos:spring-icon".to_string(), category: "Java".to_string() },
                SuggestedFramework { name: "Quarkus".to_string(), icon: "logos:quarkus-icon".to_string(), category: "Java".to_string() },
            ],
        },
        FrameworkGroup {
            category: "Other".to_string(),
            frameworks: vec![
                SuggestedFramework { name: "Go".to_string(), icon: "logos:go".to_string(), category: "Other".to_string() },
                SuggestedFramework { name: "Rust".to_string(), icon: "logos:rust".to_string(), category: "Other".to_string() },
                SuggestedFramework { name: "Elixir".to_string(), icon: "logos:elixir".to_string(), category: "Other".to_string() },
                SuggestedFramework { name: "Phoenix".to_string(), icon: "logos:phoenix".to_string(), category: "Other".to_string() },
            ],
        },
    ];
    
    Ok(groups)
}

#[tauri::command]
pub async fn get_all_frameworks(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<crate::entities::framework::Model>, String> {
    let conn = db.get_connection_clone();
    FrameworkRepository::get_all(&conn)
        .await
        .map_err(|e| format!("Failed to get frameworks: {}", e))
}

#[tauri::command]
pub async fn create_framework(
    name: String,
    icon: String,
    icon_type: String, // 'devicon' or 'file'
    category: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::framework::Model, String> {
    let conn = db.get_connection_clone();
    
    // Check if framework with this name already exists
    if let Ok(Some(_)) = FrameworkRepository::get_by_name(&conn, &name).await {
        return Err(format!("Framework with name '{}' already exists", name));
    }
    
    FrameworkRepository::create(&conn, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to create framework: {}", e))
}

#[tauri::command]
pub async fn update_framework(
    id: i32,
    name: Option<String>,
    icon: Option<String>,
    icon_type: Option<String>,
    category: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<crate::entities::framework::Model, String> {
    let conn = db.get_connection_clone();
    
    // If updating name, check if another framework with that name exists
    if let Some(ref new_name) = name {
        if let Ok(Some(existing)) = FrameworkRepository::get_by_name(&conn, new_name).await {
            if existing.id != id {
                return Err(format!("Framework with name '{}' already exists", new_name));
            }
        }
    }
    
    FrameworkRepository::update(&conn, id, name, icon, icon_type, category)
        .await
        .map_err(|e| format!("Failed to update framework: {}", e))
}

#[tauri::command]
pub async fn delete_framework(
    id: i32,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let conn = db.get_connection_clone();
    FrameworkRepository::delete(&conn, id)
        .await
        .map_err(|e| format!("Failed to delete framework: {}", e))
}

