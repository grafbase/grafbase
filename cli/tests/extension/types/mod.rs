use duct::cmd;
use tempfile::tempdir;

use crate::{cargo_bin, extension::use_latest_grafbase_sdk_in_cargo_toml};

mod authentication;
mod authorization;
mod contracts;
mod hooks;
mod resolver;

// Only checking test for a single extension, it's a massive CPU sink
#[test]
fn check_test() {
    // FIXME: Make this test work on windows, linux arm64 and macos x86
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let args = vec!["extension", "init", "--type", "authorization", &*project_path_str];
    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();
    command.run().unwrap();

    use_latest_grafbase_sdk_in_cargo_toml(&project_path);

    let result = cmd("cargo", &["check", "--tests"])
        .env("RUSTFLAGS", "")
        .dir(&project_path)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run()
        .unwrap();

    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
