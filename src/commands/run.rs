use crate::core::{config, output, picker};
use anyhow::{Result, anyhow};
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn execute(profile: Option<String>, args: Vec<String>, config: &config::Config) -> Result<()> {
    // Early check: if the first argument is a path that exists, treat it as a file/dir to open
    if let Some(p) = profile
        .as_ref()
        .filter(|p| std::path::Path::new(p).exists())
    {
        let mut all_args = vec![p.clone()];
        all_args.extend(args);
        return run_nvim(config.default_profile.clone(), all_args);
        // return run_nvim(None, all_args);
    }

    let profiles_dir = config::get_profiles_dir(config);

    let (profile_name, is_explicit) = if let Some(p) = profile {
        (Some(p), true)
    } else if let Some(ref p) = config.default_profile {
        (Some(p.clone()), false)
    } else {
        (None, false)
    };

    match profile_name {
        Some(name) => {
            let profile_path = profiles_dir.join(&name);
            if profile_path.exists() {
                run_nvim(Some(name), args)
            } else {
                if config.strict_mode {
                    return Err(anyhow!("profile '{}' not found", name));
                }

                if is_explicit || !config.use_fzf {
                    if config.fallback_to_plain_nvim {
                        output::warning(&format!("profile '{}' not found. falling back.", name));
                        run_nvim(None, args)
                    } else {
                        Err(anyhow!("profile '{}' not found", name))
                    }
                } else {
                    run_fzf(&profiles_dir, args)
                }
            }
        }
        None => {
            if config.use_fzf {
                run_fzf(&profiles_dir, args)
            } else {
                run_nvim(None, args)
            }
        }
    }
}

fn run_fzf(profiles_dir: &std::path::Path, args: Vec<String>) -> Result<()> {
    if !profiles_dir.exists() {
        fs::create_dir_all(profiles_dir)?;
    }

    let entries: Vec<String> = fs::read_dir(profiles_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if entries.is_empty() {
        return run_nvim(None, args);
    }

    if let Some(picked) = picker::pick(entries)? {
        run_nvim(Some(picked), args)
    } else {
        Ok(())
    }
}

fn run_nvim(profile: Option<String>, args: Vec<String>) -> Result<()> {
    let mut cmd = Command::new("nvim");
    if let Some(ref p) = profile {
        cmd.env("NVIM_APPNAME", format!("nvim/{}", p));
    }
    let err = cmd.args(&args).exec();
    Err(anyhow!("failed to execute 'nvim': {}", err))
}
