use crate::domains::ai::providers::{ProviderConfig, ProviderType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// AI Settings that stores all provider configurations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AISettings {
    /// Map of provider type to its configuration
    pub providers: HashMap<String, ProviderConfig>,
    /// Default provider type
    pub default_provider: Option<String>,
}

impl Default for AISettings {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Initialize default configurations for all providers
        providers.insert(
            "Ollama".to_string(),
            ProviderConfig::default_for_type(ProviderType::Ollama),
        );
        providers.insert(
            "Gemini".to_string(),
            ProviderConfig::default_for_type(ProviderType::Gemini),
        );

        Self {
            providers,
            default_provider: None,
        }
    }
}

pub struct AISettingsService {
    settings_path: PathBuf,
}

impl AISettingsService {
    pub fn new() -> Self {
        let mut settings_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        settings_path.push("portal-desktop");
        settings_path.push("ai-settings.json");
        
        Self { settings_path }
    }

    /// Load AI settings from file
    pub fn load_settings(&self) -> Result<AISettings, String> {
        if !self.settings_path.exists() {
            return Ok(AISettings::default());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| format!("Failed to read AI settings file: {}", e))?;

        // Parse as generic JSON first to filter out unsupported providers
        let mut json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse AI settings JSON: {}", e))?;

        // Filter out unsupported providers (OpenAI, Anthropic) from the providers map
        // We need to check both the key name and the provider_type field inside each config
        let supported_providers = vec!["Ollama", "Gemini"];
        let keys_to_remove: Vec<String> = if let Some(Value::Object(providers_obj)) = json.get("providers") {
            providers_obj
                .iter()
                .filter(|(key, config_value)| {
                    // Remove if key is unsupported
                    if !supported_providers.contains(&key.as_str()) {
                        return true;
                    }
                    // Also remove if provider_type field inside config is unsupported
                    if let Some(config_obj) = config_value.as_object() {
                        if let Some(Value::String(provider_type)) = config_obj.get("provider_type") {
                            if !supported_providers.contains(&provider_type.as_str()) {
                                return true;
                            }
                        }
                    }
                    false
                })
                .map(|(key, _)| key.clone())
                .collect()
        } else {
            Vec::new()
        };
        
        // Now remove the keys from the mutable reference
        if !keys_to_remove.is_empty() {
            if let Some(Value::Object(ref mut providers_obj_mut)) = json.get_mut("providers") {
                for key in keys_to_remove {
                    providers_obj_mut.remove(&key);
                }
            }
        }

        // Also check if default_provider references an unsupported provider
        if let Some(Value::String(ref default_provider)) = json.get("default_provider") {
            if !supported_providers.contains(&default_provider.as_str()) {
                if let Some(obj) = json.as_object_mut() {
                    obj.insert("default_provider".to_string(), Value::Null);
                }
            }
        }

        // Now deserialize the cleaned JSON
        let settings: AISettings = serde_json::from_value(json)
            .map_err(|e| format!("Failed to parse AI settings: {}", e))?;

        // Ensure we have default providers if they're missing
        let mut settings = settings;
        if !settings.providers.contains_key("Ollama") {
            settings.providers.insert(
                "Ollama".to_string(),
                ProviderConfig::default_for_type(ProviderType::Ollama),
            );
        }
        if !settings.providers.contains_key("Gemini") {
            settings.providers.insert(
                "Gemini".to_string(),
                ProviderConfig::default_for_type(ProviderType::Gemini),
            );
        }

        // Save the cleaned settings back to file
        self.save_settings(&settings)?;

        Ok(settings)
    }

    /// Save AI settings to file
    pub fn save_settings(&self, settings: &AISettings) -> Result<(), String> {
        // Create directory if it doesn't exist
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

    /// Get provider configuration
    pub fn get_provider_config(&self, provider_type: ProviderType) -> Result<ProviderConfig, String> {
        let settings = self.load_settings()?;
        let key = format!("{:?}", provider_type);
        
        settings.providers
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("Provider {:?} not found in settings", provider_type))
    }

    /// Save provider configuration
    pub fn save_provider_config(&self, config: ProviderConfig) -> Result<(), String> {
        let mut settings = self.load_settings()?;
        let key = format!("{:?}", config.provider_type);
        
        settings.providers.insert(key, config);
        self.save_settings(&settings)
    }

    /// Get all provider configurations
    pub fn get_all_providers(&self) -> Result<Vec<ProviderConfig>, String> {
        let settings = self.load_settings()?;
        Ok(settings.providers.values().cloned().collect())
    }

    /// Get default provider
    pub fn get_default_provider(&self) -> Result<Option<ProviderType>, String> {
        let settings = self.load_settings()?;
        match settings.default_provider {
            Some(ref name) => {
                // Try to find provider by name
                for (key, config) in &settings.providers {
                    if key == name || format!("{:?}", config.provider_type) == *name {
                        return Ok(Some(config.provider_type.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Set default provider
    pub fn set_default_provider(&self, provider_type: ProviderType) -> Result<(), String> {
        let mut settings = self.load_settings()?;
        let key = format!("{:?}", provider_type);
        
        // Verify provider exists
        if !settings.providers.contains_key(&key) {
            return Err(format!("Provider {:?} is not configured", provider_type));
        }
        
        settings.default_provider = Some(key);
        self.save_settings(&settings)
    }

    /// Delete provider configuration
    pub fn delete_provider_config(&self, provider_type: ProviderType) -> Result<(), String> {
        let mut settings = self.load_settings()?;
        let key = format!("{:?}", provider_type);
        
        // Don't allow deleting if it's the default provider
        if settings.default_provider.as_ref() == Some(&key) {
            return Err("Cannot delete the default provider".to_string());
        }
        
        settings.providers.remove(&key);
        self.save_settings(&settings)
    }
}

