// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cc-switch")]
#[command(about = "All-in-One Assistant for Claude Code, Codex & Gemini CLI")]
#[command(version = "3.10.3")]
struct Cli {
    /// Run in headless mode (no GUI, API server only)
    #[arg(long, short = 'H')]
    headless: bool,

    /// Port for the API server (headless mode only)
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Bind to all interfaces (0.0.0.0) instead of localhost only
    #[arg(long, short = 'a')]
    bind_all: bool,
}

fn main() {
    // 在 Linux 上设置 WebKit 环境变量以解决 DMA-BUF 渲染问题
    // 某些 Linux 系统（如 Debian 13.2、Nvidia GPU）上 WebKitGTK 的 DMA-BUF 渲染器可能导致白屏/黑屏
    // 参考: https://github.com/tauri-apps/tauri/issues/9394
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    let cli = Cli::parse();

    if cli.headless {
        println!("🚀 Starting CC Switch in headless mode...");
        println!(
            "   API Server: http://{}:{}",
            if cli.bind_all { "0.0.0.0" } else { "127.0.0.1" },
            cli.port
        );

        std::env::set_var("CC_SWITCH_ENABLE_WEB", "true");
        std::env::set_var("CC_SWITCH_WEB_PORT", cli.port.to_string());
        if cli.bind_all {
            std::env::set_var("CC_SWITCH_WEB_BIND_ALL", "true");
        }

        cc_switch_lib::run_headless();
    } else {
        cc_switch_lib::run();
    }
}
