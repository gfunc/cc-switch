//! Shared workspace file operations for Tauri commands and web routes.

use crate::config::write_text_file;
use crate::openclaw_config::get_openclaw_dir;
use regex::Regex;
use std::sync::LazyLock;

/// Allowed workspace filenames (whitelist for security)
const ALLOWED_FILES: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "MEMORY.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
    "BOOT.md",
];

static DAILY_MEMORY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$").unwrap());

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMemoryFileInfo {
    pub filename: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: u64,
    pub preview: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMemorySearchResult {
    pub filename: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: u64,
    pub snippet: String,
    pub match_count: usize,
}

pub struct WorkspaceService;

impl WorkspaceService {
    fn validate_filename(filename: &str) -> Result<(), String> {
        if !ALLOWED_FILES.contains(&filename) {
            return Err(format!(
                "Invalid workspace filename: {filename}. Allowed: {}",
                ALLOWED_FILES.join(", ")
            ));
        }
        Ok(())
    }

    fn validate_daily_memory_filename(filename: &str) -> Result<(), String> {
        if !DAILY_MEMORY_RE.is_match(filename) {
            return Err(format!(
                "Invalid daily memory filename: {filename}. Expected: YYYY-MM-DD.md"
            ));
        }
        Ok(())
    }

    fn floor_char_boundary(s: &str, mut i: usize) -> usize {
        if i >= s.len() {
            return s.len();
        }
        while !s.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
        if i >= s.len() {
            return s.len();
        }
        while !s.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    pub fn workspace_directory() -> std::path::PathBuf {
        get_openclaw_dir().join("workspace")
    }

    pub fn memory_directory() -> std::path::PathBuf {
        Self::workspace_directory().join("memory")
    }

    pub async fn read_workspace_file(filename: &str) -> Result<Option<String>, String> {
        Self::validate_filename(filename)?;
        let path = Self::workspace_directory().join(filename);
        if !path.exists() {
            return Ok(None);
        }
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| format!("Failed to read workspace file {filename}: {e}"))
    }

    pub async fn write_workspace_file(filename: &str, content: &str) -> Result<(), String> {
        Self::validate_filename(filename)?;
        let dir = Self::workspace_directory();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create workspace directory: {e}"))?;
        let path = dir.join(filename);
        write_text_file(&path, content)
            .map_err(|e| format!("Failed to write workspace file {filename}: {e}"))
    }

    pub async fn list_daily_memory_files() -> Result<Vec<DailyMemoryFileInfo>, String> {
        let memory_dir = Self::memory_directory();
        if !memory_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&memory_dir)
            .map_err(|e| format!("Failed to read memory directory: {e}"))?;

        let mut files = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let date = name.trim_end_matches(".md").to_string();
            let size_bytes = meta.len();
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let preview = std::fs::read_to_string(entry.path())
                .unwrap_or_default()
                .chars()
                .take(200)
                .collect::<String>();
            files.push(DailyMemoryFileInfo {
                filename: name,
                date,
                size_bytes,
                modified_at,
                preview,
            });
        }
        files.sort_by(|a, b| b.filename.cmp(&a.filename));
        Ok(files)
    }

    pub async fn read_daily_memory_file(filename: &str) -> Result<Option<String>, String> {
        Self::validate_daily_memory_filename(filename)?;
        let path = Self::memory_directory().join(filename);
        if !path.exists() {
            return Ok(None);
        }
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| format!("Failed to read daily memory file {filename}: {e}"))
    }

    pub async fn write_daily_memory_file(filename: &str, content: &str) -> Result<(), String> {
        Self::validate_daily_memory_filename(filename)?;
        let dir = Self::memory_directory();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create memory directory: {e}"))?;
        let path = dir.join(filename);
        write_text_file(&path, content)
            .map_err(|e| format!("Failed to write daily memory file {filename}: {e}"))
    }

    pub async fn delete_daily_memory_file(filename: &str) -> Result<(), String> {
        Self::validate_daily_memory_filename(filename)?;
        let path = Self::memory_directory().join(filename);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete daily memory file {filename}: {e}"))?;
        }
        Ok(())
    }

    pub async fn search_daily_memory_files(query: &str) -> Result<Vec<DailyMemorySearchResult>, String> {
        let memory_dir = Self::memory_directory();
        if !memory_dir.exists() || query.is_empty() {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let entries = std::fs::read_dir(&memory_dir)
            .map_err(|e| format!("Failed to read memory directory: {e}"))?;

        let mut results = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let date = name.trim_end_matches(".md").to_string();
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let content_lower = content.to_lowercase();
            let content_matches: Vec<usize> = content_lower
                .match_indices(&query_lower)
                .map(|(i, _)| i)
                .collect();
            let date_matches = date.to_lowercase().contains(&query_lower);

            if content_matches.is_empty() && !date_matches {
                continue;
            }

            let snippet = if let Some(&first_pos) = content_matches.first() {
                let start = if first_pos > 50 {
                    Self::floor_char_boundary(&content, first_pos - 50)
                } else {
                    0
                };
                let end = Self::ceil_char_boundary(&content, (first_pos + 70).min(content.len()));
                let mut s = String::new();
                if start > 0 {
                    s.push_str("...");
                }
                s.push_str(&content[start..end]);
                if end < content.len() {
                    s.push_str("...");
                }
                s
            } else {
                let end = Self::ceil_char_boundary(&content, 120.min(content.len()));
                let mut s = content[..end].to_string();
                if end < content.len() {
                    s.push_str("...");
                }
                s
            };

            let size_bytes = meta.len();
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            results.push(DailyMemorySearchResult {
                filename: name,
                date,
                size_bytes,
                modified_at,
                snippet,
                match_count: content_matches.len(),
            });
        }
        results.sort_by(|a, b| b.filename.cmp(&a.filename));
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::sync::{Mutex, OnceLock};

    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    fn set_test_home(path: &std::path::Path) {
        std::env::set_var("CC_SWITCH_TEST_HOME", path);
        std::env::set_var("HOME", path);
    }

    #[tokio::test]
    async fn workspace_service_reads_and_writes_workspace_file() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        WorkspaceService::write_workspace_file("AGENTS.md", "# agents").await.unwrap();
        let content = WorkspaceService::read_workspace_file("AGENTS.md").await.unwrap();
        assert_eq!(content, Some("# agents".to_string()));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_rejects_invalid_filename() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        let err = WorkspaceService::read_workspace_file("../../etc/passwd")
            .await
            .unwrap_err();
        assert!(err.contains("Invalid workspace filename"));

        let err = WorkspaceService::write_workspace_file("../../etc/passwd", "x")
            .await
            .unwrap_err();
        assert!(err.contains("Invalid workspace filename"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_returns_none_for_missing_file() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        let content = WorkspaceService::read_workspace_file("AGENTS.md").await.unwrap();
        assert_eq!(content, None);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_daily_memory_roundtrip() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-mem-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        // Write a daily memory file
        WorkspaceService::write_daily_memory_file("2024-01-15.md", "# Day 1\nHello world")
            .await
            .unwrap();

        // Read it back
        let content = WorkspaceService::read_daily_memory_file("2024-01-15.md")
            .await
            .unwrap();
        assert_eq!(content, Some("# Day 1\nHello world".to_string()));

        // List files
        let files = WorkspaceService::list_daily_memory_files().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "2024-01-15.md");
        assert_eq!(files[0].date, "2024-01-15");
        assert_eq!(files[0].preview, "# Day 1\nHello world");

        // Delete it
        WorkspaceService::delete_daily_memory_file("2024-01-15.md")
            .await
            .unwrap();
        let content = WorkspaceService::read_daily_memory_file("2024-01-15.md")
            .await
            .unwrap();
        assert_eq!(content, None);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_rejects_invalid_daily_memory_filename() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-bad-mem-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        let err = WorkspaceService::read_daily_memory_file("not-a-date.md")
            .await
            .unwrap_err();
        assert!(err.contains("Invalid daily memory filename"));

        let err = WorkspaceService::write_daily_memory_file("not-a-date.md", "x")
            .await
            .unwrap_err();
        assert!(err.contains("Invalid daily memory filename"));

        let err = WorkspaceService::delete_daily_memory_file("not-a-date.md")
            .await
            .unwrap_err();
        assert!(err.contains("Invalid daily memory filename"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_search_finds_content_and_date() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-search-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        WorkspaceService::write_daily_memory_file("2024-01-15.md", "# Day 1\nHello world")
            .await
            .unwrap();
        WorkspaceService::write_daily_memory_file("2024-01-16.md", "# Day 2\nGoodbye world")
            .await
            .unwrap();

        // Search by content
        let results = WorkspaceService::search_daily_memory_files("hello")
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "2024-01-15.md");
        assert_eq!(results[0].match_count, 1);
        assert!(results[0].snippet.contains("Hello"));

        // Search by date
        let results = WorkspaceService::search_daily_memory_files("2024-01-16")
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "2024-01-16.md");
        assert_eq!(results[0].match_count, 0); // date match only

        // Empty query returns empty
        let results = WorkspaceService::search_daily_memory_files("")
            .await
            .unwrap();
        assert!(results.is_empty());

        // No match
        let results = WorkspaceService::search_daily_memory_files("xyz123")
            .await
            .unwrap();
        assert!(results.is_empty());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_delete_is_idempotent() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-del-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        // Deleting a non-existent file should not error
        WorkspaceService::delete_daily_memory_file("2024-01-15.md")
            .await
            .unwrap();

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_list_sorts_descending() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-sort-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        WorkspaceService::write_daily_memory_file("2024-01-10.md", "A").await.unwrap();
        WorkspaceService::write_daily_memory_file("2024-01-15.md", "B").await.unwrap();
        WorkspaceService::write_daily_memory_file("2024-01-12.md", "C").await.unwrap();

        let files = WorkspaceService::list_daily_memory_files().await.unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].filename, "2024-01-15.md");
        assert_eq!(files[1].filename, "2024-01-12.md");
        assert_eq!(files[2].filename, "2024-01-10.md");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[tokio::test]
    async fn workspace_service_directories_are_correct() {
        let _guard = test_guard();
        let temp = temp_dir().join(format!("cc-switch-ws-svc-dir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        set_test_home(&temp);

        let ws = WorkspaceService::workspace_directory();
        assert!(ws.to_string_lossy().contains("workspace"));

        let mem = WorkspaceService::memory_directory();
        assert!(mem.to_string_lossy().contains("memory"));
        assert!(mem.to_string_lossy().contains("workspace"));

        let _ = std::fs::remove_dir_all(&temp);
    }
}
