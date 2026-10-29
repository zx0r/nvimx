use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

use crate::error::Result;
use crate::types::ProfileId;

#[derive(Debug, Clone, Copy)]
pub struct Terminal {
    pub is_tty: bool,
    pub color_enabled: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    #[must_use]
    pub fn new() -> Self {
        let is_tty = io::stdout().is_terminal();
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let term_dumb = std::env::var("TERM").is_ok_and(|t| t == "dumb");
        let color_enabled = is_tty && !no_color && !term_dumb;

        Self {
            is_tty,
            color_enabled,
        }
    }

    #[must_use]
    pub const fn with_capabilities(is_tty: bool, color_enabled: bool) -> Self {
        Self {
            is_tty,
            color_enabled,
        }
    }

    pub fn cyan_arrow(&self, msg: &str) {
        if self.color_enabled {
            println!("\x1b[1;36m==>\x1b[0m {msg}");
        } else {
            println!("==> {msg}");
        }
    }

    pub fn success(&self, msg: &str) {
        if self.color_enabled {
            println!("\x1b[32m✔\x1b[0m {msg}");
        } else {
            println!("✔ {msg}");
        }
    }

    pub fn error(&self, msg: &str) {
        if self.color_enabled {
            eprintln!("\x1b[31m✘\x1b[0m {msg}");
        } else {
            eprintln!("✘ {msg}");
        }
    }

    pub fn hint(&self, msg: &str) {
        if self.color_enabled {
            eprintln!("\x1b[33mhint:\x1b[0m {msg}");
        } else {
            eprintln!("hint: {msg}");
        }
    }

    pub fn dimmed(&self, msg: &str) {
        if self.color_enabled {
            println!("\x1b[2m{msg}\x1b[0m");
        } else {
            println!("{msg}");
        }
    }

    /// Prompts user for interactive confirmation.
    ///
    /// # Errors
    /// Returns `Error::Io` on read or flush errors.
    pub fn prompt_confirm(&self, prompt_text: &str) -> Result<bool> {
        let mut stdout = io::stdout();
        write!(stdout, "{prompt_text}")?;
        stdout.flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        Ok(trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes"))
    }

    pub fn erase_lines(&self, count: usize) {
        if !self.is_tty {
            return;
        }
        let mut stdout = io::stdout();
        for _ in 0..count {
            let _ = write!(stdout, "\x1b[A\x1b[2K");
        }
        let _ = stdout.flush();
    }

    /// Interactively selects a profile using `fzf`.
    ///
    /// # Errors
    /// Returns `Error::Io` if spawning or communicating with `fzf` fails.
    pub fn select_profile_fzf(&self, profiles: &[ProfileId]) -> Result<Option<ProfileId>> {
        let mut child = Command::new("fzf")
            .args([
                "--header",
                "Select Neovim Profile (ESC to cancel)",
                "--height",
                "40%",
                "--layout",
                "reverse",
                "--border",
                "--color",
                "header:italic:cyan,pointer:bold:green,hl:bold:yellow",
                "--prompt",
                "λ ",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            for profile in profiles {
                writeln!(stdin, "{}", profile.as_str())?;
            }
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selected.is_empty() {
                Ok(None)
            } else {
                Ok(ProfileId::parse(&selected).ok())
            }
        } else {
            self.dimmed("picker cancelled");
            Ok(None)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_term_constructors() {
        let term = Terminal::with_capabilities(false, false);
        assert!(!term.is_tty);
        assert!(!term.color_enabled);

        let term_color = Terminal::with_capabilities(true, true);
        assert!(term_color.is_tty);
        assert!(term_color.color_enabled);
    }

    #[test]
    fn test_erase_lines_suppression_on_non_tty() {
        let term = Terminal::with_capabilities(false, false);
        // On non-tty this must be a no-op and not panic
        term.erase_lines(5);
    }
}
