use crate::core::output;
use anyhow::{Result, anyhow};
use std::env;
use std::fs;

#[derive(Clone, Copy)]
pub enum ShellType {
    Fish,
    Zsh,
    Bash,
}

pub fn detect_shell() -> Result<ShellType> {
    let shell_path = env::var("SHELL").unwrap_or_default();
    if shell_path.contains("fish") {
        Ok(ShellType::Fish)
    } else if shell_path.contains("zsh") {
        Ok(ShellType::Zsh)
    } else if shell_path.contains("bash") {
        Ok(ShellType::Bash)
    } else {
        Err(anyhow!("Unsupported shell: {}", shell_path))
    }
}

pub fn execute(
    override_nvim: bool,
    force_env: bool,
    print_env: bool,
    print: bool,
    dry_run: bool,
    remove: bool,
) -> Result<()> {
    let shell = detect_shell()?;

    let shell_name = match shell {
        ShellType::Fish => "fish",
        ShellType::Zsh => "zsh",
        ShellType::Bash => "bash",
    };

    let shell_display = match shell {
        ShellType::Fish => "Fish",
        ShellType::Zsh => "Zsh",
        ShellType::Bash => "Bash",
    };

    if remove {
        output::header(&format!("Uninstalling {} integration...", shell_display));
        return uninstall(shell);
    }

    if print {
        println!("{}", get_full_script(shell)?);
        return Ok(());
    }

    if print_env {
        println!("{}", get_env_block(shell)?);
        return Ok(());
    }

    output::header(&format!(
        "Setting up {} shell integration...",
        shell_display
    ));
    println!();

    install_completions(shell, dry_run)?;

    if override_nvim {
        install_full_integration(shell, dry_run, force_env)?;
    } else {
        output::info("Skipping nvim wrapper (use --override-nvim to install)");
    }

    if !dry_run {
        println!();
        output::success("done");
        println!();
        output::info(&format!("To apply changes:\n  exec {}", shell_name));
    }

    Ok(())
}

fn install_completions(shell: ShellType, dry_run: bool) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Home dir not found"))?;

    let (path, content) = match shell {
        ShellType::Fish => (
            home.join(".config/fish/completions/nvimx.fish"),
            include_str!("../../../completions/nvimx.fish"),
        ),
        ShellType::Zsh => (
            home.join(".zsh/completions/_nvimx"),
            include_str!("../../../completions/_nvimx"),
        ),
        ShellType::Bash => (
            home.join(".bash_completion.d/nvimx"),
            include_str!("../../../completions/nvimx.bash"),
        ),
    };

    if dry_run {
        output::info(&format!("dry-run → completions → {}", shorten_path(&path)));
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, content)?;
    output::success(&format!("completions → {}", shorten_path(&path)));

    Ok(())
}

fn install_full_integration(shell: ShellType, dry_run: bool, _force: bool) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Home dir not found"))?;
    let full_script = get_full_script(shell)?;

    let rc_path = match shell {
        ShellType::Fish => home.join(".config/fish/conf.d/nvim.fish"),
        ShellType::Zsh => home.join(".zshrc"),
        ShellType::Bash => home.join(".bashrc"),
    };

    if dry_run {
        output::info(&format!("dry-run → wrapper → {}", shorten_path(&rc_path)));
        return Ok(());
    }

    if matches!(shell, ShellType::Fish) {
        // if rc_path.exists() {
        //     output::info("Overwriting existing Fish wrapper");
        // }

        if let Some(parent) = rc_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&rc_path, &full_script)?;
    } else {
        let content = fs::read_to_string(&rc_path).unwrap_or_default();

        if !content.contains("nvim() {") {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&rc_path)?;

            use std::io::Write;
            write!(file, "\n{}", full_script)?;
        } else {
            output::info("Integration already exists in config (skipping)");
            return Ok(());
        }
    }

    output::success(&format!("nvim wrapper → {}", shorten_path(&rc_path)));

    Ok(())
}

fn get_env_block(shell: ShellType) -> Result<String> {
    match shell {
        ShellType::Fish => Ok(
            r#"# Neovim environment configuration (only if nvimx is available)
if type -q nvimx
    set -gx EDITOR nvim
    set -gx VISUAL nvim
    set -gx GIT_EDITOR nvim
    set -gx SUDO_EDITOR nvim
end"#
                .to_string(),
        ),
        ShellType::Zsh | ShellType::Bash => Ok(r#"# Environment
if command -v nvimx >/dev/null 2>&1; then
  export EDITOR=nvim
  export VISUAL=nvim
  export GIT_EDITOR=nvim
  export SUDO_EDITOR=nvim
fi"#
        .to_string()),
    }
}

fn get_wrapper(shell: ShellType) -> Result<String> {
    match shell {
        ShellType::Fish => Ok(r#"function nvim --wraps nvim
    if test -z "$NVIM_APPNAME"
        if test (count $argv) -gt 0; and string match -qr '^-' $argv[1]
            command nvim $argv
        else
            nvimx $argv
        end
    else
        command nvim $argv
    end
end"#
            .to_string()),

        ShellType::Zsh => Ok(r#"nvim() {
  if [[ -z "$NVIM_APPNAME" ]]; then
    if [[ $# -gt 0 && "$1" == -* ]]; then
      command nvim "$@"
    else
      nvimx "$@"
    fi
  else
    command nvim "$@"
  fi
}"#
        .to_string()),

        ShellType::Bash => Ok(r#"nvim() {
  if [[ -z "$NVIM_APPNAME" ]]; then
    if [[ $# -gt 0 && "$1" =~ ^- ]]; then
      command nvim "$@"
    else
      nvimx "$@"
    fi
  else
    command nvim "$@"
  fi
}"#
        .to_string()),
    }
}

fn get_full_script(shell: ShellType) -> Result<String> {
    Ok(format!(
        "{}\n\n{}",
        get_wrapper(shell)?,
        get_env_block(shell)?
    ))
}

fn uninstall(_shell: ShellType) -> Result<()> {
    output::info("Uninstall not fully implemented yet. Please remove manually.");
    Ok(())
}

fn shorten_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        path_str.replace(&home.to_string_lossy().into_owned(), "~")
    } else {
        path_str.into_owned()
    }
}
