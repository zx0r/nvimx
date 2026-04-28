use crate::core::{config, output, xdg};
use anyhow::Result;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Ok,
    Warn,
    Error,
}

#[derive(Serialize)]
struct DoctorReport {
    nvim: Status,
    git: Status,
    fzf: Status,
    config: Status,
    registry: Status,
    profiles: Status,
}

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

fn get_version(bin: &str) -> String {
    let output = Command::new(bin)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            let first_line = s.lines().next().unwrap_or("");
            let version = first_line
                .split_whitespace()
                .find(|w| w.chars().any(|c| c.is_numeric()))
                .unwrap_or(first_line)
                .to_string();

            if bin == "nvim" && !version.starts_with('v') {
                format!("v{}", version)
            } else {
                version
            }
        }
        Err(_) => "missing".to_string(),
    }
}

fn print_check(name: &str, status: Status, msg: &str, json: bool) {
    if json {
        return;
    }
    let icon = match status {
        Status::Ok => "✔".green().to_string(),
        Status::Warn => "⚠".yellow().to_string(),
        Status::Error => "✖".red().to_string(),
    };
    println!("  {} {:<16} {}", icon, name, msg);
}

pub fn execute(json: bool) -> Result<()> {
    let cfg = config::load();
    let mut report = DoctorReport {
        nvim: Status::Error,
        git: Status::Error,
        fzf: Status::Warn,
        config: Status::Ok,
        registry: Status::Error,
        profiles: Status::Warn,
    };

    if !json {
        output::header("Environment:");
    }

    let nvim_v = get_version("nvim");
    report.nvim = if nvim_v != "missing" {
        print_check("nvim", Status::Ok, &nvim_v, json);
        Status::Ok
    } else {
        print_check("nvim", Status::Error, "not installed", json);
        Status::Error
    };

    let git_v = get_version("git");
    report.git = if git_v != "missing" {
        print_check("git", Status::Ok, &git_v, json);
        Status::Ok
    } else {
        print_check("git", Status::Error, "not installed", json);
        Status::Error
    };

    report.fzf = if Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
    {
        print_check("fzf", Status::Ok, "found", json);
        Status::Ok
    } else {
        print_check("fzf", Status::Warn, "not found", json);
        Status::Warn
    };

    if !json {
        output::header("Configuration:");
    }

    let config_path = xdg::nvimx_config_dir()?.join("config.toml");
    if config_path.exists() {
        print_check("config", Status::Ok, &shorten_path(&config_path), json);
    } else {
        print_check("config", Status::Ok, "using default values", json);
    }

    if let Some(ref p) = cfg.default_profile {
        print_check("default_profile", Status::Ok, p, json);
    }

    if !json {
        output::header("Registry:");
    }

    report.registry = if let Some(url) = cfg.registry_urls.first() {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?;
        if client.get(url).send().is_ok() {
            print_check(
                "registry",
                Status::Ok,
                &format!("reachable ({})", url),
                json,
            );
            Status::Ok
        } else {
            print_check(
                "registry",
                Status::Error,
                &format!("unreachable ({})", url),
                json,
            );
            Status::Error
        }
    } else {
        print_check("registry", Status::Warn, "none", json);
        Status::Warn
    };

    if !json {
        output::header("Profiles:");
    }

    let profiles_dir = config::get_profiles_dir(&cfg);
    let count = fs::read_dir(&profiles_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
                .count()
        })
        .unwrap_or(0);

    if count > 0 {
        print_check(
            "profiles",
            Status::Ok,
            &format!("{} installed", count),
            json,
        );
        report.profiles = Status::Ok;
    } else {
        print_check("profiles", Status::Warn, "none", json);
        report.profiles = Status::Warn;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let has_error = [report.nvim, report.git, report.registry].contains(&Status::Error);
        let has_warn = [report.fzf, report.profiles].contains(&Status::Warn);

        let (icon, res_text) = if has_error {
            ("✖".red().to_string(), "error".red().bold().to_string())
        } else if has_warn {
            ("⚠".yellow().to_string(), "warn".yellow().bold().to_string())
        } else {
            ("✔".green().to_string(), "ok".green().bold().to_string())
        };

        println!("\n{} {} {}", "result:".bold(), icon, res_text);
    }

    Ok(())
}
