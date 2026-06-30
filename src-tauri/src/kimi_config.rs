//! Kimi Code CLI configuration file read/write module
//!
//! Handles `~/.kimi-code/config.toml` (TOML format). Kimi uses additive
//! provider management: every provider is kept in `[providers.<name>]` and
//! `[models.<alias>]`, and `default_model` selects the active one.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::config::{atomic_write, get_app_config_dir, get_home_dir};
use crate::error::AppError;
use crate::provider::Provider;
use crate::settings::get_kimi_override_dir;

/// Kimi write result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KimiWriteOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
}

/// Resolve Kimi config directory.
/// Priority: CC Switch override > `KIMI_CODE_HOME` env > `~/.kimi-code`
pub fn get_kimi_dir() -> PathBuf {
    if let Some(override_dir) = get_kimi_override_dir() {
        return override_dir;
    }

    if let Some(raw) = std::env::var_os("KIMI_CODE_HOME") {
        let value = raw.to_string_lossy();
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    get_home_dir().join(".kimi-code")
}

/// Kimi main config file path
pub fn get_kimi_config_path() -> PathBuf {
    get_kimi_dir().join("config.toml")
}

/// Kimi MCP config file path
pub fn get_kimi_mcp_path() -> PathBuf {
    get_kimi_dir().join("mcp.json")
}

/// Kimi skills directory
pub fn get_kimi_skills_dir() -> PathBuf {
    get_kimi_dir().join("skills")
}

fn kimi_write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn read_kimi_config() -> Result<toml_edit::DocumentMut, AppError> {
    let path = get_kimi_config_path();
    if !path.exists() {
        return Ok(toml_edit::DocumentMut::new());
    }

    let content = fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    if content.trim().is_empty() {
        return Ok(toml_edit::DocumentMut::new());
    }

    content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| AppError::Config(format!("Failed to parse Kimi config.toml: {e}")))
}

fn create_kimi_backup(source: &str) -> Result<PathBuf, AppError> {
    let backup_dir = get_app_config_dir().join("backups").join("kimi");
    fs::create_dir_all(&backup_dir).map_err(|e| AppError::io(&backup_dir, e))?;

    let base_id = format!("kimi_{}", Local::now().format("%Y%m%d_%H%M%S"));
    let mut filename = format!("{base_id}.toml");
    let mut backup_path = backup_dir.join(&filename);
    let mut counter = 1;

    while backup_path.exists() {
        filename = format!("{base_id}_{counter}.toml");
        backup_path = backup_dir.join(&filename);
        counter += 1;
    }

    atomic_write(&backup_path, source.as_bytes())?;
    Ok(backup_path)
}

