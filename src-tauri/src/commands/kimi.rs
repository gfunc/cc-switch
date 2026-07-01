use tauri::State;

use crate::kimi_config;
use crate::store::AppState;

/// Import providers from Kimi live config to database.
///
/// Kimi uses additive mode — users may already have providers
/// configured in config.toml.
#[tauri::command]
pub fn import_kimi_providers_from_live(state: State<'_, AppState>) -> Result<usize, String> {
    crate::services::provider::import_kimi_providers_from_live(state.inner())
        .map_err(|e| e.to_string())
}

/// Get provider names in the Kimi live config.
#[tauri::command]
pub fn get_kimi_live_provider_ids() -> Result<Vec<String>, String> {
    kimi_config::get_providers()
        .map(|providers| providers.keys().cloned().collect())
        .map_err(|e| e.to_string())
}
