// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cc-switch")]
#[command(about = "All-in-One Assistant for Claude Code, Codex & Gemini CLI")]
#[command(version = "3.10.3")]
struct Cli {
    #[arg(long, short = 'H', help = "Run in headless mode (no GUI)")]
    headless: bool,
    #[arg(long, short = 'p', default_value = "8080", help = "Port for the proxy/API server")]
    port: u16,
    #[arg(long, short = 'a', help = "Bind to all interfaces (0.0.0.0)")]
    bind_all: bool,
    #[arg(long, short = 'w', help = "Enable embedded web server for browser access")]
    enable_web: bool,
    #[arg(long, default_value = "3001", help = "Port for the web server")]
    web_port: u16,
    #[arg(long, short = 'd', help = "Run as daemon (background process)")]
    daemon: bool,
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
        // 禁用 WebKitGTK 合成模式，规避 resize 时 webview 崩溃以及部分 Wayland
        // 合成器下的 surface 协商问题（整窗 UI 点击无响应、必须最大化-还原才能恢复）。
        // 参考: https://github.com/tauri-apps/tauri/issues/9394
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    let cli = Cli::parse();
    if cli.daemon {
        daemonize();
    }

    if cli.headless || cli.daemon {
        println!("🚀 Starting CC Switch in headless mode...");
        println!(
            "   Proxy Server: http://{}:{}",
            if cli.bind_all { "0.0.0.0" } else { "127.0.0.1" },
            cli.port
        );
        
        if cli.enable_web {
            println!(
                "   Web UI: http://{}:{}",
                if cli.bind_all { "0.0.0.0" } else { "127.0.0.1" },
                cli.web_port
            );
        }
        std::env::set_var("CC_SWITCH_PROXY_PORT", cli.port.to_string());
        if cli.bind_all {
            std::env::set_var("CC_SWITCH_BIND_ALL", "true");
        }
        
        if cli.enable_web {
            std::env::set_var("CC_SWITCH_ENABLE_WEB", "true");
            std::env::set_var("CC_SWITCH_WEB_PORT", cli.web_port.to_string());
        }
        cc_switch_lib::run_headless(cli.enable_web, cli.web_port);
    } else {
        if cli.enable_web {
            std::env::set_var("CC_SWITCH_ENABLE_WEB", "true");
            std::env::set_var("CC_SWITCH_WEB_PORT", cli.web_port.to_string());
        }
        cc_switch_lib::run();
    }
}

#[cfg(unix)]
fn daemonize() {
    use std::process::{self, Command, Stdio};
    
    if std::env::var("CC_SWITCH_DAEMONIZED").is_ok() {
        return;
    }
    
    println!("👻 Starting as daemon...");
    
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.env("CC_SWITCH_DAEMONIZED", "1")
        .env_remove("CC_SWITCH_DAEMON")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    
    for arg in args {
        if arg != "--daemon" && arg != "-d" {
            cmd.arg(arg);
        }
    }
    
    match cmd.spawn() {
        Ok(child) => {
            println!("   Daemon started with PID: {}", child.id());
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Failed to start daemon: {e}");
            process::exit(1);
        }
    }
}

#[cfg(not(unix))]
fn daemonize() {
    eprintln!("❌ Daemon mode is only supported on Unix systems");
    std::process::exit(1);
}
