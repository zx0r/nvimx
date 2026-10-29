#![allow(clippy::expect_used)]

use std::fs;
use std::process::{Command, Stdio};
use std::time::Instant;

fn setup_test_env(test_name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let base = std::env::temp_dir().join(format!("nvimx_e2e_{test_name}_{}", std::process::id()));
    let _ = fs::create_dir_all(&base);

    let config_dir = base.join("config");
    let profile_dir = config_dir.join("nvim").join("testprofile");
    fs::create_dir_all(&profile_dir).expect("failed to create test profile dir");

    // Minimal init.lua for 0-latency startup
    fs::write(profile_dir.join("init.lua"), b"-- e2e test init\n")
        .expect("failed to write init.lua");

    (base, config_dir)
}

#[test]
fn test_1_exact_argv_passthrough() {
    let (base, config_dir) = setup_test_env("argv_passthrough");
    let bin = env!("CARGO_BIN_EXE_nvimx");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .args(["-c", "lua print(table.concat(vim.v.argv, ' '))"])
        .arg("+qa")
        .output()
        .expect("failed to execute nvimx");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout} {stderr}");

    assert!(
        combined.contains("--headless"),
        "Expected output to contain '--headless', got: {combined}"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_2_nvim_appname_isolation() {
    let (base, config_dir) = setup_test_env("appname_isolation");
    let bin = env!("CARGO_BIN_EXE_nvimx");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .args(["--cmd", "lua print(vim.env.NVIM_APPNAME or '')"])
        .arg("+qa")
        .output()
        .expect("failed to execute nvimx");

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.contains("testprofile"),
        "Expected output to contain 'testprofile', got: {combined}"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_3_xdg_config_path_mapping() {
    let (base, config_dir) = setup_test_env("path_mapping");
    let bin = env!("CARGO_BIN_EXE_nvimx");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .args(["--cmd", "lua print(vim.fn.stdpath('config'))"])
        .arg("+qa")
        .output()
        .expect("failed to execute nvimx");

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.contains("testprofile"),
        "Expected config path to contain 'testprofile', got: {combined}"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_4_startup_overhead_benchmark() {
    let (base, config_dir) = setup_test_env("startup_overhead");
    let bin = env!("CARGO_BIN_EXE_nvimx");

    let start = Instant::now();
    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .args([
            "-c",
            "lua local t=vim.uv.hrtime(); vim.defer_fn(function() io.write(string.format('%.2f',(vim.uv.hrtime()-t)/1e6)) end,0)",
        ])
        .arg("+qa")
        .output()
        .expect("failed to execute nvimx");

    let elapsed = start.elapsed();
    assert!(output.status.success());
    println!("Total E2E test execution time: {elapsed:?}");

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_5_headless_lsp_init() {
    let (base, config_dir) = setup_test_env("lsp_init");
    let bin = env!("CARGO_BIN_EXE_nvimx");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .args([
            "-c",
            "lua local ok,_=pcall(vim.lsp.start,{name='val',cmd={'echo'},root_dir=vim.fn.getcwd()}); vim.wait(200,function() return #vim.lsp.get_clients({name='val'})>0 end); vim.print(ok)",
        ])
        .arg("+qa")
        .output()
        .expect("failed to execute nvimx");

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.contains("true"),
        "Expected output to contain 'true', got: {combined}"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_6_ipc_server_readiness() {
    let (base, config_dir) = setup_test_env("ipc_readiness");
    let bin = env!("CARGO_BIN_EXE_nvimx");
    let sock = base.join("test_nvim_ipc.sock");

    let mut child = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_dir)
        .arg("testprofile")
        .arg("--headless")
        .arg("--listen")
        .arg(&sock)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn background nvimx");

    // Wait briefly for socket to become ready
    let mut ready = false;
    for _ in 0..20 {
        if sock.exists() {
            ready = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    assert!(ready, "IPC socket was not created in time");

    let output = Command::new("nvim")
        .arg("--server")
        .arg(&sock)
        .args(["--remote-expr", "\"IPC_OK\""])
        .output()
        .expect("failed to query nvim IPC server");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("IPC_OK"),
        "Expected IPC response 'IPC_OK', got: {stdout}"
    );

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(&base);
}