/// Extract Kimi provider settings from a provider's settings_config.
fn extract_kimi_provider_config(
    provider: &Provider,
) -> Result<(String, String, String, String, Vec<KimiModel>), AppError> {
    let provider_name = provider
        .settings_config
        .get("provider_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&provider.id)
        .to_string();

    let provider_type = provider
        .settings_config
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("kimi")
        .to_string();

    let base_url = provider
        .settings_config
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let api_key = provider
        .settings_config
        .get("api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let models: Vec<KimiModel> = provider
        .settings_config
        .get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let obj = item.as_object()?;
                    Some(KimiModel {
                        id: obj.get("id")?.as_str()?.to_string(),
                        name: obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        max_context_size: obj
                            .get("max_context_size")
                            .and_then(|v| v.as_u64())
                            .map(|n| n as usize),
                        capabilities: obj
                            .get("capabilities")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok((provider_name, provider_type, base_url, api_key, models))
}

#[derive(Debug, Clone, Default)]
struct KimiModel {
    id: String,
    name: Option<String>,
    max_context_size: Option<usize>,
    capabilities: Vec<String>,
}

/// Write or update a provider in Kimi config.toml and set it as default_model.
pub fn set_provider(provider: &Provider) -> Result<KimiWriteOutcome, AppError> {
    let _guard = kimi_write_lock().lock()?;

    let (provider_name, provider_type, base_url, api_key, models) =
        extract_kimi_provider_config(provider)?;

    if models.is_empty() {
        return Err(AppError::Config(format!(
            "Kimi provider '{}' has no models configured",
            provider.id
        )));
    }

    let mut doc = read_kimi_config()?;
    let path = get_kimi_config_path();
    let raw = if path.exists() {
        fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?
    } else {
        String::new()
    };

    // Update providers.<provider_name>
    let providers_table = doc
        .entry("providers")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_like_mut()
        .ok_or_else(|| AppError::Config("Kimi config providers section is not a table".into()))?;

    let provider_key = format!("\"{}\"", provider_name)
        .parse::<toml_edit::Key>()
        .map_err(|e| AppError::Config(format!("Invalid Kimi provider key: {e}")))?;

    let provider_table = providers_table
        .entry_format(&provider_key)
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_like_mut()
        .ok_or_else(|| AppError::Config("Kimi provider entry is not a table".into()))?;

    provider_table.insert("type", toml_edit::value(provider_type));
    provider_table.insert("base_url", toml_edit::value(base_url));
    provider_table.insert("api_key", toml_edit::value(api_key));

    // Update models.<provider_name>/<model_id> for each model; the first is default.
    let default_model_alias = format!("{}/{}", provider_name, models[0].id);

    let models_table = doc
        .entry("models")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_like_mut()
        .ok_or_else(|| AppError::Config("Kimi config models section is not a table".into()))?;

    for model in &models {
        let alias = format!("{}/{}", provider_name, model.id);
        let model_table = models_table
            .entry(&alias)
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
            .as_table_like_mut()
            .ok_or_else(|| AppError::Config("Kimi model entry is not a table".into()))?;

        model_table.insert("provider", toml_edit::value(provider_name.clone()));
        model_table.insert("model", toml_edit::value(model.id.clone()));
        if let Some(size) = model.max_context_size {
            model_table.insert("max_context_size", toml_edit::value(size as i64));
        }
        if !model.capabilities.is_empty() {
            let mut arr = toml_edit::Array::new();
            for cap in &model.capabilities {
                arr.push(cap.as_str());
            }
            model_table.insert("capabilities", toml_edit::value(arr));
        }
        if let Some(name) = &model.name {
            model_table.insert("display_name", toml_edit::value(name.clone()));
        }
    }

    // Set default_model
    doc.insert(
        "default_model",
        toml_edit::value(default_model_alias.clone()),
    );

    let new_raw = doc.to_string();
    if new_raw == raw {
        return Ok(KimiWriteOutcome::default());
    }

    let backup_path = if !raw.is_empty() {
        Some(create_kimi_backup(&raw)?)
    } else {
        None
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }

    atomic_write(&path, new_raw.as_bytes())?;

    log::info!(
        "Kimi provider '{}' written to live config (default_model: {})",
        provider.id,
        default_model_alias
    );

    Ok(KimiWriteOutcome {
        backup_path: backup_path.map(|p| p.display().to_string()),
    })
}

/// Read all providers from Kimi config.toml as a JSON map keyed by provider id.
/// Used for first-launch import and additive key collision checks.
pub fn get_providers() -> Result<serde_json::Map<String, serde_json::Value>, AppError> {
    let doc = read_kimi_config()?;
    let mut map = serde_json::Map::new();

    let Some(providers) = doc.get("providers").and_then(|v| v.as_table_like()) else {
        return Ok(map);
    };

    for (name, item) in providers.iter() {
        let Some(table) = item.as_table_like() else {
            continue;
        };

        let base_url = table
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let api_key = table
            .get("api_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let provider_type = table
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("kimi")
            .to_string();

        map.insert(
            name.to_string(),
            serde_json::json!({
                "provider_name": name,
                "type": provider_type,
                "base_url": base_url,
                "api_key": api_key,
            }),
        );
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    struct TempHome {
        #[allow(dead_code)]
        dir: TempDir,
    }

    impl TempHome {
        fn new() -> Self {
            let dir = TempDir::new().expect("failed to create temp home");
            env::set_var("HOME", dir.path());
            env::set_var("USERPROFILE", dir.path());
            env::set_var("CC_SWITCH_TEST_HOME", dir.path());
            Self { dir }
        }
    }

    #[test]
    #[serial]
    fn test_set_provider_writes_provider_model_and_default() {
        let _temp = TempHome::new();
        let provider = Provider {
            id: "moonshot-cn".into(),
            name: "Moonshot CN".into(),
            settings_config: json!({
                "provider_name": "moonshot-cn",
                "type": "kimi",
                "base_url": "https://api.moonshot.cn/v1",
                "api_key": "sk-test",
                "models": [{
                    "id": "kimi-k2-0711-preview",
                    "name": "Kimi K2.5",
                    "max_context_size": 262144,
                    "capabilities": ["thinking", "image_in"]
                }]
            }),
            ..Provider::with_id("x".into(), "x".into(), json!({}), None)
        };

        set_provider(&provider).unwrap();

        let raw = fs::read_to_string(get_kimi_config_path()).unwrap();
        assert!(raw.contains("default_model = \"moonshot-cn/kimi-k2-0711-preview\""));
        assert!(raw.contains("[providers.\"moonshot-cn\"]"));
        assert!(raw.contains("base_url = \"https://api.moonshot.cn/v1\""));
        assert!(raw.contains("[models.\"moonshot-cn/kimi-k2-0711-preview\"]"));
    }

    #[test]
    #[serial]
    fn test_set_provider_preserves_unrelated_sections() {
        let _temp = TempHome::new();
        fs::create_dir_all(get_kimi_dir()).unwrap();
        fs::write(get_kimi_config_path(), "[thinking]\nmode = \"auto\"\n").unwrap();

        let provider = Provider {
            id: "moonshot-ai".into(),
            name: "Moonshot AI".into(),
            settings_config: json!({
                "provider_name": "moonshot-ai",
                "type": "kimi",
                "base_url": "https://api.moonshot.ai/v1",
                "api_key": "sk-test",
                "models": [{ "id": "kimi-k2-0711-preview", "name": "Kimi K2.5" }]
            }),
            ..Provider::with_id("x".into(), "x".into(), json!({}), None)
        };

        set_provider(&provider).unwrap();

        let raw = fs::read_to_string(get_kimi_config_path()).unwrap();
        assert!(raw.contains("[thinking]"));
        assert!(raw.contains("default_model"));
    }
}
