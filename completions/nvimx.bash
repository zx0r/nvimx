# nvimx completion (Bash)
# ----------------------------------------
# Installation:
# nvimx completions bash > ~/.bash_completion.d/nvimx
#
# Then add to your .bashrc:
# source ~/.bash_completion.d/nvimx
# ----------------------------------------

_nvimx() {
    local cur opts profiles
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    opts="list install clean doctor sandbox registry update completions help -h --help -V --version"
    
    # Root: commands + profiles
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        profiles=$(nvimx list --plain 2>/dev/null)
        COMPREPLY=( $(compgen -W "${opts} ${profiles}" -- "${cur}") )
        return 0
    fi
    
    # Registry subcommands
    if [[ "${COMP_WORDS[1]}" == "registry" && ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "list check update clear" -- "${cur}") )
        return 0
    fi

    # Fallback to general options
    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
}
complete -F _nvimx nvimx
