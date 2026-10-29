use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::types::{Config, ProfileId};

/// Complete environment topology and computed Neovim variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvLayout {
    pub config_root: PathBuf,
    pub data_root: PathBuf,
    pub state_root: PathBuf,
    pub cache_root: PathBuf,
    pub system_config: PathBuf,
    pub profiles_dir: PathBuf,
    /// Calculated `XDG_CONFIG_HOME` for custom `profiles_dir` (§2.2)
    pub xdg_config_home: Option<PathBuf>,
    /// Calculated `NVIM_APPNAME` ("nvim/<profile>")
    pub nvim_appname: Option<String>,
}

#[must_use]
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
    }
}

#[must_use]
pub fn expand_tilde(path: &Path, home: Option<&Path>) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~")
        && let Some(h) = home
    {
        return h.join(stripped);
    }
    path.to_path_buf()
}

#[must_use]
pub fn shrink_home_path(path: &Path) -> String {
    if let Some(h) = home_dir()
        && let Ok(stripped) = path.strip_prefix(&h)
    {
        return format!("~{}{}", std::path::MAIN_SEPARATOR, stripped.display());
    }
    path.display().to_string()
}

#[must_use]
pub fn resolve_bootstrap_config_dir() -> PathBuf {
    if let Some(val) = std::env::var_os("XDG_CONFIG_HOME")
        && !val.is_empty()
    {
        return PathBuf::from(val).join("nvimx");
    }
    if let Some(h) = home_dir() {
        return h.join(".config").join("nvimx");
    }
    PathBuf::from(".config/nvimx")
}

impl EnvLayout {
    /// Computes unified XDG and `NVIM_APPNAME` layout strictly following `SPEC.md` sections 1.1.1 and 2.2.
    #[must_use]
    pub fn resolve(cfg: &Config, profile: Option<&ProfileId>) -> Self {
        let home = home_dir();

        let config_root = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(
                || {
                    home.as_ref().map_or_else(
                        || PathBuf::from(".config/nvim"),
                        |h| h.join(".config").join("nvim"),
                    )
                },
                |val| PathBuf::from(val).join("nvim"),
            );

        let data_root = std::env::var_os("XDG_DATA_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(
                || {
                    home.as_ref().map_or_else(
                        || PathBuf::from(".local/share/nvim"),
                        |h| h.join(".local").join("share").join("nvim"),
                    )
                },
                |val| PathBuf::from(val).join("nvim"),
            );

        let state_root = std::env::var_os("XDG_STATE_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(
                || {
                    home.as_ref().map_or_else(
                        || PathBuf::from(".local/state/nvim"),
                        |h| h.join(".local").join("state").join("nvim"),
                    )
                },
                |val| PathBuf::from(val).join("nvim"),
            );

        let cache_root = std::env::var_os("XDG_CACHE_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(
                || {
                    home.as_ref().map_or_else(
                        || PathBuf::from(".cache/nvim"),
                        |h| h.join(".cache").join("nvim"),
                    )
                },
                |val| PathBuf::from(val).join("nvim"),
            );

        let system_config = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(
                || {
                    home.as_ref().map_or_else(
                        || PathBuf::from(".config/nvimx"),
                        |h| h.join(".config").join("nvimx"),
                    )
                },
                |val| PathBuf::from(val).join("nvimx"),
            );

        let profiles_dir = cfg.profiles_dir.as_ref().map_or_else(
            || config_root.clone(),
            |raw_p| expand_tilde(raw_p, home.as_deref()),
        );

        let is_custom_profiles_dir = profiles_dir != config_root;

        let (xdg_config_home, nvim_appname) = if is_custom_profiles_dir {
            let base_dir = profiles_dir
                .parent()
                .map_or_else(|| PathBuf::from("/"), ToOwned::to_owned);
            let folder_name = profiles_dir
                .file_name()
                .map_or_else(|| "nvim".into(), |f| f.to_string_lossy().to_string());

            let appname = profile.map(|p| format!("{folder_name}/{}", p.as_str()));
            (Some(base_dir), appname)
        } else {
            let appname = profile.map(|p| format!("nvim/{}", p.as_str()));
            (None, appname)
        };

        Self {
            config_root,
            data_root,
            state_root,
            cache_root,
            system_config,
            profiles_dir,
            xdg_config_home,
            nvim_appname,
        }
    }

