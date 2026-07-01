//! Kimi MCP sync module
//!
//! Converts between CC Switch unified MCP format and Kimi's `~/.kimi-code/mcp.json`.

use serde_json::{json, Value};
use std::collections::HashMap;

use crate::app_config::{McpApps, McpServer, MultiAppConfig};
use crate::config::{read_json_file, write_json_file};
use crate::error::AppError;
use crate::kimi_config;

use super::validation::validate_server_spec;

fn should_sync_kimi_mcp() -> bool {
    kimi_config::get_kimi_dir().exists()
}

fn read_kimi_mcp() -> Result<Value, AppError> {
    let path = kimi_config::get_kimi_mcp_path();
    if !path.exists() {
        return Ok(json!({ "mcpServers": {} }));
    }
    read_json_file(&path)
}

fn write_kimi_mcp(value: &Value) -> Result<(), AppError> {
    let path = kimi_config::get_kimi_mcp_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }
    write_json_file(&path, value)
}

fn convert_to_kimi_format(spec: &Value) -> Result<Value, AppError> {
    let obj = spec
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP spec must be a JSON object".into()))?;

    let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("stdio");
    let mut result = serde_json::Map::new();

    match typ {
        "stdio" => {
            if let Some(command) = obj.get("command") {
                result.insert("command".into(), command.clone());
            }
            if let Some(args) = obj.get("args") {
                result.insert("args".into(), args.clone());
            }
            if let Some(env) = obj.get("env") {
                result.insert("env".into(), env.clone());
            }
        }
        "sse" => {
            result.insert("transport".into(), json!("sse"));
            if let Some(url) = obj.get("url") {
                result.insert("url".into(), url.clone());
            }
            if let Some(headers) = obj.get("headers") {
                result.insert("headers".into(), headers.clone());
            }
        }
        "http" => {
            if let Some(url) = obj.get("url") {
                result.insert("url".into(), url.clone());
            }
            if let Some(headers) = obj.get("headers") {
                result.insert("headers".into(), headers.clone());
            }
        }
        _ => return Err(AppError::McpValidation(format!("Unknown MCP type: {typ}"))),
    }

    Ok(Value::Object(result))
}

fn convert_from_kimi_format(id: &str, spec: &Value) -> Result<Value, AppError> {
    let obj = spec
        .as_object()
        .ok_or_else(|| AppError::McpValidation("Kimi MCP spec must be a JSON object".into()))?;

    let mut result = serde_json::Map::new();

    if obj.contains_key("command") {
        result.insert("type".into(), json!("stdio"));
        if let Some(command) = obj.get("command") {
            result.insert("command".into(), command.clone());
        }
        if let Some(args) = obj.get("args") {
            result.insert("args".into(), args.clone());
        }
        if let Some(env) = obj.get("env") {
            result.insert("env".into(), env.clone());
        }
    } else if obj.contains_key("url") {
        let transport = obj
            .get("transport")
            .and_then(|v| v.as_str())
            .unwrap_or("http");
        result.insert("type".into(), json!(transport));
        if let Some(url) = obj.get("url") {
            result.insert("url".into(), url.clone());
        }
        if let Some(headers) = obj.get("headers") {
            result.insert("headers".into(), headers.clone());
        }
    } else {
        return Err(AppError::McpValidation(format!(
            "Kimi MCP server '{id}' has neither 'command' nor 'url' field"
        )));
    }

    Ok(Value::Object(result))
}

pub fn sync_single_server_to_kimi(
    _config: &MultiAppConfig,
    id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    if !should_sync_kimi_mcp() {
        return Ok(());
    }

    let kimi_spec = convert_to_kimi_format(server_spec)?;
    let mut mcp = read_kimi_mcp()?;
    let servers = mcp
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| AppError::Config("Kimi mcp.json missing mcpServers object".into()))?;

    servers.insert(id.to_string(), kimi_spec);
    write_kimi_mcp(&mcp)
}

