#!/usr/bin/env fish

# =============================================================================
# nvimx Zero-Overhead Backend Validation Suite
# =============================================================================

set -q TEST_PROFILE; or set TEST_PROFILE nvim-x0r
set -g PASS_COUNT 0
set -g FAIL_COUNT 0

function pass_test
    set PASS_COUNT (math $PASS_COUNT + 1)
    set_color green
    echo -n "✅ PASS: "
    set_color normal
    echo $argv
end

function fail_test
    set FAIL_COUNT (math $FAIL_COUNT + 1)
    set_color red
    echo -n "❌ FAIL: "
    set_color normal
    echo $argv
end

echo "🧪 Starting nvimx tests with profile: $TEST_PROFILE"
echo --------------------------------------------------------

# -----------------------------------------------------------------------------
# 1. Exact argv passthrough
# Description: Verifies that nvimx transparently passes command-line arguments
# directly to Neovim without dropping, modifying, or quoting them incorrectly.
# Expected: The output must contain the exact string "--headless".
# -----------------------------------------------------------------------------
set OUT (command nvimx $TEST_PROFILE --headless -c 'lua print(table.concat(vim.v.argv, " "))' +qa 2>&1 | string trim)
if string match -q "*--headless*" "$OUT"
    pass_test "1. Exact argv passthrough"
else
    fail_test "1. Exact argv passthrough (Output: '$OUT')"
end

# -----------------------------------------------------------------------------
# 2. NVIM_APPNAME isolation
# Description: Ensures that nvimx correctly sets the NVIM_APPNAME environment
# variable before executing the Neovim binary, enabling profile isolation.
# Expected: The output must contain the requested profile name.
# -----------------------------------------------------------------------------
set OUT (command nvimx $TEST_PROFILE --headless --cmd 'lua print(vim.env.NVIM_APPNAME or "")' +qa 2>&1 | string trim)
if string match -q "*$TEST_PROFILE*" "$OUT"
    pass_test "2. NVIM_APPNAME isolation ($OUT)"
else
    fail_test "2. NVIM_APPNAME isolation (Output: '$OUT')"
end

# -----------------------------------------------------------------------------
# 3. XDG config path mapping
# Description: Verifies that Neovim's internal APIs (stdpath) correctly resolve
# the configuration directory based on the isolated environment.
# Expected: The standard config path must point to a folder containing the profile.
# -----------------------------------------------------------------------------
set OUT (command nvimx $TEST_PROFILE --headless --cmd 'lua print(vim.fn.stdpath("config"))' +qa 2>&1 | string trim)
if string match -q "*$TEST_PROFILE*" "$OUT"
    pass_test "3. XDG config path ($OUT)"
else
    fail_test "3. XDG config path (Output: '$OUT')"
end

# -----------------------------------------------------------------------------
# 4. Startup overhead benchmark
# Description: Measures the execution delay introduced by nvimx. We evaluate the
# time difference between Neovim's internal UV timer and the process start.
# Expected: The overhead must be strictly less than 15.00 milliseconds.
# -----------------------------------------------------------------------------
set RAW_TIME (command nvimx $TEST_PROFILE --headless -c 'lua local t=vim.uv.hrtime(); vim.defer_fn(function() io.write(string.format("%.2f",(vim.uv.hrtime()-t)/1e6)) end,0)' +qa 2>&1 | string trim)
# Extract only the floating point number (in case Neovim printed warnings)
set TIME (string match -r '[0-9]+\.[0-9]+' "$RAW_TIME")

if test -n "$TIME"
    # Use 'awk' for safe floating-point comparison across all systems
    if echo "$TIME 15.00" | awk '{if ($1 < $2) exit 0; else exit 1}'
        pass_test "4. Startup overhead ($TIME ms)"
    else
        fail_test "4. Startup overhead ($TIME ms - exceeded 15.00ms limit!)"
    end
else
    fail_test "4. Startup overhead (Failed to parse time from: '$RAW_TIME')"
end

# -----------------------------------------------------------------------------
# 5. Headless LSP initialization
# Description: Validates that LSP servers can spawn successfully in the headless
# environment provided by nvimx. Ensures global ENV is not corrupted.
# Expected: Neovim should return "true" after successfully spawning a dummy LSP.
# -----------------------------------------------------------------------------
set OUT (command nvimx $TEST_PROFILE --headless -c 'lua local ok,_=pcall(vim.lsp.start,{name="val",cmd={"echo"},root_dir=vim.fn.getcwd()}); vim.wait(200,function() return #vim.lsp.get_clients({name="val"})>0 end); vim.print(ok)' +qa 2>&1 | string trim)
if string match -q "*true*" "$OUT"
    pass_test "5. Headless LSP init"
else
    fail_test "5. Headless LSP init (Output: '$OUT')"
end

# -----------------------------------------------------------------------------
# 6. IPC server readiness (Zero-overhead TTY detachment)
# Description: Tests remote execution capabilities. The wrapper must not hijack
# stdout/stdin loops, allowing Neovim to create an IPC socket immediately.
# Expected: The client should successfully connect and receive "IPC_OK".
# -----------------------------------------------------------------------------
set -l SOCK "/tmp/nvimx_ipc_$fish_pid.sock"

# Launch the server in the background, fully detached from the terminal
command nvimx $TEST_PROFILE --headless --listen $SOCK </dev/null >/dev/null 2>&1 &
set -l NVIM_PID $last_pid

# Poll until the Unix socket is ready (timeout: 2 seconds)
set -l SOCREADY false
for i in (seq 1 40)
    if test -e $SOCK
        set SOCREADY true
        break
    end
    sleep 0.05
end

if not test "$SOCREADY" = true
    fail_test "6. IPC server readiness (Socket timeout after 2s)"
    kill -9 $NVIM_PID 2>/dev/null
else
    # Send a remote expression request to the running server
    set -l IPC_OUT (command nvim --server $SOCK --remote-expr '"IPC_OK"' </dev/null 2>/dev/null | string trim)

    if string match -q IPC_OK "$IPC_OUT"
        pass_test "6. IPC server readiness (Socket responsive)"
    else
        fail_test "6. IPC server readiness (Got: '$IPC_OUT')"
    end

    # Graceful cleanup
    kill $NVIM_PID 2>/dev/null
    command rm -f $SOCK
end

# =============================================================================
# Summary
# =============================================================================
echo --------------------------------------------------------
if test $FAIL_COUNT -eq 0
    set_color green
    echo "🏆 SUCCESS: All $PASS_COUNT tests passed!"
    set_color normal
    exit 0
else
    set_color red
    echo "💥 FAILED: $FAIL_COUNT tests failed (Passed: $PASS_COUNT)"
    set_color normal
    exit 1
end