    /// Resolves the 4 isolated XDG paths for a profile (Config, Data, State, Cache).
    #[must_use]
    pub fn profile_storage_paths(&self, profile: &str) -> [PathBuf; 4] {
        [
            self.profiles_dir.join(profile),
            self.data_root.join(profile),
            self.state_root.join(profile),
            self.cache_root.join(profile),
        ]
    }

    /// Replaces the current process with `nvim` via POSIX execvp(3). Never returns on success.
    ///
    /// # Errors
    /// Returns `Error::Io` if `execvp` fails to execute.
    #[cfg(unix)]
    pub fn exec_nvim(&self, args: &[OsString]) -> Result<()> {
        use std::os::unix::process::CommandExt;
        use std::process::Command;

        let mut cmd = Command::new("nvim");
        cmd.args(args);

        if let Some(ref xdg) = self.xdg_config_home {
            cmd.env("XDG_CONFIG_HOME", xdg);
        }
        if let Some(ref app) = self.nvim_appname {
            cmd.env("NVIM_APPNAME", app);
        } else {
            cmd.env_remove("NVIM_APPNAME");
        }

        let err = cmd.exec();
        Err(crate::error::Error::Io(err))
    }

    /// Spawns `nvim` process on non-Unix systems.
    ///
    /// # Errors
    /// Returns `Error::Io` if process fails to spawn or exit.
    #[cfg(not(unix))]
    pub fn exec_nvim(&self, args: &[OsString]) -> Result<()> {
        use std::process::Command;

        let mut cmd = Command::new("nvim");
        cmd.args(args);

        if let Some(ref xdg) = self.xdg_config_home {
            cmd.env("XDG_CONFIG_HOME", xdg);
        }
        if let Some(ref app) = self.nvim_appname {
            cmd.env("NVIM_APPNAME", app);
        } else {
            cmd.env_remove("NVIM_APPNAME");
        }

        let status = cmd.status()?;
        if status.success() {
            std::process::exit(0);
        }
        std::process::exit(status.code().unwrap_or(1));
    }

    /// Atomically replaces target binary, safely handling cross-device links (`EXDEV`).
    ///
    /// # Errors
    /// Returns `Error::Io` if renaming/copying fails.
    pub fn atomic_replace_exe(current_exe: &Path, temp_download: &Path) -> Result<()> {
        if fs::rename(temp_download, current_exe).is_err() {
            fs::copy(temp_download, current_exe)?;
            let _ = fs::remove_file(temp_download);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(current_exe, fs::Permissions::from_mode(0o755));
        }

        Ok(())
    }

    /// Scans `profiles_dir` returning sorted valid profile identifiers.
    ///
    /// # Errors
    /// Returns `Error::Io` if directory read fails.
    pub fn list_profiles(&self) -> Result<Vec<ProfileId>> {
        if !self.profiles_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        let entries = fs::read_dir(&self.profiles_dir)?;
        for entry in entries {
            let entry = entry?;
            let ft = entry.file_type()?;
            if !ft.is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if name_str.starts_with('.') || name_str.starts_with('_') {
                continue;
            }
            if let Ok(pid) = ProfileId::parse(&name_str) {
                profiles.push(pid);
            }
        }
        profiles.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Ok(profiles)
    }

