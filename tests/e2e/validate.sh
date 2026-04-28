#!/usr/bin/env bash
# =============================================================================
# nvimx Zero-Overhead Backend Validation Suite (Bash)
# =============================================================================

# Allow overriding the binary path (crucial for cargo test integration)
NVIMX="${NVIMX_BIN:-nvimx}"
TEST_PROFILE="${TEST_PROFILE:-nvim-x0r}"

PASS_COUNT=0
FAIL_COUNT=0

# Colors for output
GREEN='\033[32m'
RED='\033[31m'
NC='\033[0m'

pass_test() {
  PASS_COUNT=$((PASS_COUNT + 1))
  echo -e "${GREEN}✅ PASS:${NC} $1"
}

fail_test() {
  FAIL_COUNT=$((FAIL_COUNT + 1))
  echo -e "${RED}❌ FAIL:${NC} $1"
}

echo "🧪 Starting nvimx tests using binary: $NVIMX"
echo "--------------------------------------------------------"

# 1. Exact argv passthrough
OUT=$(command "$NVIMX" "$TEST_PROFILE" --headless -c 'lua print(table.concat(vim.v.argv, " "))' +qa 2>&1 | tr -d '\r\n')
if [[ "$OUT" == *"--headless"* ]]; then
  pass_test "1. Exact argv passthrough"
else
  fail_test "1. Exact argv passthrough (Output: '$OUT')"
fi

# 2. NVIM_APPNAME isolation
OUT=$(command "$NVIMX" "$TEST_PROFILE" --headless --cmd 'lua print(vim.env.NVIM_APPNAME or "")' +qa 2>&1 | tr -d '\r\n')
if [[ "$OUT" == *"$TEST_PROFILE"* ]]; then
  pass_test "2. NVIM_APPNAME isolation ($OUT)"
else
  fail_test "2. NVIM_APPNAME isolation (Output: '$OUT')"
fi

# 3. XDG config path mapping
OUT=$(command "$NVIMX" "$TEST_PROFILE" --headless --cmd 'lua print(vim.fn.stdpath("config"))' +qa 2>&1 | tr -d '\r\n')
if [[ "$OUT" == *"$TEST_PROFILE"* ]]; then
  pass_test "3. XDG config path ($OUT)"
else
  fail_test "3. XDG config path (Output: '$OUT')"
fi

# 4. Startup overhead benchmark
RAW_TIME=$(command "$NVIMX" "$TEST_PROFILE" --headless -c 'lua local t=vim.uv.hrtime(); vim.defer_fn(function() io.write(string.format("%.2f",(vim.uv.hrtime()-t)/1e6)) end,0)' +qa 2>&1)
TIME=$(echo "$RAW_TIME" | grep -oE '[0-9]+\.[0-9]+' | head -n 1)

if [[ -n "$TIME" ]]; then
  if echo "$TIME 15.00" | awk '{if ($1 < $2) exit 0; else exit 1}'; then
    pass_test "4. Startup overhead ($TIME ms)"
  else
    fail_test "4. Startup overhead ($TIME ms - exceeded 15.00ms!)"
  fi
else
  fail_test "4. Startup overhead (Failed to parse time from: '$RAW_TIME')"
fi

# 5. Headless LSP initialization
OUT=$(command "$NVIMX" "$TEST_PROFILE" --headless -c 'lua local ok,_=pcall(vim.lsp.start,{name="val",cmd={"echo"},root_dir=vim.fn.getcwd()}); vim.wait(200,function() return #vim.lsp.get_clients({name="val"})>0 end); vim.print(ok)' +qa 2>&1 | tr -d '\r\n')
if [[ "$OUT" == *"true"* ]]; then
  pass_test "5. Headless LSP init"
else
  fail_test "5. Headless LSP init (Output: '$OUT')"
fi

# 6. IPC server readiness
SOCK="/tmp/nvimx_ipc_$$.sock"
command "$NVIMX" "$TEST_PROFILE" --headless --listen "$SOCK" </dev/null >/dev/null 2>&1 &
NVIM_PID=$!

SOCREADY=false
for i in {1..40}; do
  if [[ -e "$SOCK" ]]; then
    SOCREADY=true
    break
  fi
  sleep 0.05
done

if [[ "$SOCREADY" == false ]]; then
  fail_test "6. IPC server readiness (Socket timeout)"
  kill -9 $NVIM_PID 2>/dev/null
else
  IPC_OUT=$(command nvim --server "$SOCK" --remote-expr '"IPC_OK"' </dev/null 2>/dev/null | tr -d '\r\n')

  if [[ "$IPC_OUT" == *"IPC_OK"* ]]; then
    pass_test "6. IPC server readiness (Socket responsive)"
  else
    fail_test "6. IPC server readiness (Got: '$IPC_OUT')"
  fi

  kill $NVIM_PID 2>/dev/null
  command rm -f "$SOCK"
fi

# =============================================================================
# Summary
# =============================================================================
echo "--------------------------------------------------------"
if [ $FAIL_COUNT -eq 0 ]; then
  echo -e "${GREEN}🏆 SUCCESS: All $PASS_COUNT tests passed!${NC}"
  exit 0
else
  echo -e "${RED}💥 FAILED: $FAIL_COUNT tests failed (Passed: $PASS_COUNT)${NC}"
  exit 1
fi
