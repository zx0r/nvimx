Create a minimal terminal animation for a CLI tool called "nvimx".

Scene:
- dark terminal UI
- smooth macOS-style rendering
- crisp monospace font (JetBrains Mono style)

Command:
nvimx lazyvim --headless -c "lua print(vim.fn.stdpath('config'))" +qa

Output:
[✓] zero-overhead
[✓] isolated environment
[✓] argv passthrough

Final text:
you bring args → we don't touch them
you choose a profile → we don't leak state

Style:
- ultra smooth typing
- no flicker
- clean cursor animation
- high contrast text

Duration:
3–4 seconds loop
