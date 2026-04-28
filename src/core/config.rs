use crate::core::xdg;
use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub default_profile: Option<String>,
    #[serde(default = "default_registry_urls")]
    pub registry_urls: Vec<String>,
    #[serde(default = "default_true")]
    pub use_fzf: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_false")]
    pub auto_update: bool,
    #[serde(default = "default_true")]
    pub auto_install_missing: bool,
    #[serde(default = "default_false")]
    pub strict_mode: bool,
    #[serde(default = "default_true")]
    pub fallback_to_plain_nvim: bool,
    pub profiles_dir: Option<String>,
}

fn default_registry_urls() -> Vec<String> {
    vec!["https://raw.githubusercontent.com/zx0r/nvimx-registry/main/index.json".to_string()]
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_output() -> String {
    "text".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_profile: None,
            registry_urls: default_registry_urls(),
            use_fzf: true,
            log_level: "info".to_string(),
            output: "text".to_string(),
            auto_update: false,
            auto_install_missing: true,
            strict_mode: false,
            fallback_to_plain_nvim: true,
            profiles_dir: None,
        }
    }
}

pub fn load() -> Config {
    let path = xdg::nvimx_config_dir()
        .unwrap_or_default()
        .join("config.toml");
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut config: Config = toml::from_str(&content).unwrap_or_default();

    // Backwards compatibility for registry_url
    if config.registry_urls.is_empty() {
        if let Ok(value) = content.parse::<toml::Value>() {
            if let Some(url) = value.get("registry_url").and_then(|v| v.as_str()) {
                config.registry_urls = vec![url.to_string()];
            } else {
                config.registry_urls = default_registry_urls();
            }
        } else {
            config.registry_urls = default_registry_urls();
        }
    }

    config
}

pub fn get_profiles_dir(config: &Config) -> PathBuf {
    if let Some(ref dir) = config.profiles_dir {
        if let Some(stripped) = dir.strip_prefix("~/") {
            dirs::home_dir().unwrap_or_default().join(stripped)
        } else if dir == "~" {
            dirs::home_dir().unwrap_or_default()
        } else {
            PathBuf::from(dir)
        }
    } else {
        xdg::config_root().unwrap_or_default()
    }
}
