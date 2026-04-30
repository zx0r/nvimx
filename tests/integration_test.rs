use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn run_e2e_validation_suite() {
    let nvimx_bin = env!("CARGO_BIN_EXE_nvimx");

    let mut script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    script_path.push("tests");
    script_path.push("e2e");
    script_path.push("validate.sh");

    let status = Command::new("bash")
        .arg(&script_path)
        .env("NVIMX_BIN", nvimx_bin)
        .status()
        .expect("Failed to execute bash script. Is bash installed?");

    assert!(
        status.success(),
        "E2E Bash validation suite failed! Check the output above."
    );
}
