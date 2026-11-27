use crate::domains::ai::platform_daemon::write_atomic;
use crate::domains::ai::providers::{ProviderConfig, ProviderType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// AI Settings — agent-platform only.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AISettings {
    pub providers: HashMap<String, ProviderConfig>,
    pub default_provider: Option<String>,
}

impl Default for AISettings {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "AgentPlatform".to_string(),
            ProviderConfig::default_for_type(ProviderType::AgentPlatform),
        );
        Self {
            providers,
            default_provider: Some("AgentPlatform".to_string()),
        }
    }
}

pub struct AISettingsService {
    settings_path: PathBuf,
}

impl AISettingsService {
    pub fn new() -> Self {
        let mut settings_path = crate::app_paths::config_dir();
        settings_path.push("ai-settings.json");
        Self { settings_path }
    }

    /// Load settings, migrating away from legacy Ollama/Gemini direct providers.
    pub fn load_settings(&self) -> Result<AISettings, String> {
        if !self.settings_path.exists() {
            return Ok(AISettings::default());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| format!("Failed to read AI settings file: {}", e))?;

        let mut json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse AI settings JSON: {}", e))?;

        let legacy_keys = ["Ollama", "Gemini", "OpenAI", "Anthropic"];
        let mut migrated_model: Option<String> = None;
        let mut migrated_enabled = false;
        // A plain read must not rewrite the file; only a migration does.
        let mut migrated = false;

        if let Some(Value::Object(providers_obj)) = json.get_mut("providers") {
            for key in legacy_keys {
                if let Some(legacy) = providers_obj.remove(key) {
                    migrated = true;
                    if let Some(obj) = legacy.as_object() {
                        if migrated_model.is_none() {
                            migrated_model = obj
                                .get("model")
                                .and_then(Value::as_str)
                                .map(str::to_string)
                                .filter(|m| !m.is_empty());
                        }
                        if obj.get("enabled").and_then(Value::as_bool) == Some(true) {
                            migrated_enabled = true;
                        }
                    }
                }
            }
            let before = providers_obj.len();
            providers_obj.retain(|key, _| key == "AgentPlatform");
            migrated |= providers_obj.len() != before;
        }

        if let Some(obj) = json.as_object_mut() {
            migrated |=
                obj.get("default_provider").and_then(Value::as_str) != Some("AgentPlatform");
            obj.insert(
                "default_provider".to_string(),
                Value::String("AgentPlatform".to_string()),
            );
        }

        let mut settings: AISettings = serde_json::from_value(json)
            .map_err(|e| format!("Failed to parse AI settings: {}", e))?;

        if !settings.providers.contains_key("AgentPlatform") {
            migrated = true;
            settings.providers.insert(
                "AgentPlatform".to_string(),
                ProviderConfig::default_for_type(ProviderType::AgentPlatform),
            );
        }

        if let Some(platform) = settings.providers.get_mut("AgentPlatform") {
            if let Some(model) = migrated_model {
                platform.model = model;
            }
            if migrated_enabled {
                platform.enabled = true;
            }
        }

        settings.default_provider = Some("AgentPlatform".to_string());
        if migrated {
            self.save_settings(&settings)?;
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AISettings) -> Result<(), String> {
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize AI settings: {}", e))?;
        write_atomic(&self.settings_path, &content)
            .map_err(|e| format!("Failed to write AI settings file: {}", e))?;
        Ok(())
    }

    pub fn get_provider_config(
        &self,
        provider_type: ProviderType,
    ) -> Result<ProviderConfig, String> {
        if provider_type != ProviderType::AgentPlatform {
            return Err("Only AgentPlatform is supported".to_string());
        }
        let settings = self.load_settings()?;
        settings
            .providers
            .get("AgentPlatform")
            .cloned()
            .ok_or_else(|| "AgentPlatform not found in settings".to_string())
    }

    pub fn save_provider_config(&self, config: ProviderConfig) -> Result<(), String> {
        if config.provider_type != ProviderType::AgentPlatform {
            return Err("Only AgentPlatform is supported".to_string());
        }
        let mut settings = self.load_settings()?;
        settings
            .providers
            .insert("AgentPlatform".to_string(), config);
        self.save_settings(&settings)
    }

    pub fn get_all_providers(&self) -> Result<Vec<ProviderConfig>, String> {
        let settings = self.load_settings()?;
        Ok(settings.providers.values().cloned().collect())
    }

    pub fn get_default_provider(&self) -> Result<Option<ProviderType>, String> {
        let settings = self.load_settings()?;
        if settings.providers.contains_key("AgentPlatform") {
            Ok(Some(ProviderType::AgentPlatform))
        } else {
            Ok(None)
        }
    }

    pub fn set_default_provider(&self, provider_type: ProviderType) -> Result<(), String> {
        if provider_type != ProviderType::AgentPlatform {
            return Err("Only AgentPlatform is supported".to_string());
        }
        let mut settings = self.load_settings()?;
        settings.default_provider = Some("AgentPlatform".to_string());
        self.save_settings(&settings)
    }

    pub fn delete_provider_config(&self, _provider_type: ProviderType) -> Result<(), String> {
        Err("Cannot delete the agent-platform provider".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(dir: &tempfile::TempDir) -> AISettingsService {
        AISettingsService {
            settings_path: dir.path().join("ai-settings.json"),
        }
    }

    #[test]
    fn reading_does_not_rewrite_but_migrating_does() {
        let dir = tempfile::tempdir().unwrap();
        let s = service(&dir);
        // Compact form: any rewrite would pretty-print it.
        let compact = serde_json::to_string(&AISettings::default()).unwrap();
        fs::write(&s.settings_path, &compact).unwrap();
        s.load_settings().unwrap();
        assert_eq!(fs::read_to_string(&s.settings_path).unwrap(), compact);

        fs::write(
            &s.settings_path,
            r#"{"providers":{"Ollama":{"model":"m"}}}"#,
        )
        .unwrap();
        s.load_settings().unwrap();
        let saved = fs::read_to_string(&s.settings_path).unwrap();
        assert!(!saved.contains("Ollama"), "{saved}");
    }

    #[test]
    fn reads_never_see_a_half_written_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = service(&dir);
        s.save_settings(&AISettings::default()).unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..200 {
                    // Windows may refuse a replace while a reader holds the file;
                    // the old contents stay, which is all this test cares about.
                    let _ = s.save_settings(&AISettings::default());
                }
            });
            for _ in 0..500 {
                if let Err(e) = s.load_settings() {
                    assert!(!e.contains("parse"), "{e}");
                }
            }
        });
    }

    #[test]
    fn saving_replaces_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = service(&dir);
        s.save_settings(&AISettings::default()).unwrap();
        let mut next = AISettings::default();
        next.default_provider = None;
        s.save_settings(&next).unwrap();
        assert!(fs::read_to_string(&s.settings_path)
            .unwrap()
            .contains("\"default_provider\": null"));
        assert_eq!(
            fs::read_dir(dir.path()).unwrap().count(),
            1,
            "temp left behind"
        );
    }
}
