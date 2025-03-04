use crate::cargo_bin;
use insta::assert_debug_snapshot;
use std::fs;
use std::process;
use tempfile::tempdir;
use wiremock::matchers;

#[tokio::test]
async fn update_all_extensions() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    std::fs::create_dir_all(&project_path).unwrap();

    // Create a grafbase.toml file with extension version requirements
    let grafbase_toml_content = r#"
[extensions]
echo.version = "^1.0"
jwt.version = "*"
rest.version = "0.3.0"
spicedb.version = "2"
"#;

    fs::write(project_path.join("grafbase.toml"), grafbase_toml_content).unwrap();

    // Setup mock server
    let mock_server = wiremock::MockServer::start().await;

    // Mock response for extension version query
    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "extensionVersionsByVersionRequirement": [
                    { "__typename": "ExtensionVersion", "version": "1.1.1", "extension": { "name": "echo" } },
                    { "__typename": "ExtensionVersion", "version": "0.19.7", "extension": { "name": "jwt" } },
                    { "__typename": "ExtensionVersion", "version": "0.3.0", "extension": { "name": "rest" } },
                    { "__typename": "ExtensionVersion", "version": "2.7.0", "extension": { "name": "spicedb" } },
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Run extension update command
    let mut update_command = process::Command::new(cargo_bin("grafbase"));
    update_command
        .args(["extension", "update"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .current_dir(&project_path);

    let update_output = update_command.output().unwrap();

    if !update_output.status.success() {
        panic!("Update failed\n{update_output:#?}");
    }

    let lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    insta::assert_snapshot!(&lockfile_contents);

    // Now check out what happens when a version does not exist

    mock_server.reset().await;

    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "extensionVersionsByVersionRequirement": [
                    { "__typename": "ExtensionVersion", "version": "1.1.1", "extension": { "name": "echo" } },
                    { "__typename": "ExtensionVersionDoesNotExistError" },
                    { "__typename": "ExtensionVersion", "version": "0.3.0", "extension": { "name": "rest" } },
                    { "__typename": "ExtensionVersion", "version": "2.7.0", "extension": { "name": "spicedb" } },
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let mut update_command = process::Command::new(cargo_bin("grafbase"));
    update_command
        .args(["extension", "update"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .current_dir(&project_path);

    let update_output = update_command.output().unwrap();

    assert!(!update_output.status.success());

    assert_debug_snapshot!((String::from_utf8(update_output.stdout), String::from_utf8(update_output.stderr)), @r#"
    (
        Ok(
            "❌ No published version of extension \"jwt\" matches \"*\"\n",
        ),
        Ok(
            "",
        ),
    )
    "#);

    let new_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    assert_eq!(new_lockfile_contents, lockfile_contents);

    // And when an extension does not exist

    mock_server.reset().await;

    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "extensionVersionsByVersionRequirement": [
                    { "__typename": "ExtensionVersion", "version": "1.1.1", "extension": { "name": "echo" } },
                    { "__typename": "ExtensionDoesNotExistError" },
                    { "__typename": "ExtensionVersion", "version": "0.3.0", "extension": { "name": "rest" } },
                    { "__typename": "ExtensionVersion", "version": "2.7.0", "extension": { "name": "spicedb" } },
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let mut update_command = process::Command::new(cargo_bin("grafbase"));
    update_command
        .args(["extension", "update"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .current_dir(&project_path);

    let update_output = update_command.output().unwrap();

    assert!(!update_output.status.success());

    assert_debug_snapshot!((String::from_utf8(update_output.stdout), String::from_utf8(update_output.stderr)), @r#"
    (
        Ok(
            "❌ Extension \"jwt\" does not exist\n",
        ),
        Ok(
            "",
        ),
    )
    "#);

    let new_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    assert_eq!(new_lockfile_contents, lockfile_contents);
}

#[tokio::test]
async fn update_specific_extensions() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    std::fs::create_dir_all(&project_path).unwrap();

    // Create a grafbase.toml file with extension version requirements
    let grafbase_toml_content = r#"
[extensions]
echo.version = "^1.0"
jwt.version = "*"
rest.version = "0.3.0"
spicedb.version = "2"
"#;

    fs::write(project_path.join("grafbase.toml"), grafbase_toml_content).unwrap();

    // First run update without a lockfile. This should error.
    {
        // Run extension update command without a lockfile
        let mut update_command = process::Command::new(cargo_bin("grafbase"));
        update_command
            .args(["extension", "update", "--name", "echo", "--name", "jwt"])
            .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
            .current_dir(&project_path);

        let update_output = update_command.output().unwrap();

        assert!(!update_output.status.success());

        assert_debug_snapshot!((String::from_utf8(update_output.stdout), String::from_utf8(update_output.stderr)), @r#"
        (
            Ok(
                "",
            ),
            Ok(
                "Error: ❌ No lockfile found, please run `grafbase extension update` without --name first\n",
            ),
        )
        "#);
    }

    fs::write(project_path.join("grafbase.toml"), grafbase_toml_content).unwrap();

    fs::write(
        project_path.join("grafbase-extensions.lock"),
        r#"
        version = "1"

        [[extensions]]
        name = "echo"
        version = "1.1.1"

        [[extensions]]
        name = "jwt"
        version = "0.19.7"

        [[extensions]]
        name = "rest"
        version = "0.3.0"

        [[extensions]]
        name = "spicedb"
        version = "2.7.0"
        "#,
    )
    .unwrap();

    // Setup mock server
    let mock_server = wiremock::MockServer::start().await;

    // Mock response for extension version query
    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "extensionVersionsByVersionRequirement": [
                    { "__typename": "ExtensionVersion", "version": "1.1.3", "extension": { "name": "echo" } },
                    { "__typename": "ExtensionVersion", "version": "0.20.20", "extension": { "name": "jwt" } },
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Run extension update command
    let mut update_command = process::Command::new(cargo_bin("grafbase"));
    update_command
        .args(["extension", "update", "--name", "echo", "--name", "jwt"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .current_dir(&project_path);

    let update_output = update_command.output().unwrap();

    if !update_output.status.success() {
        panic!("Update failed\n{update_output:#?}");
    }

    let lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    insta::assert_snapshot!(&lockfile_contents);
}
