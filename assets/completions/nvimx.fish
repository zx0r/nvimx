# nvimx completion (Fish)
# ----------------------------------------
# Installation:
# nvimx completions fish > ~/.config/fish/completions/nvimx.fish
# ----------------------------------------

# Disable file completion
complete -c nvimx -f

# Commands (shown at root)
complete -c nvimx -n "__fish_use_subcommand" -a "list"        -d "List profiles"
complete -c nvimx -n "__fish_use_subcommand" -a "install"     -d "Install a profile"
complete -c nvimx -n "__fish_use_subcommand" -a "clean"       -d "Remove profile data"
complete -c nvimx -n "__fish_use_subcommand" -a "doctor"      -d "Check system health"
complete -c nvimx -n "__fish_use_subcommand" -a "sandbox"     -d "Run profile in isolation"
complete -c nvimx -n "__fish_use_subcommand" -a "registry"    -d "Manage registries"
complete -c nvimx -n "__fish_use_subcommand" -a "update"      -d "Update nvimx"
complete -c nvimx -n "__fish_use_subcommand" -a "completions" -d "Generate shell completions"
complete -c nvimx -n "__fish_use_subcommand" -a "help"        -d "Show help"

# Global flags
complete -c nvimx -s h -l help -d "Show help"
complete -c nvimx -s V -l version -d "Show version"

# Dynamic profiles (root usage)
complete -c nvimx \
  -n "not __fish_seen_subcommand_from list install clean doctor sandbox registry update completions help" \
  -a "(nvimx list --plain 2>/dev/null)" \
  -d "Profile" \
  -r

# Dynamic profiles (commands expecting profile)
complete -c nvimx \
  -n "__fish_seen_subcommand_from install clean sandbox" \
  -a "(nvimx list --plain 2>/dev/null)" \
  -d "Profile" \
  -r

# Registry subcommands
complete -c nvimx \
  -n "__fish_seen_subcommand_from registry; and not __fish_seen_subcommand_from list check update clear" \
  -a "list"   -d "List registries"
complete -c nvimx \
  -n "__fish_seen_subcommand_from registry; and not __fish_seen_subcommand_from list check update clear" \
  -a "check"  -d "Check registry health"
complete -c nvimx \
  -n "__fish_seen_subcommand_from registry; and not __fish_seen_subcommand_from list check update clear" \
  -a "update" -d "Force registry update"
complete -c nvimx \
  -n "__fish_seen_subcommand_from registry; and not __fish_seen_subcommand_from list check update clear" \
  -a "clear"  -d "Clear registry cache"
