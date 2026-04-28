use crate::core::output;
use anyhow::{Context, Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;
use serde::Deserialize;
use std::io::{Read, Write};
use std::process::Command;
use std::{env, fs};

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn is_brew_install() -> bool {
    let current_exe = env::current_exe().unwrap_or_default();
    let path_str = current_exe.display().to_string();
    path_str.contains("Cellar") || path_str.contains("homebrew")
}

pub fn execute() -> Result<()> {
    output::header("update");

    if is_brew_install() {
        output::success("nvimx is managed by homebrew");
        output::hint("run 'brew upgrade nvimx' to update");
        return Ok(());
    }

    let repo = "zx0r/nvimx";
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version_str)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("nvimx")
        .build()?;

    output::success("checking for updates...");
    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            repo
        ))
        .send()
        .context("failed to fetch release info from github")?
        .json()
        .context("failed to parse github api response")?;

    let latest_version_str = release.tag_name.trim_start_matches('v');
    let latest_version =
        Version::parse(latest_version_str).unwrap_or_else(|_| Version::new(0, 0, 0));

    if latest_version <= current_version {
        output::success(&format!("nvimx is up to date (v{})", current_version_str));
        return Ok(());
    }

    output::success(&format!(
        "new version available: v{} (current: v{})",
        latest_version_str, current_version_str
    ));

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x86_64",
        ("linux", "x86_64") => "linux-x86_64",
        _ => {
            return Err(anyhow!(
                "unsupported platform for auto-update: {}-{}",
                os,
                arch
            ));
        }
    };

    let asset_name = format!("nvimx-{}", target);
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow!(
                "could not find binary '{}' in the latest release",
                asset_name
            )
        })?;

    output::success("downloading...");
    let mut response = client.get(&asset.browser_download_url).send()?;
    let content_length = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(content_length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut bytes = Vec::with_capacity(content_length as usize);
    let mut buffer = [0; 8192];
    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
        pb.inc(n as u64);
    }
    pb.finish_with_message("download complete");

    output::success("installing...");
    let current_exe = env::current_exe()?;
    let tmp_path = current_exe.with_extension("tmp");

    let mut tmp_file = fs::File::create(&tmp_path)?;
    tmp_file.write_all(&bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_path, perms)?;
    }

    if fs::rename(&tmp_path, &current_exe).is_err() {
        let status = Command::new("sudo")
            .args([
                "mv",
                tmp_path.to_str().unwrap(),
                current_exe.to_str().unwrap(),
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow!("failed to replace binary. check permissions."));
        }
    }

    output::success(&format!("updated to v{}", latest_version_str));
    Ok(())
}
