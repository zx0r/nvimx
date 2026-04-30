mod cli;
mod commands;
mod core;
mod setup;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, SetupCommands};
use core::config;
use core::output;

fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    println!("nvimx v{}", version);
    println!(
        "platform: {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("build: {}", build_type);
}

fn main() {
    let config = config::load();

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let args: Vec<String> = std::env::args().collect();
            if args.len() >= 2 && (args[1] == "-v" || args[1] == "-V" || args[1] == "--version") {
                print_version();
                std::process::exit(0);
            }
            match e.kind() {
                clap::error::ErrorKind::UnknownArgument => {
                    let flag = args
                        .iter()
                        .find(|a| a.starts_with('-'))
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    output::bad(&format!("unknown option '{}'", flag));
                    output::hint("run 'nvimx --help'");
                }
                clap::error::ErrorKind::DisplayHelp => {
                    let _ = e.print();
                    std::process::exit(0);
                }
                clap::error::ErrorKind::DisplayVersion => {
                    print_version();
                    std::process::exit(0);
                }
                _ => {
                    output::bad("invalid usage");
                    output::hint("run 'nvimx --help'");
                }
            }
            std::process::exit(1);
        }
    };

    if let Err(err) = run(cli, config) {
        output::bad(&format!("{}", err));
        std::process::exit(1);
    }
}

fn run(cli: Cli, config: config::Config) -> anyhow::Result<()> {
    if let Some(command) = cli.command {
        match command {
            Commands::List { plain } => commands::list::execute(plain)?,
            Commands::Install { repo, name } => commands::install::execute(repo, name, &config)?,
            Commands::Clean { profile } => commands::clean::execute(profile)?,
            Commands::Doctor { json } => commands::doctor::execute(json)?,
            Commands::Setup { command, force } => match command {
                Some(SetupCommands::Shell {
                    override_nvim,
                    force_env,
                    print_env,
                    print,
                    dry_run,
                    remove,
                }) => {
                    setup::shell::execute(
                        override_nvim,
                        force_env,
                        print_env,
                        print,
                        dry_run,
                        remove,
                    )?;
                }
                None => commands::setup::execute(force, &config)?,
            },
            Commands::Sandbox { profile } => commands::sandbox::execute(profile, &config)?,
            Commands::Registry { command: reg_cmd } => {
                commands::registry::execute(reg_cmd, &config)?
            }
            Commands::Update => commands::update::execute()?,
            Commands::Completions { shell } => cli::print_completions(shell),
            Commands::Help { command: sub } => {
                let mut cmd = Cli::command();
                if let Some(s) = sub {
                    if let Some(sub_cmd) = cmd.find_subcommand_mut(&s) {
                        let _ = sub_cmd.print_help();
                    } else {
                        output::bad(&format!("no such command: {}", s));
                    }
                } else {
                    let _ = cmd.print_help();
                }
            }
        }
    } else {
        if let Some(ref p) = cli.profile {
            let reserved = [
                "list",
                "install",
                "clean",
                "doctor",
                "setup",
                "sandbox",
                "registry",
                "update",
                "completions",
                "help",
                "repro",
                "debug",
            ];
            if reserved.contains(&p.as_str()) {
                output::bad(&format!(
                    "'{}' is a reserved command and cannot be used as a profile name",
                    p
                ));
                std::process::exit(1);
            }
        }
        commands::run::execute(cli.profile, cli.args, &config)?;
    }
    Ok(())
}
