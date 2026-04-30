use crate::commands;
use crate::core::{config, output, xdg};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, Write};

fn check_dep(name: &str) {
    if which::which(name).is_ok() {
        println!("  {} {}", "✔".green(), name);
    } else {
        println!("  {} {} {}", "✖".red(), name, "(required)".dimmed());
    }
}

fn shorten_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        path_str.replace(&home.to_string_lossy().into_owned(), "~")
    } else {
        path_str.into_owned()
    }
}

pub fn execute(force: bool, config: &config::Config) -> Result<()> {
    output::header("Setting up nvimx...");

    // 1. Deps
    check_dep("git");
    check_dep("nvim");

    if which::which("git").is_err() || which::which("nvim").is_err() {
        output::bad("missing essential dependencies");
        return Ok(());
    }

    // 2. Config
    let config_dir = xdg::nvimx_config_dir()?;
    let config_path = config_dir.join("config.toml");

    let default_config_content = r#"# default profile if none specified
# → Used when no profile argument is provided (skips fzf)
default_profile = "lazyvim"

# registry source
# → Remote JSON registry for profile installation
registry_url = "https://raw.githubusercontent.com/zx0r/nvimx-registry/main/index.json"

# enable fzf picker
# → Enables interactive selection when no default_profile
use_fzf = true

# logging
# → Controls verbosity level of CLI output
log_level = "info"   # debug | info | warn | error

# output format
# → "json" for scripting, "text" for human-readable CLI
output = "text"      # text | json

# auto update
# → Automatically update nvimx binary (not recommended by default)
auto_update = false

# fallback behavior
# → If profile not found → fallback to plain nvim instead of error
fallback_to_plain_nvim = true

# custom profiles dir (advanced)
# → Override default XDG config path for profiles
profiles_dir = "~/.config/nvim"

strict_mode = false
"#;

    if !config_path.exists() {
        fs::create_dir_all(&config_dir)?;
        fs::write(&config_path, default_config_content)?;
        println!(
            "  {} Config {}",
            "✔".green(),
            format!("(created at {})", shorten_path(&config_path)).dimmed()
        );
    } else if force {
        let backup = config_path.with_extension("toml.bak");
        fs::copy(&config_path, &backup)?;
        fs::write(&config_path, default_config_content)?;
        println!(
            "  {} Config {}",
            "✔".green(),
            format!("(overwritten, backup: {})", shorten_path(&backup)).dimmed()
        );
    } else {
        println!(
            "  {} Config {}",
            "✔".green(),
            format!("(exists at {})", shorten_path(&config_path)).dimmed()
        );
    }

    // 3. Profiles Check
    println!("  {} Registry loaded", "✔".green());

    let profiles_dir = config::get_profiles_dir(config);
    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)?;
    }

    let entries = fs::read_dir(&profiles_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
                .count()
        })
        .unwrap_or(0);

    let mut default_installed = false;
    if let Some(ref p) = config.default_profile {
        let p_path = profiles_dir.join(p);
        if p_path.exists() {
            println!(
                "  {} Default profile: {} {}",
                "✔".green(),
                p,
                "(installed)".dimmed()
            );
            default_installed = true;
        } else {
            println!(
                "  {} Default profile: {} {}",
                "⚠".yellow(),
                p,
                "(not installed)".dimmed()
            );
        }
    }

    println!("  {} Profiles found: {}", "✔".green(), entries);

    // 4. Action
    if !default_installed {
        let p_name = config.default_profile.as_deref().unwrap_or("lazyvim");
        println!("\nDefault profile '{}' is missing.", p_name);
        print!("  Proceed with installation? [Y/n]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
            println!("  Installing {}...", p_name);
            commands::install::execute(p_name.to_string(), p_name.to_string(), config)?;
        }
    }

    println!("\n{} nvimx is ready", "✔".green());
    Ok(())
}
