use std::ffi::{OsStr, OsString};

use crate::error::{Error, Result};
use crate::layout::EnvLayout;
use crate::types::{Command, Config, LaunchPayload, ProfileId, RESERVED_COMMANDS, ShellKind};

/// Fast-path bypass for `-v`, `-V`, `--version`, `-h`, `--help`.
/// Operates with 0 filesystem syscalls and 0 allocations.
#[must_use]
pub fn try_fast_path(first_arg: Option<&OsStr>) -> Option<Command> {
    let arg = first_arg?.to_str()?;
    match arg {
        "-v" | "-V" | "--version" => Some(Command::Version),
        "-h" | "--help" => Some(Command::Help),
        _ => None,
    }
}

/// Parses CLI invocation arguments into an operational `Command`.
///
/// # Errors
/// Returns `Error` on invalid argument structure or reserved command violations.
pub fn parse_invocation(args: &[OsString], cfg: &Config) -> Result<Command> {
    if args.len() <= 1 {
        return Ok(Command::Launch(LaunchPayload {
            profile: cfg.default_profile.clone(),
            nvim_args: Vec::new(),
        }));
    }

    let first_arg = &args[1];
    let first_str = first_arg.to_string_lossy();

    // 1. Check flags & subcommands
    match first_str.as_ref() {
        "-v" | "-V" | "--version" => return Ok(Command::Version),
        "-h" | "--help" => return Ok(Command::Help),
        "repro" => return Ok(Command::ReservedAbort { name: "repro" }),
        "debug" => return Ok(Command::ReservedAbort { name: "debug" }),
        "list" | "ls" => {
            let plain = args[2..].iter().any(|a| a == "--plain");
            return Ok(Command::List { plain });
        }
        "install" => {
            return parse_install(&args[2..]);
        }
        "clean" => {
            return parse_clean(&args[2..]);
        }
        "init" => {
            let shell = args
                .get(2)
                .and_then(|s| s.to_str())
                .and_then(ShellKind::parse);
            return Ok(Command::Init { shell });
        }
        "sandbox" => {
            let target = args
                .get(2)
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            return Ok(Command::Sandbox { target });
        }
        _ => {}
    }

    // Check if user accidentally called a reserved identifier with arguments
    for &reserved in &RESERVED_COMMANDS {
        if first_str == reserved {
            return Ok(Command::ReservedAbort { name: reserved });
        }
    }

    // 2. Hot-Path Disambiguation (SPEC.md §2.1.2)
    let layout = EnvLayout::resolve(cfg, None);
    if let Ok(pid) = ProfileId::parse(&first_str)
        && layout.profile_exists(&pid)
    {
        return Ok(Command::Launch(LaunchPayload {
            profile: Some(pid),
            nvim_args: args[2..].to_vec(),
        }));
    }

    // 3. First argument is not an existing profile -> Treat as file/flag to default profile
    Ok(Command::Launch(LaunchPayload {
        profile: cfg.default_profile.clone(),
        nvim_args: args[1..].to_vec(),
    }))
}

fn parse_install(args: &[OsString]) -> Result<Command> {
    let mut positional = Vec::new();
    for arg in args {
        let s = arg.to_string_lossy();
        if !s.starts_with('-') {
            positional.push(s.to_string());
        }
    }

    if positional.len() < 2 {
        return Err(Error::Generic("usage: nvimx install <repo> <name>".into()));
    }

    let repo_or_preset = positional[0].clone();
    let name = positional[1].clone();

    // Invariant: name cannot be a reserved identifier
    for &reserved in &RESERVED_COMMANDS {
        if name == reserved {
            return Err(Error::ReservedNameUsed(name));
        }
    }

    let _ = ProfileId::parse(&name)?;

    Ok(Command::Install {
        repo_or_preset,
        name,
    })
}