pub fn remove_server_from_kimi(id: &str) -> Result<(), AppError> {
    if !should_sync_kimi_mcp() {
        return Ok(());
    }

    let mut mcp = read_kimi_mcp()?;
    let servers = mcp
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| AppError::Config("Kimi mcp.json missing mcpServers object".into()))?;

    servers.remove(id);
    write_kimi_mcp(&mcp)
}

pub fn import_from_kimi(config: &mut MultiAppConfig) -> Result<usize, AppError> {
    if !should_sync_kimi_mcp() {
        return Ok(0);
    }

    let mcp = read_kimi_mcp()?;
    let servers = config.mcp.servers.get_or_insert_with(HashMap::new);

    let kimi_servers = mcp
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut changed = 0;
    for (id, spec) in kimi_servers {
        let unified_spec = match convert_from_kimi_format(&id, &spec) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Skip invalid Kimi MCP server '{id}': {e}");
                continue;
            }
        };

        if let Err(e) = validate_server_spec(&unified_spec) {
            log::warn!("Skip invalid MCP server '{id}' after conversion: {e}");
            continue;
        }

        if let Some(existing) = servers.get_mut(&id) {
            if !existing.apps.kimi {
                existing.apps.kimi = true;
                changed += 1;
            }
        } else {
            servers.insert(
                id.clone(),
                McpServer {
                    id: id.clone(),
                    name: id.clone(),
                    server: unified_spec,
                    apps: McpApps {
                        claude: false,
                        codex: false,
                        gemini: false,
                        opencode: false,
                        hermes: false,
                        kimi: true,
                    },
                    description: None,
                    homepage: None,
                    docs: None,
                    tags: Vec::new(),
                },
            );
            changed += 1;
        }
    }

    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_stdio_to_kimi() {
        let spec = json!({
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem"],
            "env": { "HOME": "/Users/test" }
        });
        let result = convert_to_kimi_format(&spec).unwrap();
        assert_eq!(result["command"], "npx");
        assert_eq!(result["args"][0], "-y");
        assert!(result.get("type").is_none());
    }

    #[test]
    fn test_convert_sse_to_kimi() {
        let spec = json!({
            "type": "sse",
            "url": "https://example.com/mcp",
            "headers": { "Authorization": "Bearer xxx" }
        });
        let result = convert_to_kimi_format(&spec).unwrap();
        assert_eq!(result["transport"], "sse");
        assert_eq!(result["url"], "https://example.com/mcp");
        assert_eq!(result["headers"]["Authorization"], "Bearer xxx");
    }

    #[test]
    fn test_convert_http_to_kimi() {
        let spec = json!({
            "type": "http",
            "url": "https://example.com/mcp",
            "headers": { "Authorization": "Bearer xxx" }
        });
        let result = convert_to_kimi_format(&spec).unwrap();
        assert!(result.get("transport").is_none());
        assert_eq!(result["url"], "https://example.com/mcp");
        assert_eq!(result["headers"]["Authorization"], "Bearer xxx");
    }

    #[test]
    fn test_convert_kimi_http_to_unified() {
        let spec = json!({
            "url": "https://example.com/mcp",
            "headers": { "Authorization": "Bearer xxx" }
        });
        let result = convert_from_kimi_format("remote", &spec).unwrap();
        assert_eq!(result["type"], "http");
        assert_eq!(result["url"], "https://example.com/mcp");
        assert_eq!(result["headers"]["Authorization"], "Bearer xxx");
    }

    #[test]
    fn test_convert_kimi_sse_to_unified() {
        let spec = json!({
            "transport": "sse",
            "url": "https://example.com/mcp",
            "headers": { "Authorization": "Bearer xxx" }
        });
        let result = convert_from_kimi_format("remote", &spec).unwrap();
        assert_eq!(result["type"], "sse");
        assert_eq!(result["url"], "https://example.com/mcp");
        assert_eq!(result["headers"]["Authorization"], "Bearer xxx");
    }
}
