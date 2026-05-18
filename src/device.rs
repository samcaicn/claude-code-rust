// Copyright 2025
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub timeout_sec: Option<u64>,
    #[serde(rename = "device.uuid")]
    pub device_uuid: Option<String>,
    pub language: Option<String>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            model: Some("doubao-seed-2-0-lite-260215".to_string()),
            provider: Some("custom".to_string()),
            timeout_sec: Some(120),
            device_uuid: None,
            language: Some("en".to_string()),
        }
    }
}

const LANG_EN: &str = "en";
const LANG_ZH: &str = "zh";

fn get_system_lang() -> &'static str {
    let lang = env::var("LANG").or_else(|_| env::var("LC_ALL")).unwrap_or_default();
    if lang.starts_with("zh") {
        LANG_ZH
    } else {
        LANG_EN
    }
}

fn prompt_language_selection() -> String {
    let sys_lang = get_system_lang();
    if sys_lang == LANG_ZH {
        println!();
        println!("╔══════════════════════════════════════════╗");
        println!("║          Claude Rust 设置                ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║  请选择界面语言 / Select language:       ║");
        println!("║                                          ║");
        println!("║  [1] English                            ║");
        println!("║  [2] 中文                                ║");
        println!("║                                          ║");
        println!("╚══════════════════════════════════════════╝");
        print!("请输入选择 [2]: ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let choice = input.trim();
            if choice.is_empty() || choice == "2" {
                return LANG_ZH.to_string();
            }
        }
        LANG_ZH.to_string()
    } else {
        println!();
        println!("╔══════════════════════════════════════════╗");
        println!("║          Claude Rust Setup              ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║  Select language:                       ║");
        println!("║                                          ║");
        println!("║  [1] English                            ║");
        println!("║  [2] 中文                                ║");
        println!("║                                          ║");
        println!("╚══════════════════════════════════════════╝");
        print!("Enter choice [1]: ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let choice = input.trim();
            if choice == "2" {
                return LANG_ZH.to_string();
            }
        }
        LANG_EN.to_string()
    }
}

pub fn select_language() -> String {
    if let Ok(lang) = env::var("CLAUDE_RS_LANG") {
        if lang == LANG_ZH || lang == LANG_EN {
            return lang;
        }
    }

    if let Some(config) = load_config() {
        if let Some(ref lang) = config.language {
            if lang == LANG_ZH || lang == LANG_EN {
                return lang.clone();
            }
        }
    }

    let selected = prompt_language_selection();
    unsafe { env::set_var("CLAUDE_RS_LANG", &selected) };
    selected
}

#[derive(Debug, Deserialize, Serialize)]
struct RegisterRequest {
    hardware_fingerprint: String,
    platform: String,
}

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    #[serde(rename = "success")]
    success: Option<bool>,
    #[serde(rename = "data")]
    data: Option<RegisterData>,
    #[serde(rename = "user_token")]
    user_token: Option<String>,
    #[serde(rename = "uuid")]
    uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisterData {
    #[serde(rename = "user_token")]
    user_token: Option<String>,
    #[serde(rename = "uuid")]
    uuid: Option<String>,
}

pub fn get_hardware_fingerprint() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output();

        if let Ok(output) = output {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return uuid.to_string();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
            return machine_id.trim().to_string();
        }
    }

    format!("unknown-{}-{}", hostname::get().unwrap_or_default().to_string_lossy(), std::process::id())
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("CLAUDE_RS_CONFIG") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidates = [
        exe_dir.join("config.toml"),
        exe_dir.parent()?.join("config.toml"),
        exe_dir.join("config.toml"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    if let Ok(cwd) = env::current_dir() {
        let local = cwd.join("config.toml");
        if local.exists() {
            return Some(local);
        }
    }

    None
}

fn default_config_path() -> PathBuf {
    env::var("CLAUDE_RS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
                .join("config.toml")
        })
}

pub fn load_config() -> Option<DeviceConfig> {
    let path = resolve_config_path()?;
    let content = fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

pub fn has_valid_token() -> bool {
    load_config()
        .and_then(|c| c.api_key)
        .map(|k| !k.is_empty() && k != "YOUR_API_KEY_HERE")
        .unwrap_or(false)
}

pub fn register_device() -> anyhow::Result<DeviceConfig> {
    let fingerprint = get_hardware_fingerprint();
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let base_url = env::var("CLAUDE_RS_REGISTER_URL")
        .unwrap_or_else(|_| "https://admin.tuptup.top/api".to_string());

    let url = format!("{}/device/register", base_url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request = RegisterRequest {
        hardware_fingerprint: fingerprint.clone(),
        platform: platform.to_string(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()?;

    if !response.status().is_success() {
        anyhow::bail!("Registration failed with status: {}", response.status());
    }

    let data: RegisterResponse = response.json()?;

    let user_token = data
        .data
        .as_ref()
        .and_then(|d| d.user_token.clone())
        .or(data.user_token)
        .ok_or_else(|| anyhow::anyhow!("No user_token in response"))?;

    let uuid = data
        .data
        .as_ref()
        .and_then(|d| d.uuid.clone())
        .or(data.uuid);

    let config = DeviceConfig {
        api_key: Some(user_token),
        base_url: Some(base_url.clone()),
        model: Some("doubao-seed-2-0-lite-260215".to_string()),
        provider: Some("custom".to_string()),
        timeout_sec: Some(120),
        device_uuid: uuid,
        language: None,
    };

    Ok(config)
}

pub fn apply_config_to_env(config: &DeviceConfig) {
    if let Some(ref api_key) = config.api_key {
        unsafe { env::set_var("ANTHROPIC_AUTH_TOKEN", api_key) };
    }
    if let Some(ref base_url) = config.base_url {
        unsafe { env::set_var("ANTHROPIC_BASE_URL", base_url) };
    }
    if let Some(ref model) = config.model {
        unsafe { env::set_var("ANTHROPIC_DEFAULT_MODEL", model) };
    }
    if let Some(ref provider) = config.provider {
        unsafe { env::set_var("CLAUDE_RS_PROVIDER", provider) };
    }
}

pub fn init() -> anyhow::Result<DeviceConfig> {
    let selected_lang = select_language();
    unsafe { env::set_var("CLAUDE_RS_LANG", &selected_lang) };

    if let Some(mut config) = load_config() {
        config.language = Some(selected_lang.clone());
        if has_valid_token() {
            save_config(&config)?;
            apply_config_to_env(&config);
            return Ok(config);
        }
    }

    let config = register_device()?;
    let mut config = config;
    config.language = Some(selected_lang);
    save_config(&config)?;
    apply_config_to_env(&config);

    let is_zh = env::var("CLAUDE_RS_LANG").unwrap_or_default() == LANG_ZH;
    if is_zh {
        eprintln!("✓ 设备注册成功");
        if let Some(ref uuid) = config.device_uuid {
            eprintln!("  UUID: {}", uuid);
        }
        if let Some(ref key) = config.api_key {
            eprintln!("  Token: {}...", &key[..key.len().min(20)]);
        }
    } else {
        eprintln!("✓ Device registered successfully");
        if let Some(ref uuid) = config.device_uuid {
            eprintln!("  UUID: {}", uuid);
        }
        if let Some(ref key) = config.api_key {
            eprintln!("  Token: {}...", &key[..key.len().min(20)]);
        }
    }
    eprintln!();

    Ok(config)
}

fn save_config(config: &DeviceConfig) -> anyhow::Result<()> {
    let config_path = default_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}