    /// Checks if a profile configuration directory exists on disk.
    #[must_use]
    pub fn profile_exists(&self, profile: &ProfileId) -> bool {
        self.profiles_dir.join(profile.as_str()).is_dir()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::{LogLevel, OutputFormat};

    fn dummy_config(profiles_dir: Option<PathBuf>) -> Config {
        Config {
            default_profile: None,
            use_fzf: true,
            log_level: LogLevel::Info,
            output_format: OutputFormat::Text,
            auto_update: false,
            auto_install_missing: true,
            strict_mode: false,
            fallback_to_plain_nvim: true,
            profiles_dir,
            user_presets: Vec::new(),
        }
    }

    #[test]
    fn test_expand_tilde() {
        let home = Path::new("/custom/home");
        assert_eq!(
            expand_tilde(Path::new("~"), Some(home)),
            PathBuf::from("/custom/home")
        );
        assert_eq!(
            expand_tilde(Path::new("~/configs/nvim"), Some(home)),
            PathBuf::from("/custom/home/configs/nvim")
        );
        assert_eq!(
            expand_tilde(Path::new("/etc/nvim"), Some(home)),
            PathBuf::from("/etc/nvim")
        );
    }

    #[test]
    fn test_default_layout_resolution() {
        let cfg = dummy_config(None);
        let pid = ProfileId::parse("lazyvim").unwrap();
        let layout = EnvLayout::resolve(&cfg, Some(&pid));

        assert_eq!(layout.xdg_config_home, None);
        assert_eq!(layout.nvim_appname, Some("nvim/lazyvim".to_string()));
        assert_eq!(layout.profiles_dir, layout.config_root);

        let vanilla_layout = EnvLayout::resolve(&cfg, None);
        assert_eq!(vanilla_layout.xdg_config_home, None);
        assert_eq!(vanilla_layout.nvim_appname, None);
    }

    #[test]
    fn test_custom_profiles_dir_layout_resolution() {
        let custom_dir = PathBuf::from("/Volumes/Data/my-dots/nvim-profiles");
        let cfg = dummy_config(Some(custom_dir.clone()));
        let pid = ProfileId::parse("kickstart").unwrap();
        let layout = EnvLayout::resolve(&cfg, Some(&pid));

        assert_eq!(
            layout.xdg_config_home,
            Some(PathBuf::from("/Volumes/Data/my-dots"))
        );
        assert_eq!(
            layout.nvim_appname,
            Some("nvim-profiles/kickstart".to_string())
        );
        assert_eq!(layout.profiles_dir, custom_dir);
    }

    #[test]
    fn test_profile_storage_paths() {
        let cfg = dummy_config(None);
        let layout = EnvLayout::resolve(&cfg, None);
        let paths = layout.profile_storage_paths("testprofile");

        assert_eq!(paths[0], layout.profiles_dir.join("testprofile"));
        assert_eq!(paths[1], layout.data_root.join("testprofile"));
        assert_eq!(paths[2], layout.state_root.join("testprofile"));
        assert_eq!(paths[3], layout.cache_root.join("testprofile"));
    }

    #[test]
    fn test_list_profiles_and_exists() {
        let temp_dir =
            std::env::temp_dir().join(format!("nvimx_test_profiles_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let _ = fs::create_dir_all(temp_dir.join("beta"));
        let _ = fs::create_dir_all(temp_dir.join("alpha"));
        let _ = fs::create_dir_all(temp_dir.join(".hidden"));
        let _ = fs::create_dir_all(temp_dir.join("_internal"));
        let _ = fs::write(temp_dir.join("regular_file.txt"), b"ignore");

        let cfg = dummy_config(Some(temp_dir.clone()));
        let layout = EnvLayout::resolve(&cfg, None);

        let profiles = layout.list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].as_str(), "alpha");
        assert_eq!(profiles[1].as_str(), "beta");

        let alpha_pid = ProfileId::parse("alpha").unwrap();
        assert!(layout.profile_exists(&alpha_pid));

        let gamma_pid = ProfileId::parse("gamma").unwrap();
        assert!(!layout.profile_exists(&gamma_pid));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_atomic_replace_exe() {
        let temp_dir =
            std::env::temp_dir().join(format!("nvimx_test_atomic_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let target_exe = temp_dir.join("binary");
        let temp_exe = temp_dir.join(".binary.tmp");

        fs::write(&target_exe, b"version 1").unwrap();
        fs::write(&temp_exe, b"version 2").unwrap();

        EnvLayout::atomic_replace_exe(&target_exe, &temp_exe).unwrap();

        assert_eq!(fs::read(&target_exe).unwrap(), b"version 2");
        assert!(!temp_exe.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