fn parse_clean(args: &[OsString]) -> Result<Command> {
    let mut profile = String::new();
    let mut auto_confirm = false;

    for arg in args {
        let s = arg.to_string_lossy();
        if s == "--yes" || s == "-y" {
            auto_confirm = true;
        } else if !s.starts_with('-') && profile.is_empty() {
            profile = s.to_string();
        }
    }

    if profile.is_empty() {
        return Err(Error::Generic("usage: nvimx clean <profile> [-y]".into()));
    }

    Ok(Command::Clean {
        profile,
        auto_confirm,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_try_fast_path() {
        assert_eq!(
            try_fast_path(Some(OsStr::new("-v"))),
            Some(Command::Version)
        );
        assert_eq!(
            try_fast_path(Some(OsStr::new("--version"))),
            Some(Command::Version)
        );
        assert_eq!(try_fast_path(Some(OsStr::new("-h"))), Some(Command::Help));
        assert_eq!(
            try_fast_path(Some(OsStr::new("--help"))),
            Some(Command::Help)
        );
        assert_eq!(try_fast_path(Some(OsStr::new("help"))), None);
        assert_eq!(try_fast_path(Some(OsStr::new("run"))), None);
        assert_eq!(try_fast_path(None), None);
    }

    #[test]
    fn test_parse_init() {
        let cfg = Config::default_config();

        let init_default = parse_invocation(&["nvimx".into(), "init".into()], &cfg).unwrap();
        assert_eq!(init_default, Command::Init { shell: None });

        let init_fish =
            parse_invocation(&["nvimx".into(), "init".into(), "fish".into()], &cfg).unwrap();
        assert_eq!(
            init_fish,
            Command::Init {
                shell: Some(ShellKind::Fish)
            }
        );
    }

    #[test]
    fn test_parse_reserved_subcommands() {
        let cfg = Config::default_config();

        let repro_cmd = parse_invocation(&["nvimx".into(), "repro".into()], &cfg).unwrap();
        assert_eq!(repro_cmd, Command::ReservedAbort { name: "repro" });

        let debug_cmd = parse_invocation(&["nvimx".into(), "debug".into()], &cfg).unwrap();
        assert_eq!(debug_cmd, Command::ReservedAbort { name: "debug" });
    }

    #[test]
    fn test_parse_list() {
        let cfg = Config::default_config();

        let list_default = parse_invocation(&["nvimx".into(), "list".into()], &cfg).unwrap();
        assert_eq!(list_default, Command::List { plain: false });

        let list_plain =
            parse_invocation(&["nvimx".into(), "list".into(), "--plain".into()], &cfg).unwrap();
        assert_eq!(list_plain, Command::List { plain: true });

        let ls_default = parse_invocation(&["nvimx".into(), "ls".into()], &cfg).unwrap();
        assert_eq!(ls_default, Command::List { plain: false });

        let ls_plain =
            parse_invocation(&["nvimx".into(), "ls".into(), "--plain".into()], &cfg).unwrap();
        assert_eq!(ls_plain, Command::List { plain: true });
    }

    #[test]
    fn test_parse_clean() {
        let cfg = Config::default_config();

        let clean_cmd = parse_invocation(
            &[
                "nvimx".into(),
                "clean".into(),
                "lazyvim".into(),
                "-y".into(),
            ],
            &cfg,
        )
        .unwrap();
        assert_eq!(
            clean_cmd,
            Command::Clean {
                profile: "lazyvim".into(),
                auto_confirm: true,
            }
        );
    }

    #[test]
    fn test_parse_install_reserved_name() {
        let cfg = Config::default_config();

        let res = parse_invocation(
            &[
                "nvimx".into(),
                "install".into(),
                "repo".into(),
                "clean".into(),
            ],
            &cfg,
        );
        assert!(matches!(res, Err(Error::ReservedNameUsed(_))));
    }

    #[test]
    fn test_hot_path_disambiguation_file_vs_profile() {
        let cfg = Config::default_config();

        // An argument with a dot (like "main.rs") is not a valid profile name -> treated as file for default profile
        let cmd =
            parse_invocation(&["nvimx".into(), "main.rs".into(), "--clean".into()], &cfg).unwrap();

        assert_eq!(
            cmd,
            Command::Launch(LaunchPayload {
                profile: None,
                nvim_args: vec!["main.rs".into(), "--clean".into()],
            })
        );
    }
}
