use std::ffi::OsString;
use std::process::{Command as ProcessCommand, ExitCode};

use nvimx::cli;
use nvimx::layout::{self, EnvLayout};
use nvimx::term::Terminal;
use nvimx::types::{Command, Config, LaunchPayload, ProfileId, ShellKind};

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();

    // 1. Fast-Path Bypass: -v / --version / -h / --help (0 syscalls, 0 FS reads)
    if let Some(cmd) = cli::try_fast_path(args.get(1).map(AsRef::as_ref)) {
        return execute_fast_path(&cmd);
    }

    // 2. Load Configuration & System Topology
    let bootstrap_config_dir = layout::resolve_bootstrap_config_dir();
    let config = Config::load(&bootstrap_config_dir);
    let term = Terminal::new();

    // 3. Parse Full Invocation (Subcommand or Hot-Path Launch)
    let command = match cli::parse_invocation(&args, &config) {
        Ok(cmd) => cmd,
        Err(err) => {
            term.error(&err.to_string());
            return err.exit_code();
        }
    };

    // 4. Central Dispatch
    match command {
        Command::Launch(ref payload) => handle_launch(payload, &config, term),
        Command::List { plain } => handle_list(plain, &config, term),
        Command::Clean {
            ref profile,
            auto_confirm,
        } => handle_clean(profile, auto_confirm, &config, term),
        Command::Init { shell } => handle_init(shell, &config, term),
        Command::Sandbox { ref target } => handle_sandbox(target, &config, term),
        Command::ReservedAbort { name } => {
            term.error(&format!(
                "'{name}' is a reserved command and cannot be used as a profile name"
            ));
            ExitCode::FAILURE
        }
        Command::Version | Command::Help => execute_fast_path(&command),
    }
}

