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

        if let Some(Value::Object(providers_obj)) = json.get_mut("providers") {
            for key in legacy_keys {
                if let Some(legacy) = providers_obj.remove(key) {
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
            providers_obj.retain(|key, _| key == "AgentPlatform");
        }

        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "default_provider".to_string(),
                Value::String("AgentPlatform".to_string()),
            );
        }

        let mut settings: AISettings = serde_json::from_value(json)
            .map_err(|e| format!("Failed to parse AI settings: {}", e))?;

        if !settings.providers.contains_key("AgentPlatform") {
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
        self.save_settings(&settings)?;
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AISettings) -> Result<(), String> {
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize AI settings: {}", e))?;
        fs::write(&self.settings_path, content)
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
