use crate::{cargo_bin, extension::use_latest_grafbase_sdk_in_cargo_toml};
use std::process;
use tempfile::tempdir;
use wiremock::matchers;

#[tokio::test]
async fn publish_basic() {
    // FIXME: Make this test work on windows, linux arm64 and darwin x86.
    if cfg!(windows)
        || (cfg!(target_arch = "aarch64") && cfg!(target_os = "linux"))
        || (cfg!(target_arch = "x86_64") && cfg!(target_os = "macos"))
    {
        return;
    }

    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    let project_path_str = project_path.to_string_lossy();

    let mut init_command = process::Command::new(cargo_bin("grafbase"));

    init_command.args(["extension", "init", "--type", "resolver", &*project_path_str]);

    let init_output = init_command.output().unwrap();

    assert!(
        init_output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&init_output.stdout),
        String::from_utf8_lossy(&init_output.stderr)
    );

    use_latest_grafbase_sdk_in_cargo_toml(&project_path);

    // Check that we error on missing built files
    {
        let mut publish_command = process::Command::new(cargo_bin("grafbase"));
        publish_command
            .arg("extension")
            .arg("publish")
            .current_dir(&project_path);

        let build_output = publish_command.output().unwrap();

        assert!(
            !build_output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&build_output.stdout),
            String::from_utf8_lossy(&build_output.stderr)
        );

        insta::assert_snapshot!(String::from_utf8(build_output.stdout).unwrap(), @"");
        insta::assert_snapshot!(String::from_utf8(build_output.stderr).unwrap(), @"Error: Failed to open extension manifest at `./build/manifest.json`: No such file or directory (os error 2)");
    }

    let mut build_command = process::Command::new(cargo_bin("grafbase"));
    build_command
        .arg("extension")
        .arg("build")
        // we do -D warnings in CI, the template has unused variable warnings...
        .env("RUSTFLAGS", "")
        .current_dir(&project_path);

    let build_output = build_command.output().unwrap();

    assert!(
        build_output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&build_output.stdout),
        String::from_utf8_lossy(&build_output.stderr)
    );

    let build_path = project_path.join("build");
    assert!(std::fs::exists(build_path.join("extension.wasm")).unwrap());
    assert!(std::fs::exists(build_path.join("manifest.json")).unwrap());

    // Now publish!

    let mock_server = wiremock::MockServer::start().await;

    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "extensionPublish": {
                    "__typename": "ExtensionPublishSuccess",
                    "extensionVersion": {
                        "extension": {
                            "name": "the-extension-name",
                        },
                        "version": "1.0.0",
                    },
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let mut publish_command = process::Command::new(cargo_bin("grafbase"));
    publish_command
        .arg("extension")
        .arg("publish")
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .current_dir(&project_path);

    let publish_output = publish_command.output().unwrap();

    assert!(
        publish_output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&publish_output.stdout),
        String::from_utf8_lossy(&publish_output.stderr)
    );
}
