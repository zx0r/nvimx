use crate::core::{config, output};
use anyhow::Result;

pub fn execute(plain: bool) -> Result<()> {
    let cfg = config::load();
    let profiles_dir = config::get_profiles_dir(&cfg);

    if !profiles_dir.exists() {
        if !plain {
            output::error("No profiles found");
        }
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(profiles_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
        .collect();

    if entries.is_empty() {
        if !plain {
            output::error("No profiles detected");
        }
        return Ok(());
    }

    entries.sort_by_key(|e| e.file_name());

    if !plain {
        output::header("Profiles");
    }

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if plain {
            println!("{}", name);
        } else {
            output::success(&name);
        }
    }

    if !plain {
        println!();
    }
    Ok(())
}
