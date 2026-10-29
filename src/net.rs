use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};
use crate::types::{BUILTIN_PRESETS, Config, SemVer};

/// Resolves repository source through 3-tier hierarchy:
/// 1. Built-in presets (0 ms, offline) -> 2. User config presets -> 3. Raw source / GitHub slug.
#[must_use]
pub fn resolve_source_url<'a>(raw_source: &'a str, cfg: &'a Config) -> &'a str {
    // 1. Built-in presets
    for preset in BUILTIN_PRESETS {
        if preset.name.eq_ignore_ascii_case(raw_source) {
            return preset.repo;
        }
    }

    // 2. User-defined config presets
    for (name, repo) in &cfg.user_presets {
        if name.eq_ignore_ascii_case(raw_source) {
            return repo.as_str();
        }
    }

    // 3. Raw URL or GitHub slug
    raw_source
}

/// Normalizes source into a full git URL if it's a slug (`owner/repo`).
#[must_use]
pub fn normalize_repo_url(source: &str) -> String {
    if !source.contains("://") && !source.contains('@') {
        format!("https://github.com/{source}")
    } else {
        source.to_string()
    }
}

/// Clones git repository via `gh repo clone` with fallback to `git clone --depth 1`.
///
/// # Errors
/// Returns `Error::CommandFailed` or `Error::Io` if clone fails.
pub fn clone_repository(repo_url: &str, dest: &Path) -> Result<()> {
    let final_url = normalize_repo_url(repo_url);

    // Try gh repo clone first
    let gh_status = Command::new("gh")
        .args([
            "repo",
            "clone",
            &final_url,
            &dest.to_string_lossy(),
            "--",
            "--depth",
            "1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(status) = gh_status
        && status.success()
    {
        return Ok(());
    }

    // Fallback to git clone --depth 1
    let git_status = Command::new("git")
        .args(["clone", "--depth", "1", &final_url, &dest.to_string_lossy()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if git_status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed {
            cmd: format!("git clone {final_url}"),
            status: git_status.code().unwrap_or(1),
        })
    }
}

/// Parses a version string into `SemVer`.
///
/// # Errors
/// Returns `Error::VersionParseError` if string cannot be parsed into 3 numeric components.
pub fn parse_semver(raw: &str) -> Result<SemVer> {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);

    let mut parts = stripped.split('.');
    let major_str = parts.next().unwrap_or("");
    let minor_str = parts.next().unwrap_or("");
    let patch_raw = parts.next().unwrap_or("");

    // Strip pre-release or build metadata (e.g. "4-beta", "0+build")
    let patch_str = patch_raw.split(['-', '+']).next().unwrap_or("");

    let major = major_str
        .parse::<u32>()
        .map_err(|_| Error::VersionParseError(raw.to_string()))?;
    let minor = minor_str
        .parse::<u32>()
        .map_err(|_| Error::VersionParseError(raw.to_string()))?;
    let patch = patch_str
        .parse::<u32>()
        .map_err(|_| Error::VersionParseError(raw.to_string()))?;

    Ok(SemVer(major, minor, patch))
}

#[must_use]
pub const fn target_asset_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "nvimx-macos-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "nvimx-macos-x86_64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "nvimx-linux-x86_64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "nvimx-linux-arm64"
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64")
    )))]
    {
        "nvimx"
    }
}

/// Queries GitHub releases API via system curl and returns latest version and download URL.
///
/// # Errors
/// Returns `Error::SelfUpdateFailed` or `Error::CommandFailed` if query or parsing fails.
pub fn fetch_latest_release() -> Result<(SemVer, String)> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/zx0r/nvimx/releases/latest",
        ])
        .output()?;

    if !output.status.success() {
        return Err(Error::SelfUpdateFailed(
            "failed to query GitHub releases API".into(),
        ));
    }

    let body = String::from_utf8_lossy(&output.stdout);
    parse_github_release_json(&body)
}

