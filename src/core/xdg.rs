use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

fn get_home() -> Result<PathBuf> {
    dirs::home_dir().context("Could not find home directory")
}

pub fn config_root() -> Result<PathBuf> {
    if cfg!(windows) {
        Ok(env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join("AppData").join("Local"))
            .join("nvim"))
    } else {
        Ok(env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join(".config"))
            .join("nvim"))
    }
}

pub fn data_root() -> Result<PathBuf> {
    if cfg!(windows) {
        Ok(env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join("AppData").join("Local"))
            .join("nvim-data"))
    } else {
        Ok(env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join(".local/share"))
            .join("nvim"))
    }
}

pub fn state_root() -> Result<PathBuf> {
    if cfg!(windows) {
        data_root()
    } else {
        Ok(env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join(".local/state"))
            .join("nvim"))
    }
}

pub fn cache_root() -> Result<PathBuf> {
    if cfg!(windows) {
        Ok(env::var("TEMP")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                get_home()
                    .unwrap_or_default()
                    .join("AppData")
                    .join("Local")
                    .join("Temp")
            })
            .join("nvim"))
    } else {
        Ok(env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_home().unwrap_or_default().join(".cache"))
            .join("nvim"))
    }
}

#[allow(dead_code)]
pub fn backup_dir() -> Result<PathBuf> {
    Ok(data_root()?
        .parent()
        .context("data_root has no parent")?
        .join("nvimx")
        .join("backups"))
}

pub fn nvimx_config_dir() -> Result<PathBuf> {
    Ok(config_root()?
        .parent()
        .context("config_root has no parent")?
        .join("nvimx"))
}

pub fn profile_dir(name: &str) -> Result<PathBuf> {
    Ok(config_root()?.join(name))
}
