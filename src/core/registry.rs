use crate::core::xdg;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum RegistryError {
    #[error("Registry is empty")]
    EmptyRegistry,
    #[error("Invalid profile name '{0}'")]
    InvalidName(String),
    #[error("Duplicate name '{0}'")]
    DuplicateName(String),
    #[error("Invalid repo URL '{0}'")]
    InvalidUrl(String),
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] ureq::Error),
    #[error("All registries failed and no cache available")]
    AllFailed,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Registry {
    pub profiles: Vec<RegistryProfile>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegistryProfile {
    pub name: String,
    pub repo: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub profile_type: String,
    #[serde(default)]
    pub default: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegistryMeta {
    fetched_at: u64,
    ttl: u64,
    url: String,
}

fn get_cache_paths() -> Result<(PathBuf, PathBuf)> {
    let base = xdg::cache_root()?.join("registry");
    fs::create_dir_all(&base)?;
    Ok((base.join("data.json"), base.join("meta.json")))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn fetch_smart(urls: &[String], force_update: bool) -> Result<Registry> {
    let (data_path, meta_path) = get_cache_paths()?;

    if !force_update && data_path.exists() && meta_path.exists() {
        let cached_reg = fs::read_to_string(&meta_path)
            .ok()
            .and_then(|meta_str| serde_json::from_str::<RegistryMeta>(&meta_str).ok())
            .filter(|meta| now() - meta.fetched_at < meta.ttl)
            .and_then(|_| fs::read_to_string(&data_path).ok())
            .and_then(|data_str| serde_json::from_str::<Registry>(&data_str).ok());

        if let Some(reg) = cached_reg {
            return Ok(reg);
        }
    }

    // Try fetch
    for url in urls {
        match fetch_raw(url) {
            Ok(registry) => {
                let _ = save_cache(&registry, url);
                return Ok(registry);
            }
            Err(_) => continue,
        }
    }

    // Fallback to expired cache if available
    if data_path.exists() {
        let data_str = fs::read_to_string(&data_path)?;
        return Ok(serde_json::from_str(&data_str)?);
    }

    Err(RegistryError::AllFailed.into())
}

pub fn fetch_raw(url: &str) -> Result<Registry, RegistryError> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(3)))
        .build()
        .new_agent();

    let registry: Registry = agent
        .get(url)
        .header("User-Agent", "nvimx")
        .call()?
        .body_mut()
        .read_json()?;

    validate(&registry).map_err(|e| RegistryError::InvalidName(e.to_string()))?;

    Ok(registry)
}

fn save_cache(registry: &Registry, url: &str) -> Result<()> {
    let (data_path, meta_path) = get_cache_paths()?;
    fs::write(data_path, serde_json::to_string(registry)?)?;
    let meta = RegistryMeta {
        fetched_at: now(),
        ttl: 3600, // 1 hour
        url: url.to_string(),
    };
    fs::write(meta_path, serde_json::to_string(&meta)?)?;
    Ok(())
}

pub fn clear_cache() -> Result<()> {
    let (data_path, meta_path) = get_cache_paths()?;
    let _ = fs::remove_file(data_path);
    let _ = fs::remove_file(meta_path);
    Ok(())
}

fn validate(registry: &Registry) -> Result<()> {
    if registry.profiles.is_empty() {
        return Err(anyhow::anyhow!("empty"));
    }
    let mut names = HashSet::new();
    for p in &registry.profiles {
        if !names.insert(&p.name) {
            return Err(anyhow::anyhow!("duplicate"));
        }
    }
    Ok(())
}
