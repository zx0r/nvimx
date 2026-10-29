# nvimx completion (Fish)
# ----------------------------------------
# Installation:
# nvimx init fish > ~/.config/fish/completions/nvimx.fish
# ----------------------------------------

# Disable file completion
complete -c nvimx -f

# Commands (shown at root)
complete -c nvimx -n "__fish_use_subcommand" -a "list"        -d "List profiles"
complete -c nvimx -n "__fish_use_subcommand" -a "ls"          -d "List profiles"
complete -c nvimx -n "__fish_use_subcommand" -a "install"     -d "Install a profile"
complete -c nvimx -n "__fish_use_subcommand" -a "clean"       -d "Remove profile data"
complete -c nvimx -n "__fish_use_subcommand" -a "init"        -d "Initialize environment or completions"
complete -c nvimx -n "__fish_use_subcommand" -a "sandbox"     -d "Run profile in isolation"

# Global flags
complete -c nvimx -s h -l help -d "Show help"
complete -c nvimx -s V -l version -d "Show version"

# Dynamic profiles (root usage)
complete -c nvimx \
  -n "not __fish_seen_subcommand_from list ls install clean init sandbox" \
  -a "(nvimx list --plain 2>/dev/null)" \
  -d "Profile" \
  -r

# Dynamic profiles (commands expecting profile)
complete -c nvimx \
  -n "__fish_seen_subcommand_from clean sandbox" \
  -a "(nvimx list --plain 2>/dev/null)" \
  -d "Profile" \
  -r
