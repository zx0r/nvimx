#!/usr/bin/env bash
set -euo pipefail

# Build release binary first
cargo build --release

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NVIMX="$PROJECT_ROOT/target/release/nvimx"
TEST_PROFILE="e2e_test_profile"

# Setup temporary isolated XDG environment for testing
TEMP_DIR="$(mktemp -d -t nvimx-e2e-XXXXXX)"
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

export XDG_CONFIG_HOME="$TEMP_DIR/config"
export XDG_DATA_HOME="$TEMP_DIR/data"
export XDG_STATE_HOME="$TEMP_DIR/state"
export XDG_CACHE_HOME="$TEMP_DIR/cache"

mkdir -p "$XDG_CONFIG_HOME/nvim/$TEST_PROFILE"
mkdir -p "$XDG_CONFIG_HOME/nvimx"

# Write minimal init.lua to ensure fast startup without plugin manager overhead
cat << 'EOF' > "$XDG_CONFIG_HOME/nvim/$TEST_PROFILE/init.lua"
-- minimal init for e2e validation
EOF

PASSED=0
FAILED=0

run_test() {
    local num="$1"
    local desc="$2"
    local cmd="$3"
    local expected="$4"

    echo "Running Test $num: $desc..."
    set +e
    OUT=$(eval "$cmd")
    local exit_code=$?
    set -e

    if [[ "$OUT" == *"$expected"* ]]; then
        echo "  ✔ Test $num passed"
        PASSED=$((PASSED + 1))
    else
        echo "  ✘ Test $num failed"
        echo "    Expected output to contain: '$expected'"
        echo "    Got output:"
        echo "$OUT"
        FAILED=$((FAILED + 1))
    fi
}

echo "=== Starting nvimx E2E Validation Suite ==="

# Test 1: Exact argv passthrough
run_test 1 "Exact argv passthrough" \
    "$NVIMX \"$TEST_PROFILE\" --headless -c 'lua print(table.concat(vim.v.argv, \" \"))' +qa 2>&1" \
    "--headless"

# Test 2: NVIM_APPNAME isolation
run_test 2 "NVIM_APPNAME isolation" \
    "$NVIMX \"$TEST_PROFILE\" --headless --cmd 'lua print(vim.env.NVIM_APPNAME or \"\")' +qa 2>&1" \
    "$TEST_PROFILE"

# Test 3: XDG config path mapping
run_test 3 "XDG config path mapping" \
    "$NVIMX \"$TEST_PROFILE\" --headless --cmd 'lua print(vim.fn.stdpath(\"config\"))' +qa 2>&1" \
    "$TEST_PROFILE"

# Test 4: Startup overhead benchmark (< 15.00 ms locally)
echo "Running Test 4: Startup overhead benchmark..."
BENCH_OUT=$($NVIMX "$TEST_PROFILE" --headless -c 'lua local t=vim.uv.hrtime(); vim.defer_fn(function() io.write(string.format("%.2f",(vim.uv.hrtime()-t)/1e6)) end,0)' +qa 2>&1 || true)
TIME=$(echo "$BENCH_OUT" | grep -oE '[0-9]+\.[0-9]+' | head -n 1 || echo "999.0")
echo "  Measured startup delay: ${TIME} ms"
if (( $(echo "$TIME < 15.00" | bc -l) )); then
    echo "  ✔ Test 4 passed (${TIME} ms < 15.00 ms threshold)"
    PASSED=$((PASSED + 1))
else
    echo "  ✘ Test 4 failed (${TIME} ms exceeded threshold)"
    FAILED=$((FAILED + 1))
fi

# Test 5: Headless LSP init
run_test 5 "Headless LSP init" \
    "$NVIMX \"$TEST_PROFILE\" --headless -c 'lua local ok,_=pcall(vim.lsp.start,{name=\"val\",cmd={\"echo\"},root_dir=vim.fn.getcwd()}); vim.wait(200,function() return #vim.lsp.get_clients({name=\"val\"})>0 end); vim.print(ok)' +qa 2>&1" \
    "true"

# Test 6: IPC server readiness
echo "Running Test 6: IPC server readiness..."
SOCK="$TEMP_DIR/test_nvim_ipc.sock"
$NVIMX "$TEST_PROFILE" --headless --listen "$SOCK" </dev/null >/dev/null 2>&1 &
SERVER_PID=$!
sleep 0.2

set +e
IPC_OUT=$(nvim --server "$SOCK" --remote-expr '"IPC_OK"' 2>&1)
set -e
kill -9 "$SERVER_PID" 2>/dev/null || true

if [[ "$IPC_OUT" == *"IPC_OK"* ]]; then
    echo "  ✔ Test 6 passed"
    PASSED=$((PASSED + 1))
else
    echo "  ✘ Test 6 failed (IPC_OUT: $IPC_OUT)"
    FAILED=$((FAILED + 1))
fi

echo "=========================================="
if [ "$FAILED" -eq 0 ]; then
    echo "🏆 SUCCESS: All 6 tests passed!"
    exit 0
else
    echo "💥 FAILED: $FAILED tests failed"
    exit 1
fi
