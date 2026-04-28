use crate::cli::{CacheCommands, RegistryCommands};
use crate::core::{config, output, registry};
use anyhow::Result;

pub fn execute(command: RegistryCommands, config: &config::Config) -> Result<()> {
    match command {
        RegistryCommands::List => {
            output::header("Registries:");
            for url in &config.registry_urls {
                println!("  • {}", url);
            }
        }
        RegistryCommands::Check => {
            output::header("Registry Status:");
            for url in &config.registry_urls {
                match registry::fetch_raw(url) {
                    Ok(_) => output::success(&format!("{} reachable", url)),
                    Err(e) => output::bad(&format!("{} failed ({})", url, e)),
                }
            }
        }
        RegistryCommands::Update => {
            output::header("Updating Registry:");
            match registry::fetch_smart(&config.registry_urls, true) {
                Ok(_) => output::success("cache updated"),
                Err(e) => output::bad(&format!("failed to update: {}", e)),
            }
        }
        RegistryCommands::Cache { command } => match command {
            CacheCommands::Clear => {
                registry::clear_cache()?;
                output::success("registry cache cleared");
            }
        },
    }
    println!();
    Ok(())
}
