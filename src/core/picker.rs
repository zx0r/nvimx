use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn pick(items: Vec<String>) -> Result<Option<String>> {
    let mut child = Command::new("fzf")
        .args([
            "--header",
            "Select Neovim Profile (ESC to cancel)",
            "--height",
            "40%",
            "--layout",
            "reverse",
            "--border",
            "--color",
            "header:italic:cyan,pointer:bold:green,hl:bold:yellow",
            "--prompt",
            "λ ",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn 'fzf': {}. Is it installed?", e))?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    for item in items {
        writeln!(stdin, "{}", item)?;
    }
    drop(stdin);

    let out = child.wait_with_output()?;
    Ok(if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    })
}
