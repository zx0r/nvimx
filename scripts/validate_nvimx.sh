#!/usr/bin/env bash
# ==============================================================================
# nvimx Production Validation Suite (v2.1 - Cross-Platform & TTY-Safe)
# Usage: ./validate_nvimx.sh [--profile <name>] [--verbose]
# ==============================================================================
set -euo pipefail

# ==============================================================================
# 1. CONFIGURATION
# ==============================================================================
: "${NVIMX_BIN:=nvimx}"
: "${NVIM_BIN:=nvim}"
: "${TEST_PROFILE:=default}"  # <-- Изменено на default. Переопределите через --profile
: "${MAX_STARTUP_MS:=15}"
: "${VERBOSE:=0}"

declare -r TEMP_DIR=$(mktemp -d)
declare -i PASS_COUNT=0
declare -i FAIL_COUNT=0

# ==============================================================================
# 2. UTILITIES (Defined BEFORE use)
# ==============================================================================
get_ms() {
    local ts
    ts=$(date +%s%N 2>/dev/null) || ts=""
    # Linux: %N returns nanoseconds. macOS: returns literal 'N'
    if [[ "$ts" =~ ^[0-9]{13,}$ ]]; then
        echo $(( ts / 1000000 ))
    else
        echo $(( $(date +%s) * 1000 ))
    fi
}

log() {
    local level="$1"; shift
    local color=""
    case "$level" in
        INFO)  color="\033[1;34m" ;;
        PASS)  color="\033[1;32m" ;;
        FAIL)  color="\033[1;31m" ;;
        DEBUG) [[ "$VERBOSE" -eq 1 ]] && color="\033[0;37m" || return 0 ;;
    esac
    printf "${color}[%s] %s\033[0m\n" "$level" "$*" >&2
}

cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null || true
    jobs -p 2>/dev/null | xargs -r kill 2>/dev/null || true
    stty sane 2>/dev/null || true
}
trap cleanup EXIT INT TERM

# ==============================================================================
# 3. TEST SUITE
# ==============================================================================
test_argv_passthrough() {
    local profile="$1"
    local out
    out=$("$NVIMX_BIN" "$profile" --headless \
        --cmd 'lua io.stdout:write(table.concat(vim.v.argv, " ") .. "\n"); io.flush()' \
        +q </dev/null 2>/dev/null) || return 1
    echo "$out" | grep -q -- "--headless"
}

test_appname_isolation() {
    local profile="$1"
    local out
    out=$("$NVIMX_BIN" "$profile" --headless \
        --cmd 'lua print(vim.env.NVIM_APPNAME or "")' +q \
        </dev/null 2>/dev/null) || return 1
    [[ "$out" == "$profile" ]]
}

test_xdg_config() {
    local profile="$1"
    local out
    out=$("$NVIMX_BIN" "$profile" --headless \
        --cmd 'lua print(vim.fn.stdpath("config"))' +q \
        </dev/null 2>/dev/null) || return 1
    echo "$out" | grep -q "$profile"
}

test_startup_overhead() {
    local profile="$1" max="$MAX_STARTUP_MS"
    local out ms
    out=$("$NVIMX_BIN" "$profile" --headless \
        --cmd 'lua local t=vim.uv.hrtime(); vim.defer_fn(function() io.write(string.format("%.2f",(vim.uv.hrtime()-t)/1e6)) end,0)' \
        +q </dev/null 2>/dev/null) || return 1
    
    ms=$(echo "$out" | grep -oE '[0-9]+\.[0-9]+' | head -1)
    [[ -n "$ms" ]] && awk "BEGIN {exit ($ms <= $max) ? 0 : 1}"
}

test_headless_lsp() {
    local profile="$1"
    local out
    out=$("$NVIMX_BIN" "$profile" --headless \
        --cmd 'lua local ok,_=pcall(vim.lsp.start,{name="val",cmd={"echo"},root_dir=vim.fn.getcwd()}); vim.wait(200,function() return #vim.lsp.get_clients({name="val"})>0 end); vim.print(ok)' \
        +q </dev/null 2>/dev/null) || return 1
    [[ "$out" == "true" ]]
}

