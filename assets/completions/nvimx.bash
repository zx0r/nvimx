# nvimx completion (Bash)
# ----------------------------------------
# Installation:
# eval "$(nvimx init bash)"
# ----------------------------------------

_nvimx() {
    local cur opts profiles
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    opts="list ls install clean init sandbox -h --help -V --version"
    
    # Root: commands + profiles
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        profiles=$(nvimx list --plain 2>/dev/null)
        COMPREPLY=( $(compgen -W "${opts} ${profiles}" -- "${cur}") )
        return 0
    fi
    
    # Subcommands
    if [[ "${COMP_WORDS[1]}" == "init" && ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "fish zsh bash" -- "${cur}") )
        return 0
    fi

    # Fallback to general options
    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
}
complete -F _nvimx nvimx
