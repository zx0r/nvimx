use crate::core::{config, output, picker};
use anyhow::{Context, Result, anyhow};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

/// Executes Neovim with optional profile resolution.
pub fn execute(
    first_arg: Option<String>,
    rest_args: Vec<String>,
    config: &config::Config,
) -> Result<()> {
    // Convert incoming Strings to OsStrings for internal processing
    let first_arg_os = first_arg.map(OsString::from);
    let rest_args_os: Vec<OsString> = rest_args.into_iter().map(OsString::from).collect();

    let profiles_dir = config::get_profiles_dir(config);

    // ─────────────────────────────────────────────────────────────────────
    // SMART RESOLUTION (0-overhead): Profile, file, or flag?
    // ─────────────────────────────────────────────────────────────────────
    let (profile_name, is_explicit, actual_args) = if let Some(arg) = first_arg_os {
        let arg_str = arg.to_string_lossy();
        let profile_path = profiles_dir.join(arg_str.as_ref());

        // If it is a valid profile name AND the directory exists -> it is a profile
        if is_valid_profile_name(&arg_str) && profile_path.is_dir() {
            (Some(arg_str.into_owned()), true, rest_args_os)
        } else {
            // Otherwise, it is a file (even if not created yet) or a flag.
            // Return the argument back to the args pool and use the default profile.
            let mut all_args = Vec::with_capacity(rest_args_os.len() + 1);
            all_args.push(arg);
            all_args.extend(rest_args_os);

            (config.default_profile.clone(), false, all_args)
        }
    } else {
        // Launch without arguments (bare nvimx)
        (config.default_profile.clone(), false, rest_args_os)
    };

    match profile_name {
        Some(name) => {
            let profile_path = profiles_dir.join(&name);

            if profile_path.exists() && profile_path.is_dir() {
                // HOT PATH: Profile found. Launch instantly.
                run_nvim(Some(&name), actual_args, ProfileFallback::Preserve)
            } else {
                // COLD PATH: Profile not found, apply fallback strategy
                handle_missing_profile(&name, is_explicit, config, actual_args)
            }
        }
        None => {
            // No profile specified explicitly or by default
            if config.use_fzf {
                run_fzf(&profiles_dir, actual_args)
            } else {
                run_nvim(None::<&str>, actual_args, ProfileFallback::ClearOnNone)
            }
        }
    }
}

/// Handles a missing profile according to settings.
fn handle_missing_profile(
    name: &str,
    is_explicit: bool,
    config: &config::Config,
    args: Vec<OsString>,
) -> Result<()> {
    if config.strict_mode {
        return Err(anyhow!(
            "profile '{}' not found (strict mode enabled)",
            name
        ));
    }

    if is_explicit {
        if config.fallback_to_plain_nvim {
            output::warning(&format!(
                "profile '{}' not found — falling back to vanilla nvim",
                name
            ));
            return run_nvim(None::<&str>, args, ProfileFallback::ClearOnNone);
        }
        return Err(anyhow!("profile '{}' not found", name));
    }

    if config.use_fzf {
        let profiles_dir = config::get_profiles_dir(config);
        return run_fzf(&profiles_dir, args);
    }

    if config.fallback_to_plain_nvim {
        output::warning(&format!(
            "default profile '{}' not found — using vanilla nvim",
            name
        ));
        run_nvim(None::<&str>, args, ProfileFallback::ClearOnNone)
    } else {
        Err(anyhow!("profile '{}' not found", name))
    }
}

/// Interactive profile selection via FZF.
fn run_fzf(profiles_dir: &Path, args: Vec<OsString>) -> Result<()> {
    if !profiles_dir.exists() {
        fs::create_dir_all(profiles_dir)
            .with_context(|| format!("failed to create profiles directory: {:?}", profiles_dir))?;
    }

    let entries: Vec<String> = fs::read_dir(profiles_dir)
        .with_context(|| format!("failed to read profiles directory: {:?}", profiles_dir))?
        .filter_map(Result::ok)
        .filter(|e| {
            // Optimization: use file_type() without stat syscall (on Unix)
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let name_str = e.file_name().to_string_lossy().into_owned();
            is_dir && !name_str.starts_with('.') && !name_str.starts_with('_')
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    if entries.is_empty() {
        output::info("no profiles found — launching vanilla nvim");
        return run_nvim(None::<&str>, args, ProfileFallback::ClearOnNone);
    }

    match picker::pick(entries) {
        Ok(Some(picked)) => run_nvim(Some(picked.as_str()), args, ProfileFallback::Preserve),
        Ok(None) => {
            output::info("picker cancelled");
            Ok(())
        }
        Err(e) => Err(e).context("fzf picker failed"),
    }
}

#[derive(Clone, Copy)]
enum ProfileFallback {
    ClearOnNone,
    Preserve,
}

/// Executes Neovim (Zero-overhead exec).
///
/// Uses generics to avoid allocations.
/// Returns an error ONLY if `exec()` fails to replace the process.
fn run_nvim<P, I, S>(profile: Option<P>, args: I, fallback: ProfileFallback) -> Result<()>
where
    P: AsRef<str>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("nvim");

    match (profile, fallback) {
        (Some(p), _) => {
            let p_ref = p.as_ref();
            if !is_valid_profile_name(p_ref) {
                return Err(anyhow!(
                    "refusing to launch with invalid profile name: '{}'",
                    p_ref
                ));
            }
            // Mapping to ~/.config/nvim/<profile>
            cmd.env("NVIM_APPNAME", format!("nvim/{}", p_ref));
        }
        (None, ProfileFallback::ClearOnNone) => {
            cmd.env_remove("NVIM_APPNAME");
        }
        (None, ProfileFallback::Preserve) => {}
    }

    // Passthrough arguments at the OS byte level
    cmd.args(args);

    // System call execvp(). Replaces nvimx process with nvim.
    // If successful, this line will NEVER return.
    let err = cmd.exec();

    Err(anyhow!(
        "failed to execute nvim: {}\n\
         Common causes: nvim not in PATH, permission denied, or invalid arguments",
        err
    ))
}

/// Strict profile name validation for path safety.
fn is_valid_profile_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_profile_names() {
        assert!(is_valid_profile_name("python"));
        assert!(is_valid_profile_name("rust-dev"));
        assert!(is_valid_profile_name("my_profile_2"));
        assert!(is_valid_profile_name("a"));
    }

    #[test]
    fn test_invalid_profile_names() {
        assert!(!is_valid_profile_name(""));
        assert!(!is_valid_profile_name("nvim/lazy")); // Protection against Path Traversal
        assert!(!is_valid_profile_name("profile name"));
        assert!(!is_valid_profile_name("profile.sh"));
        assert!(!is_valid_profile_name(&"a".repeat(65)));
    }
}
