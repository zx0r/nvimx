# zsh completion for nvimx
_nvimx_profiles() {
    local -a profiles
    profiles=($(nvimx list --raw 2>/dev/null))
    if [[ ${#profiles} -eq 0 ]]; then
        profiles=("empty")
    fi
    _describe -t profiles 'profiles' profiles
}

_nvimx() {
    local line
    local -a cmds
    cmds=(
        'list:List profiles'
        'install:Install profile'
        'clean:Clean profile data'
        'doctor:Check system health'
        'sandbox:Run profile in isolated environment'
        'registry:Manage registries'
        'update:Update nvimx'
        'completions:Generate completions'
        'help:Show help'
    )

    _arguments -C \
        "1: :->first_arg" \
        "*::arg:->rest_args"

    case $state in
        first_arg)
            _describe -t commands 'commands' cmds
            _nvimx_profiles
            ;;
        rest_args)
            case $line[1] in
                clean) _nvimx_profiles ;;
            esac
            ;;
    esac
}
_nvimx "$@"
