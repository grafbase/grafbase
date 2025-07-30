mod install;
mod publish;
mod types;
mod update;

use duct::cmd;
use std::path::Path;
use tempfile::tempdir;

use crate::cargo_bin;

fn use_latest_grafbase_sdk_in_cargo_toml(project_path: &Path) {
    let grafbase_sdk_dir = format!("{}/../crates/grafbase-sdk", env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();

    let regex = regex::Regex::new(r#"grafbase-sdk\s*=\s*".*""#).unwrap();
    let cargo_toml = regex.replace_all(
        &cargo_toml,
        format!(r#"grafbase-sdk = {{ path = "{grafbase_sdk_dir}" }}"#),
    );

    let regex = regex::Regex::new(r#"grafbase-sdk\s*=\s*\{\s*version\s*=\s*".*?"(.*)\}"#).unwrap();
    let cargo_toml = regex.replace_all(
        &cargo_toml,
        format!(r#"grafbase-sdk = {{ path = "{grafbase_sdk_dir}", features = ["test-utils"] }}"#),
    );
    println!("{cargo_toml}");

    std::fs::write(project_path.join("Cargo.toml"), cargo_toml.as_ref()).unwrap();
}

#[test]
fn build_with_source_dir() {
    // FIXME: Make this test work on windows, linux arm64 and darwin x86.
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let extension_path = temp_dir.path().join("my_extension");
    let extension_path_str = extension_path.to_string_lossy();

    // Initialize extension in a subdirectory
    let args = vec!["extension", "init", "--type", "resolver", &*extension_path_str];
    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();
    command.run().unwrap();

    use_latest_grafbase_sdk_in_cargo_toml(&extension_path);

    // Build from parent directory using --source-dir
    let args = vec!["extension", "build", "--source-dir", "my_extension"];

    let result = cmd(cargo_bin("grafbase"), &args)
        .env("RUSTFLAGS", "")
        .dir(temp_dir.path())
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

    // Verify the build artifacts exist in the default build directory
    let build_path = temp_dir.path().join("build");
    assert!(std::fs::exists(build_path.join("extension.wasm")).unwrap());
    assert!(std::fs::exists(build_path.join("manifest.json")).unwrap());

    // Verify that the target directory was created in the source directory, not nested
    let target_path = extension_path.join("target");
    assert!(target_path.exists());

    // Verify that no nested directory structure was created
    let nested_path = extension_path.join("my_extension").join("target");
    assert!(!nested_path.exists());
}

#[test]
fn build_with_source_dir_and_scratch_dir() {
    // FIXME: Make this test work on windows, linux arm64 and darwin x86.
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let extension_path = temp_dir.path().join("my_extension");
    let extension_path_str = extension_path.to_string_lossy();

    // Initialize extension in a subdirectory
    let args = vec!["extension", "init", "--type", "resolver", &*extension_path_str];
    let command = cmd(cargo_bin("grafbase"), &args).stdout_null().stderr_null();
    command.run().unwrap();

    use_latest_grafbase_sdk_in_cargo_toml(&extension_path);

    // Build from parent directory using both --source-dir and --scratch-dir
    let args = vec![
        "extension",
        "build",
        "--source-dir",
        "my_extension",
        "--scratch-dir",
        "my_extension/custom_target",
    ];

    let result = cmd(cargo_bin("grafbase"), &args)
        .env("RUSTFLAGS", "")
        .dir(temp_dir.path())
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

    // Verify the build artifacts exist in the default build directory
    let build_path = temp_dir.path().join("build");
    assert!(std::fs::exists(build_path.join("extension.wasm")).unwrap());
    assert!(std::fs::exists(build_path.join("manifest.json")).unwrap());

    // Verify that the custom target directory was created in the right place
    let custom_target_path = extension_path.join("custom_target");
    assert!(custom_target_path.exists());

    // Verify that no nested directory structure was created
    let nested_path = extension_path.join("my_extension").join("custom_target");
    assert!(!nested_path.exists(), "Nested directory structure should not exist");

    // Also verify no "my_extension/my_extension" directory was created
    let double_nested = extension_path.join("my_extension");
    assert!(!double_nested.exists(), "Double nested directory should not exist");
}
