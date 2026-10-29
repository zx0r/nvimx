use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Built-in canonical Neovim starters. Embedded in .rodata: 0 syscalls, 0 ms latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPreset {
    pub name: &'static str,
    pub repo: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_PRESETS: &[BuiltinPreset] = &[
    BuiltinPreset {
        name: "lazyvim",
        repo: "https://github.com/LazyVim/starter",
        description: "Full-featured distribution based on lazy.nvim",
    },
    BuiltinPreset {
        name: "astronvim",
        repo: "https://github.com/AstroNvim/template",
        description: "Aesthetic, extensible distribution template",
    },
    BuiltinPreset {
        name: "nvchad",
        repo: "https://github.com/NvChad/starter",
        description: "Blazing fast Neovim config with custom UI",
    },
    BuiltinPreset {
        name: "kickstart",
        repo: "https://github.com/nvim-lua/kickstart.nvim",
        description: "Minimal, single-file starting point",
    },
];

/// List of 8 system-reserved identifiers that cannot be used as profile names.
pub const RESERVED_COMMANDS: [&str; 8] = [
    "list", "ls", "install", "clean", "init", "sandbox", "repro", "debug",
];

/// A guaranteed valid profile identifier (Parse, Don't Validate).
/// Invariants:
/// 1. Length 1..=64 bytes.
/// 2. Strictly ASCII `[a-zA-Z0-9_-]`.
/// 3. Cannot match any of `RESERVED_COMMANDS`.
/// 4. 0 path-traversal risk (no `.`, `/`, `\`, null bytes, control chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileId(String);

impl ProfileId {
    /// Parses and validates a profile name according to system invariants.
    ///
    /// # Errors
    /// Returns `Error::InvalidProfileName` if length is outside 1..=64 or contains forbidden chars.
    /// Returns `Error::ReservedNameUsed` if matching any of `RESERVED_COMMANDS`.
    pub fn parse(raw: &str) -> Result<Self> {
        let bytes = raw.as_bytes();
        if bytes.is_empty() || bytes.len() > 64 {
            return Err(Error::InvalidProfileName(raw.to_string()));
        }
        for &b in bytes {
            let valid = b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
            if !valid {
                return Err(Error::InvalidProfileName(raw.to_string()));
            }
        }
        for &reserved in &RESERVED_COMMANDS {
            if raw == reserved {
                return Err(Error::ReservedNameUsed(raw.to_string()));
            }
        }
        Ok(Self(raw.to_string()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ProfileId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Parsed operational command from CLI args.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Fast-path commands (0 syscalls, instant exit)
    Version,
    Help,

    /// Hot-path Neovim invocation
    Launch(LaunchPayload),

    /// Service subcommands
    List {
        plain: bool,
    },
    Install {
        repo_or_preset: String,
        name: String,
    },
    Clean {
        profile: String,
        auto_confirm: bool,
    },
    Init {
        shell: Option<ShellKind>,
    },
    Sandbox {
        target: String,
    },

    /// Diagnostic reserved identifiers (always exit code 1)
    ReservedAbort {
        name: &'static str,
    },
}

/// Payload required for Neovim process replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPayload {
    pub profile: Option<ProfileId>,
    pub nvim_args: Vec<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

impl ShellKind {
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }
}

/// Minimal in-memory configuration mapped from `config.toml`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub default_profile: Option<ProfileId>,
    pub use_fzf: bool,
    pub log_level: LogLevel,
    pub output_format: OutputFormat,
    pub auto_update: bool,
    pub auto_install_missing: bool,
    pub strict_mode: bool,
    pub fallback_to_plain_nvim: bool,
    pub profiles_dir: Option<PathBuf>,
    pub user_presets: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Parsed `SemVer` representation: (major, minor, patch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer(pub u32, pub u32, pub u32);

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_presets_integrity() {
        assert_eq!(BUILTIN_PRESETS.len(), 4);
        assert_eq!(BUILTIN_PRESETS[0].name, "lazyvim");
        assert_eq!(BUILTIN_PRESETS[1].name, "astronvim");
        assert_eq!(BUILTIN_PRESETS[2].name, "nvchad");
        assert_eq!(BUILTIN_PRESETS[3].name, "kickstart");
    }

    #[test]
    fn test_profile_id_valid() {
        let valid_names = ["dev", "my_profile", "test-123", "P1", "a", "0"];
        for name in valid_names {
            let pid = ProfileId::parse(name);
            assert!(pid.is_ok(), "Expected valid name: {name}");
            assert_eq!(pid.as_ref().map_or("", ProfileId::as_str), name);
        }
    }

    #[test]
    fn test_profile_id_invalid_chars() {
        let long_name = "a".repeat(65);
        let invalid_names = [
            "",
            "dev/profile",
            "../bad",
            "profile.name",
            "hello world",
            "special!",
            long_name.as_str(),
        ];
        for name in invalid_names {
            assert!(
                matches!(ProfileId::parse(name), Err(Error::InvalidProfileName(_))),
                "Expected InvalidProfileName for '{name}'"
            );
        }
    }

    #[test]
    fn test_profile_id_reserved_names() {
        for reserved in RESERVED_COMMANDS {
            assert!(
                matches!(ProfileId::parse(reserved), Err(Error::ReservedNameUsed(_))),
                "Expected ReservedNameUsed for '{reserved}'"
            );
        }
    }

    #[test]
    fn test_shell_kind_parse() {
        assert_eq!(ShellKind::parse("bash"), Some(ShellKind::Bash));
        assert_eq!(ShellKind::parse("zsh"), Some(ShellKind::Zsh));
        assert_eq!(ShellKind::parse("fish"), Some(ShellKind::Fish));
        assert_eq!(ShellKind::parse("unknown"), None);
    }

    #[test]
    fn test_semver_ord_and_display() {
        let v1 = SemVer(0, 1, 0);
        let v2 = SemVer(0, 2, 0);
        let v3 = SemVer(1, 0, 0);
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert_eq!(v1.to_string(), "v0.1.0");
    }
}
