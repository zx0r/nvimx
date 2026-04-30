use crate::core::{config, output, registry};
use anyhow::Result;
use std::process::{Command, Stdio};

pub fn execute(repo_or_name: String, name: String, config: &config::Config) -> Result<()> {
    let reserved = [
        "list",
        "install",
        "clean",
        "doctor",
        "setup",
        "sandbox",
        "registry",
        "update",
        "completions",
        "help",
        "repro",
        "debug",
    ];
    if reserved.contains(&name.as_str()) {
        output::bad(&format!("Name '{}' is reserved", name));
        return Ok(());
    }

    let profiles_dir = config::get_profiles_dir(config);
    let dest = profiles_dir.join(&name);

    if dest.exists() {
        output::bad("Profile already exists");
        return Ok(());
    }

    let mut repo_url = repo_or_name.clone();

    // Registry resolution
    if !repo_url.contains("://") && !repo_url.contains('@') && !config.registry_urls.is_empty() {
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
        }
    }

    if !repo_url.contains("://") && !repo_url.contains('@') {
        repo_url = format!("https://github.com/{}", repo_url);
    }

    output::action(&format!("Fetching downloads for: {}", repo_url));

    let mut success = false;
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
                    dest.to_str().unwrap(),
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
            .args(["clone", "--depth", "1", &repo_url, dest.to_str().unwrap()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| {
                if s.success() {
                    success = true;
                }
            });
    }

    if success {
        output::success("done");
    } else {
        output::bad("Installation failed");
    }
    Ok(())
}