/// Parses version tag and asset download URL from release JSON response.
///
/// # Errors
/// Returns `Error::SelfUpdateFailed` if required fields cannot be found.
pub fn parse_github_release_json(json: &str) -> Result<(SemVer, String)> {
    let tag = extract_json_string(json, "tag_name")
        .ok_or_else(|| Error::SelfUpdateFailed("missing 'tag_name' in release".into()))?;
    let version = parse_semver(&tag)?;

    let asset_name = target_asset_name();
    let url = extract_asset_download_url(json, asset_name).ok_or_else(|| {
        Error::SelfUpdateFailed(format!("no asset found for target '{asset_name}'"))
    })?;

    Ok((version, url))
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":");
    let key_pos = json.find(&pattern)?;
    let after_key = &json[key_pos + pattern.len()..];
    let first_quote = after_key.find('"')?;
    let content_start = first_quote + 1;
    let second_quote = after_key[content_start..].find('"')?;
    Some(after_key[content_start..content_start + second_quote].to_string())
}

fn extract_asset_download_url(json: &str, target_name: &str) -> Option<String> {
    let name_pattern = format!("\"name\": \"{target_name}\"");
    let name_pos = json.find(&name_pattern)?;
    let after_name = &json[name_pos..];

    let url_pattern = "\"browser_download_url\":";
    let url_pos = after_name.find(url_pattern)?;
    let after_url = &after_name[url_pos + url_pattern.len()..];
    let first_quote = after_url.find('"')?;
    let content_start = first_quote + 1;
    let second_quote = after_url[content_start..].find('"')?;
    Some(after_url[content_start..content_start + second_quote].to_string())
}

/// Downloads a binary payload directly to a local file destination using system curl.
///
/// # Errors
/// Returns `Error::CommandFailed` or `Error::Io` if download fails.
pub fn download_file(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "-o", &dest.to_string_lossy(), url])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed {
            cmd: format!("curl -fsSL -o {} {url}", dest.display()),
            status: status.code().unwrap_or(1),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_source_url_hierarchy() {
        let mut cfg = Config::default_config();
        cfg.user_presets
            .push(("custom".into(), "https://example.com/custom.git".into()));

        // 1. Built-in preset
        assert_eq!(
            resolve_source_url("lazyvim", &cfg),
            "https://github.com/LazyVim/starter"
        );
        assert_eq!(
            resolve_source_url("kickstart", &cfg),
            "https://github.com/nvim-lua/kickstart.nvim"
        );

        // 2. User config preset
        assert_eq!(
            resolve_source_url("custom", &cfg),
            "https://example.com/custom.git"
        );

        // 3. Fallback raw URL or slug
        assert_eq!(resolve_source_url("user/dotfiles", &cfg), "user/dotfiles");
        assert_eq!(
            resolve_source_url("https://gitlab.com/me/dots", &cfg),
            "https://gitlab.com/me/dots"
        );
    }

    #[test]
    fn test_normalize_repo_url() {
        assert_eq!(
            normalize_repo_url("user/repo"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            normalize_repo_url("https://github.com/foo/bar"),
            "https://github.com/foo/bar"
        );
        assert_eq!(
            normalize_repo_url("git@github.com:foo/bar.git"),
            "git@github.com:foo/bar.git"
        );
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("v0.10.4").unwrap(), SemVer(0, 10, 4));
        assert_eq!(parse_semver("2.48.1").unwrap(), SemVer(2, 48, 1));
        assert_eq!(parse_semver("v1.2.3-rc1").unwrap(), SemVer(1, 2, 3));
        assert_eq!(parse_semver("3.0.0+build123").unwrap(), SemVer(3, 0, 0));

        assert!(parse_semver("invalid").is_err());
        assert!(parse_semver("1.2").is_err());
        assert!(parse_semver("").is_err());
    }

    #[test]
    fn test_parse_github_release_json() {
        let asset = target_asset_name();
        let sample_json = format!(
            r#"{{
                "tag_name": "v0.2.0",
                "assets": [
                    {{
                        "name": "{asset}",
                        "browser_download_url": "https://github.com/zx0r/nvimx/releases/download/v0.2.0/{asset}"
                    }}
                ]
            }}"#
        );

        let (ver, url) = parse_github_release_json(&sample_json).unwrap();
        assert_eq!(ver, SemVer(0, 2, 0));
        assert_eq!(
            url,
            format!("https://github.com/zx0r/nvimx/releases/download/v0.2.0/{asset}")
        );
    }
}
