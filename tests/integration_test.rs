use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn run_e2e_validation_suite() {
    // Получаем путь к скомпилированному бинарнику `nvimx` (target/debug/nvimx)
    let nvimx_bin = env!("CARGO_BIN_EXE_nvimx");

    // Получаем путь к bash скрипту (относительно корня проекта)
    let mut script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    script_path.push("tests");
    script_path.push("e2e");
    script_path.push("validate.sh");

    // Запускаем bash скрипт
    let status = Command::new("bash")
        .arg(&script_path)
        .env("NVIMX_BIN", nvimx_bin) // Передаем скрипту путь к бинарнику
        .status()
        .expect("Failed to execute bash script. Is bash installed?");

    // Тест считается успешным, только если скрипт вернул exit code 0
    assert!(
        status.success(),
        "E2E Bash validation suite failed! Check the output above."
    );
}
