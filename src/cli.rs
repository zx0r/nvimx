use clap::builder::styling::{AnsiColor, Style, Styles};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::io;

/// Professional CLI styling for nvimx.
/// Bold headers, no colors for commands/args.
/// Respects terminal capabilities and NO_COLOR.
const HEADER: Style = AnsiColor::BrightWhite.on_default().bold();
const RESET: Style = Style::new();

const STYLES: Styles = Styles::styled()
    .header(HEADER)
    .usage(HEADER)
    .literal(Style::new())
    .placeholder(Style::new());

#[derive(Parser)]
#[command(
    name = "nvimx",
    version = "0.1.0",
    styles = STYLES,
    about = "Zero-overhead Neovim profile manager",
    after_help = "For more information: https://github.com/zx0r/nvimx",
    override_usage = "nvimx [PROFILE] [FILES...][-- ARGS...]\n  nvimx <COMMAND> [ARGS...]",
    args_conflicts_with_subcommands = true,
    disable_help_subcommand = true,
    help_template = format!(
        "\
{{about-section}}

{{usage-heading}}
  {{usage}}

{bold}Commands:{reset}  
{{subcommands}}

{bold}Arguments:{reset}
  [PROFILE]     Profile to run (defaults to configured profile)
  [FILES...]    Files to open
  [ARGS...]     Arguments passed to Neovim (after `--`)

{bold}Options:{reset}  
{{options}}

{{after-help}}
",
        bold = HEADER.render(),
        reset = RESET.render_reset(),
    )
)]
pub struct Cli {
    /// Profile name to run (defaults to configured profile)
    #[arg(hide = true)]
    pub profile: Option<String>,

    /// Arguments passed directly to Neovim
    #[arg(trailing_var_arg = true, hide = true, allow_hyphen_values = true)]
    pub args: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
#[command(subcommand_help_heading = "Commands")]
pub enum Commands {
    /// List all locally installed profiles
    List {
        /// Show bare names only (useful for scripts)
        #[arg(long)]
        plain: bool,
    },
    /// Install a new profile from a git repository or registry
    Install {
        /// Git repository URL or registry profile name
        repo: String,
        /// Name for the local profile folder
        name: String,
    },
    /// Safely remove data, state, and cache for a profile
    Clean {
        /// Profile name to clean
        profile: String,
    },
    /// Check system dependencies and environment health
    Doctor {
        /// Output the report in machine-readable JSON format
        #[arg(long)]
        json: bool,
    },
    /// Initialize environment, config, and install initial profiles
    Setup {
        /// Overwrite existing configuration with defaults
        #[arg(long, short)]
        force: bool,

        #[command(subcommand)]
        command: Option<SetupCommands>,
    },
    /// Run a profile in an isolated environment
    Sandbox {
        /// Profile name or repository URL to sandbox
        profile: String,
    },
    /// Inspect and manage configured profile registries
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Self-update the nvimx binary to the latest version
    Update,
    /// Generate auto-completion scripts for your shell
    Completions {
        /// Target shell (bash, zsh, fish)
        shell: Shell,
    },
    /// Show help information for nvimx or its subcommands
    Help {
        /// Specific command to show help for
        command: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SetupCommands {
    /// Install shell completions and nvim wrapper function
    Shell {
        /// Replace standard nvim with nvimx wrapper
        #[arg(long)]
        override_nvim: bool,
        /// Overwrite existing EDITOR/VISUAL environment variables
        #[arg(long)]
        force_env: bool,
        /// Print environment export commands only
        #[arg(long)]
        print_env: bool,
        /// Print the shell integration script to stdout
        #[arg(long)]
        print: bool,
        /// Show what changes would be made without applying them
        #[arg(long)]
        dry_run: bool,
        /// Remove nvimx shell integration
        #[arg(long)]
        remove: bool,
    },
}

#[derive(Subcommand)]
pub enum RegistryCommands {
    /// List all configured registry URLs
    List,
    /// Validate reachability and schema of configured registries
    Check,
    /// Force update the local registry cache
    Update,
    /// Manage the local registry cache file
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Permanently remove the local registry cache file
    Clear,
}

pub fn print_completions(shell: Shell) {
    match shell {
        Shell::Zsh => {
            print!("{}", include_str!("../assets/completions/_nvimx"));
        }
        Shell::Fish => {
            print!("{}", include_str!("../assets/completions/nvimx.fish"));
        }
        Shell::Bash => {
            let mut cmd = Cli::command();
            cmd.build();
            let bin_name = cmd.get_name().to_string();
            let mut out = Vec::new();
            generate(shell, &mut cmd, bin_name, &mut out);
            let s = String::from_utf8_lossy(&out);
            println!("{}", s.replace("[PROFILE]", "").replace("[ARGS]...", ""));
        }
        _ => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
    }
}
