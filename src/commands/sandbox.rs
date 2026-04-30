use crate::core::{config, output, registry, xdg};
use anyhow::{Result, anyhow};
use std::fs;
use std::process::{Command, Stdio};

pub fn execute(profile: String, config: &config::Config) -> Result<()> {
    output::header(&format!("Sandbox: {}", profile));

    // 1. Prepare Sandbox Root
    let sandbox_root = xdg::cache_root()?.join("sandbox").join(&profile);

    // Standard XDG structure inside sandbox
    let config_home = sandbox_root.join("config");
    let data_home = sandbox_root.join("data");
    let state_home = sandbox_root.join("state");
    let cache_home = sandbox_root.join("cache");

    // Neovim specifically looks for 'nvim' folder inside XDG_CONFIG_HOME
    let nvim_config_dir = config_home.join("nvim");

    if sandbox_root.exists() {
        output::info("Wiping previous sandbox state...");
        let _ = fs::remove_dir_all(&sandbox_root);
    }

    fs::create_dir_all(&nvim_config_dir)?;
    fs::create_dir_all(&data_home)?;
    fs::create_dir_all(&state_home)?;
    fs::create_dir_all(&cache_home)?;

    // 2. Resolve and Clone Profile
    let mut repo_url = profile.clone();
    if !repo_url.contains("://") && !repo_url.contains('@') {
        let resolved = registry::fetch_smart(&config.registry_urls, false)
            .ok()
            .and_then(|reg| {
                reg.profiles
                    .into_iter()
                    .find(|p| p.name == repo_url)
                    .map(|p| p.repo)
            });

        if let Some(repo) = resolved {
            repo_url = repo;
            output::success(&format!("Resolved to: {}", repo_url));
        }
    }

    if !repo_url.contains("://") && !repo_url.contains('@') {
        repo_url = format!("https://github.com/{}", repo_url);
    }

    output::info("Cloning profile into sandbox...");
    let mut success = false;

    // Try gh then git
    let _ = Command::new("gh")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .and_then(|_| {
            Command::new("gh")
                .args([
                    "repo",
                    "clone",
                    &repo_url,
                    nvim_config_dir.to_str().unwrap(),
                    "--",
                    "--depth",
                    "1",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        })
        .map(|s| {
            if s.success() {
                success = true;
            }
        });

    if !success {
        let _ = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo_url,
                nvim_config_dir.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| {
                if s.success() {
                    success = true;
                }
            });
    }

    if !success {
        return Err(anyhow!("Failed to clone profile into sandbox"));
    }

    // 3. Launch Neovim with full isolation
    output::success(&format!("Launching Neovim (sandbox: {})", profile));

    let title_string = format!("nvimx sandbox: {}", profile);

    Command::new("nvim")
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XDG_DATA_HOME", &data_home)
        .env("XDG_STATE_HOME", &state_home)
        .env("XDG_CACHE_HOME", &cache_home)
        .env("NVIM_APPNAME", "nvim") // Standard name inside custom CONFIG_HOME
        .env("NVIMX_SANDBOX", &profile)
        .args([
            "--cmd",
            &format!("set title titlestring={}", title_string.replace(' ', "\\ ")),
        ])
        .status()
        .map_err(|e| anyhow!("Failed to execute 'nvim': {}", e))?;

    output::success("Sandbox session ended");
    Ok(())
}