test_ipc_readiness() {
    local profile="$1"
    local sock="$TEMP_DIR/nvimx_ipc_$$.sock"
    local pid

    "$NVIMX_BIN" "$profile" --headless --listen "$sock" \
        </dev/null >/dev/null 2>&1 &
    pid=$!

    for i in {1..40}; do
        [[ -e "$sock" ]] && break
        sleep 0.05
    done

    if [[ ! -e "$sock" ]]; then
        kill "$pid" 2>/dev/null || true
        return 1
    fi

    local out
    out=$("$NVIM_BIN" --server "$sock" --remote-expr '"IPC_OK"' \
        </dev/null 2>/dev/null) || true
    out=$(echo "$out" | tr -d '[:space:]')

    kill "$pid" 2>/dev/null || true
    rm -f "$sock" 2>/dev/null || true

    [[ "$out" == "IPC_OK" ]]
}

# ==============================================================================
# 4. TEST RUNNER
# ==============================================================================
run_test() {
    local name="$1" func="$2" profile="$3"
    log INFO "▶ Running: $name"
    
    local start_ms=$(get_ms)
    
    # Disable set -e to allow suite continuation on single failure
    set +e
    (eval "$func" "$profile")
    local rc=$?
    set -e
    
    local end_ms=$(get_ms)
    local duration_ms=$(( end_ms - start_ms ))

    if [[ $rc -eq 0 ]]; then
        log PASS "✅ PASS ($name) [${duration_ms}ms]"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        # Instant diagnostics
        local dbg_appname dbg_config
        dbg_appname=$("$NVIMX_BIN" "$profile" --headless --cmd 'lua print(vim.env.NVIM_APPNAME or "nil")' +q </dev/null 2>/dev/null) || dbg_appname="[err]"
        dbg_config=$("$NVIMX_BIN" "$profile" --headless --cmd 'lua print(vim.fn.stdpath("config"))' +q </dev/null 2>/dev/null) || dbg_config="[err]"
        
        log FAIL "❌ FAIL ($name) [${duration_ms}ms]"
        log DEBUG "   💡 APPNAME: '$dbg_appname'"
        log DEBUG "   💡 CONFIG:  '$dbg_config'"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# ==============================================================================
# 5. ENTRYPOINT
# ==============================================================================
main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --profile) TEST_PROFILE="$2"; shift 2 ;;
            --nvimx)   NVIMX_BIN="$2"; shift 2 ;;
            --verbose) VERBOSE=1; shift ;;
            --help)    echo "Usage: $0 [--profile <name>] [--nvimx <path>] [--verbose]"; exit 0 ;;
            *)         log FAIL "Unknown flag: $1"; exit 1 ;;
        esac
    done

    for cmd in "$NVIMX_BIN" "$NVIM_BIN" awk; do
        command -v "$cmd" >/dev/null 2>&1 || { log FAIL "Missing: $cmd"; exit 127; }
    done
    unset NVIM_APPNAME 2>/dev/null || true

    log INFO "🔬 Starting nvimx validation suite..."
    log DEBUG "Config: NVIMX=$NVIMX_BIN | NVIM=$NVIM_BIN | PROFILE=$TEST_PROFILE"

    run_test "test_argv_passthrough" test_argv_passthrough "$TEST_PROFILE"
    run_test "test_appname_isolation" test_appname_isolation "$TEST_PROFILE"
    run_test "test_xdg_config"       test_xdg_config       "$TEST_PROFILE"
    run_test "test_startup_overhead" test_startup_overhead "$TEST_PROFILE"
    run_test "test_headless_lsp"     test_headless_lsp     "$TEST_PROFILE"
    run_test "test_ipc_readiness"    test_ipc_readiness    "$TEST_PROFILE"

    echo -e "\n📊 =========================================="
    echo -e "📊 VALIDATION RESULTS"
    echo -e "📊 =========================================="
    log PASS "Passed : $PASS_COUNT"
    [[ $FAIL_COUNT -gt 0 ]] && log FAIL "Failed : $FAIL_COUNT" || log INFO "Failed : $FAIL_COUNT"
    echo -e "📊 =========================================="
    
    [[ $FAIL_COUNT -eq 0 ]] && exit 0 || exit 1
}

main "$@"

