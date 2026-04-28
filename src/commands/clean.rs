use crate::core::{output, xdg};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn shorten_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(&*home_str) {
            return path_str.replacen(&*home_str, "~", 1);
        }
    }
    path_str.to_string()
}

pub fn execute(profile: String) -> Result<()> {
    let config_path = xdg::profile_dir(&profile)?;
    let data_path = xdg::data_root()?.join(&profile);
    let state_path = xdg::state_root()?.join(&profile);
    let cache_path = xdg::cache_root()?.join(&profile);

    let paths = [
        ("Config", config_path),
        ("Data", data_path),
        ("State", state_path),
        ("Cache", cache_path),
    ];

    let mut found_paths = Vec::new();
    for (label, path) in &paths {
        if path.exists() {
            found_paths.push((*label, path));
        }
    }

    if found_paths.is_empty() {
        output::info("nothing to clean");
        return Ok(());
    }

    // 1. Print Header
    output::header(&format!("Cleaning {}...", profile));

    // 2. Print Warning and Prompt
    output::warning("the following directories will be deleted:");
    for (_, path) in &found_paths {
        println!("  \x1b[31m•\x1b[0m {}", shorten_path(path));
    }
    println!();

    print!("Proceed? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        output::info("cleanup cancelled");
        return Ok(());
    }

    // 3. Clear only the warning/bullets/prompt block (N + 3 lines)
    let lines_to_clear = found_paths.len() + 3;
    for _ in 0..lines_to_clear {
        print!("\x1b[A\x1b[2K");
    }
    io::stdout().flush()?;

    // 4. Actual Deletion and Execution Results
    for (label, path) in found_paths {
        match fs::remove_dir_all(path) {
            Ok(_) => output::success(&format!("{} → {}", label, shorten_path(path))),
            Err(e) => output::bad(&format!("{}: {}", label, e)),
        }
    }

    println!("\n{} {} cleaned", "✔".green(), profile);
    Ok(())
}
