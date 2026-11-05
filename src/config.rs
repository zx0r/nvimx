use std::fs;
use std::path::{Path, PathBuf};

use crate::layout::{expand_tilde, home_dir};
use crate::types::{Config, LogLevel, OutputFormat, ProfileId};

impl Config {
    #[must_use]
    pub const fn default_config() -> Self {
        Self {
            default_profile: None,
            use_fzf: true,
            log_level: LogLevel::Info,
            output_format: OutputFormat::Text,
            strict_mode: false,
            fallback_to_plain_nvim: true,
            profiles_dir: None,
        }
    }

    /// Loads configuration from `system_config_dir / config.toml`.
    /// Returns default configuration if file is missing or unreadable.
    #[must_use]
    pub fn load(system_config_dir: &Path) -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let config_file = system_config_dir.join("config.toml");
        fs::read_to_string(&config_file).map_or_else(
            |_| Self::default_config(),
            |content| Self::load_from_str(&content, &home),
        )
    }

    /// Zero-allocation line parser for `config.toml`. Fills defaults on syntax errors.
    #[must_use]
    pub fn load_from_str(content: &str, home: &Path) -> Self {
        let mut cfg = Self::default_config();

        for raw_line in content.lines() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let Some((raw_key, raw_val)) = line.split_once('=') else {
                continue;
            };

            let key = raw_key.trim();
            let val = strip_quotes(raw_val.trim());

            match key {
                "default_profile" => {
                    cfg.default_profile = if val.is_empty() {
                        None
                    } else {
                        ProfileId::parse(val).ok()
                    };
                }
                "use_fzf" => {
                    if let Ok(b) = val.parse::<bool>() {
                        cfg.use_fzf = b;
                    }
                }
                "log_level" => match val {
                    "debug" => cfg.log_level = LogLevel::Debug,
                    "info" => cfg.log_level = LogLevel::Info,
                    "warn" => cfg.log_level = LogLevel::Warn,
                    "error" => cfg.log_level = LogLevel::Error,
                    _ => {}
                },
                "output" | "output_format" => match val {
                    "text" => cfg.output_format = OutputFormat::Text,
                    "json" => cfg.output_format = OutputFormat::Json,
                    _ => {}
                },
                "strict_mode" => {
                    if let Ok(b) = val.parse::<bool>() {
                        cfg.strict_mode = b;
                    }
                }
                "fallback_to_plain_nvim" => {
                    if let Ok(b) = val.parse::<bool>() {
                        cfg.fallback_to_plain_nvim = b;
                    }
                }
                "profiles_dir" if !val.is_empty() => {
                    let path = PathBuf::from(val);
                    cfg.profiles_dir = Some(expand_tilde(&path, Some(home)));
                }
                _ => {}
            }
        }

        cfg
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut quote_char = ' ';

    for (idx, ch) in line.char_indices() {
        if (ch == '"' || ch == '\'') && !in_quote {
            in_quote = true;
            quote_char = ch;
        } else if in_quote && ch == quote_char {
            in_quote = false;
        } else if ch == '#' && !in_quote {
            return &line[..idx];
        }
    }
    line
}

fn strip_quotes(val: &str) -> &str {
    let trimmed = val.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            ""
        }
    } else {
        trimmed
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_str_full() {
        let toml_data = r#"
            # Global config
            default_profile = "astronvim"
            use_fzf = false
            log_level = "debug"
            output = "json"
            strict_mode = true
            fallback_to_plain_nvim = false
            profiles_dir = "~/my_nvim_profiles"
        "#;

        let home = Path::new("/home/user");
        let cfg = Config::load_from_str(toml_data, home);

        assert_eq!(
            cfg.default_profile,
            Some(ProfileId::parse("astronvim").unwrap())
        );
        assert!(!cfg.use_fzf);
        assert_eq!(cfg.log_level, LogLevel::Debug);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.strict_mode);
        assert!(!cfg.fallback_to_plain_nvim);
        assert_eq!(
            cfg.profiles_dir,
            Some(PathBuf::from("/home/user/my_nvim_profiles"))
        );
    }

    #[test]
    fn test_comments_and_quotes() {
        let toml_data = r#"
            # Full line comment
            default_profile = "lazyvim" # Trailing comment
            output_format = 'text' # single quote test
        "#;
        let home = Path::new("/home/user");
        let cfg = Config::load_from_str(toml_data, home);

        assert_eq!(
            cfg.default_profile,
            Some(ProfileId::parse("lazyvim").unwrap())
        );
        assert_eq!(cfg.output_format, OutputFormat::Text);
    }

    #[test]
    fn test_corrupted_toml_resilience() {
        let bad_toml = r"
            default_profile = !invalid syntax!
            use_fzf = not_a_boolean
            log_level = 12345
            output = [1, 2, 3]
            something_corrupt
        ";
        let home = Path::new("/home/user");
        let cfg = Config::load_from_str(bad_toml, home);

        // All fields retain default values safely without panicking
        assert_eq!(cfg.default_profile, None);
        assert!(cfg.use_fzf);
        assert_eq!(cfg.log_level, LogLevel::Info);
        assert_eq!(cfg.output_format, OutputFormat::Text);
    }

    #[test]
    fn test_load_missing_file() {
        let missing_path = Path::new("/non/existent/path/for/config");
        let cfg = Config::load(missing_path);
        assert_eq!(cfg, Config::default_config());
    }
}