fn execute_fast_path(cmd: &Command) -> ExitCode {
    match cmd {
        Command::Version => {
            println!("nvimx {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Help => {
            println!(
                "Zero-overhead Neovim profile manager\n\nUsage:\n  nvimx [PROFILE] [FILES...] [-- ARGS...]\n  nvimx <COMMAND> [FLAGS...]\n\nCommands:\n  list, ls     List all locally installed profiles\n  clean        Safely remove data, state, and cache for a profile\n  init         Output shell completions for fish, zsh, or bash\n  sandbox      Run a profile in an isolated ephemeral environment\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version"
            );
            ExitCode::SUCCESS
        }
        _ => ExitCode::FAILURE,
    }
}

fn handle_launch(payload: &LaunchPayload, cfg: &Config, term: Terminal) -> ExitCode {
    let layout = EnvLayout::resolve(cfg, None);

    let active_profile = if let Some(ref p) = payload.profile {
        if layout.profile_exists(p) {
            Some(p.clone())
        } else if cfg.strict_mode {
            term.error(&format!("profile '{}' not found", p.as_str()));
            return ExitCode::FAILURE;
        } else if cfg.fallback_to_plain_nvim {
            term.hint(&format!(
                "profile '{}' not found, falling back to plain nvim",
                p.as_str()
            ));
            None
        } else {
            term.error(&format!("profile '{}' not found", p.as_str()));
            return ExitCode::FAILURE;
        }
    } else if cfg.use_fzf {
        let profiles = layout.list_profiles().unwrap_or_default();
        if profiles.is_empty() {
            term.hint("No profiles detected");
            None
        } else {
            match term.select_profile_fzf(&profiles) {
                Ok(Some(chosen)) => Some(chosen),
                Ok(None) => return ExitCode::SUCCESS,
                Err(_) => None,
            }
        }
    } else {
        None
    };

    let target_layout = EnvLayout::resolve(cfg, active_profile.as_ref());
    if let Err(err) = target_layout.exec_nvim(&payload.nvim_args) {
        term.error(&err.to_string());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn handle_list(plain: bool, cfg: &Config, term: Terminal) -> ExitCode {
    let layout = EnvLayout::resolve(cfg, None);
    let profiles = match layout.list_profiles() {
        Ok(p) => p,
        Err(e) => {
            term.error(&e.to_string());
            return ExitCode::FAILURE;
        }
    };

    if plain {
        for p in profiles {
            println!("{}", p.as_str());
        }
    } else if profiles.is_empty() {
        term.hint("No profiles detected");
    } else {
        println!("\nProfiles");
        for p in profiles {
            term.success(p.as_str());
        }
        println!();
    }
    ExitCode::SUCCESS
}

fn handle_clean(profile_name: &str, auto_confirm: bool, cfg: &Config, term: Terminal) -> ExitCode {
    let layout = EnvLayout::resolve(cfg, None);
    let paths = layout.profile_storage_paths(profile_name);
    let existing: Vec<_> = paths.into_iter().filter(|p| p.exists()).collect();

    if existing.is_empty() {
        term.dimmed("nothing to clean");
        return ExitCode::SUCCESS;
    }

    println!("Cleaning {profile_name}...");
    println!("warning: the following directories will be deleted:");
    for p in &existing {
        println!("  • {}", layout::shrink_home_path(p));
    }
    println!();

    let confirmed = if auto_confirm {
        true
    } else if term.is_tty {
        term.prompt_confirm("Proceed? [y/N]: ").unwrap_or(false)
    } else {
        false
    };

    if !confirmed {
        term.dimmed("cleanup cancelled");
        return ExitCode::SUCCESS;
    }

    if term.is_tty {
        term.erase_lines(existing.len() + 3);
    }

    let labels = ["Config", "Data", "State", "Cache"];
    for (i, p) in existing.iter().enumerate() {
        let _ = std::fs::remove_dir_all(p);
        let label = labels.get(i).unwrap_or(&"Storage");
        term.success(&format!("{label} → {}", layout::shrink_home_path(p)));
    }
    println!();
    term.success(&format!("{profile_name} cleaned"));
    ExitCode::SUCCESS
}

fn handle_init(shell: Option<ShellKind>, cfg: &Config, term: Terminal) -> ExitCode {
    if let Some(sh) = shell {
        match sh {
            ShellKind::Fish => print!("{}", include_str!("../assets/completions/nvimx.fish")),
            ShellKind::Zsh => print!("{}", include_str!("../assets/completions/_nvimx")),
            ShellKind::Bash => print!("{}", include_str!("../assets/completions/nvimx.bash")),
        }
        return ExitCode::SUCCESS;
    }

    let nvim_ok = ProcessCommand::new("nvim")
        .arg("--version")
        .output()
        .is_ok();
    if !nvim_ok {
        term.error("missing essential dependency: nvim");
        return ExitCode::FAILURE;
    }

    let config_dir = layout::resolve_bootstrap_config_dir();
    let _ = std::fs::create_dir_all(&config_dir);

    let layout = EnvLayout::resolve(cfg, None);
    let _ = std::fs::create_dir_all(&layout.profiles_dir);

    term.success("nvimx initialized");
    ExitCode::SUCCESS
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if ft.is_file() {
            std::fs::copy(&from, &to)?;
        } else if ft.is_symlink() {
            #[cfg(unix)]
            {
                if let Ok(target) = std::fs::read_link(&from) {
                    let _ = std::os::unix::fs::symlink(target, &to);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = std::fs::copy(&from, &to);
            }
        }
    }
    Ok(())
}

fn handle_sandbox(target: &str, cfg: &Config, term: Terminal) -> ExitCode {
    let layout = EnvLayout::resolve(cfg, None);
    let target_name = if target.is_empty() { "default" } else { target };
    let sandbox_dir = layout.cache_root.join("sandbox").join(target_name);

    let _ = std::fs::remove_dir_all(&sandbox_dir);
    let config_dir = sandbox_dir.join("config").join("nvim");
    let data_dir = sandbox_dir.join("data");
    let state_dir = sandbox_dir.join("state");
    let cache_dir = sandbox_dir.join("cache");

    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        term.error(&format!("Failed to create sandbox: {e}"));
        return ExitCode::FAILURE;
    }
    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(&state_dir);
    let _ = std::fs::create_dir_all(&cache_dir);

    if target.is_empty() {
        println!("Sandbox: clean ephemeral environment");
    } else {
        let pid = match ProfileId::parse(target) {
            Ok(p) => p,
            Err(e) => {
                term.error(&e.to_string());
                return ExitCode::FAILURE;
            }
        };

        if !layout.profile_exists(&pid) {
            term.error(&format!("profile '{target}' not found"));
            return ExitCode::FAILURE;
        }

        let src_dir = layout.profiles_dir.join(pid.as_str());
        println!("Sandbox: {target}");
        term.success(&format!(
            "Cloning local profile from: {}",
            layout::shrink_home_path(&src_dir)
        ));

        if let Err(e) = copy_dir_all(&src_dir, &config_dir) {
            term.error(&format!("Sandbox copy failed: {e}"));
            return ExitCode::FAILURE;
        }
    }

    term.success(&format!("Launching Neovim (sandbox: {target_name})"));

    let status = ProcessCommand::new("nvim")
        .env("XDG_CONFIG_HOME", sandbox_dir.join("config"))
        .env("XDG_DATA_HOME", &data_dir)
        .env("XDG_STATE_HOME", &state_dir)
        .env("XDG_CACHE_HOME", &cache_dir)
        .env("NVIM_APPNAME", "nvim")
        .env("NVIMX_SANDBOX", target_name)
        .args([
            "--cmd",
            &format!("set title titlestring=nvimx\\ sandbox:\\ {target_name}"),
        ])
        .status();

    term.success("Sandbox session ended");
    status.map_or(ExitCode::FAILURE, |s| {
        ExitCode::from(s.code().unwrap_or(0) as u8)
    })
}
