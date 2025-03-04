use crate::cargo_bin;
use std::fs;
use std::process;
use tempfile::tempdir;
use wiremock::matchers;

#[tokio::test]
async fn install_with_up_to_date_lockfile() {
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

    let original_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    // Now run grafbase install

    wiremock::Mock::given(matchers::method("GET"))
        .and(matchers::path_regex("/extensions.*"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json("{}"))
        .mount(&mock_server)
        .await;

    let mut install_command = process::Command::new(cargo_bin("grafbase"));
    install_command
        .args(["extension", "install"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .env("EXTENSION_REGISTRY_URL", mock_server.uri())
        .current_dir(&project_path);

    let install_output = install_command.output().unwrap();

    if !install_output.status.success() {
        panic!("Install failed\n{install_output:#?}");
    }

    let updated_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    // Check if lockfile was updated
    assert_eq!(original_lockfile_contents, updated_lockfile_contents);
}

#[tokio::test]
async fn install_with_no_lockfile() {
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

    // Now run grafbase install

    wiremock::Mock::given(matchers::method("GET"))
        .and(matchers::path_regex("/extensions.*"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json("{}"))
        .mount(&mock_server)
        .await;

    let mut install_command = process::Command::new(cargo_bin("grafbase"));
    install_command
        .args(["extension", "install"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .env("EXTENSION_REGISTRY_URL", mock_server.uri())
        .current_dir(&project_path);

    let install_output = install_command.output().unwrap();

    if !install_output.status.success() {
        panic!("Install failed\n{install_output:#?}");
    }

    let updated_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    insta::assert_snapshot!(updated_lockfile_contents, @r#"
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
    "#);
}

#[tokio::test]
async fn install_with_lockfile_with_one_missing_one_outdated_and_one_extra_extension() {
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

    // foo isn't in the config
    // rest doesn't match the version in the config
    // echo is not completely up to date, but it won't be updated because it is locked
    // spicedb is in the config but missing here
    let extensions_lock = r#"
        version = "1"

        [[extensions]]
        name = "echo"
        version = "1.1.0"

        [[extensions]]
        name = "jwt"
        version = "0.19.7"

        [[extensions]]
        name = "foo"
        version = "0.19.7"

        [[extensions]]
        name = "rest"
        version = "0.2.0"
    "#;

    fs::write(project_path.join("grafbase-extensions.lock"), extensions_lock).unwrap();

    // Setup mock server
    let mock_server = wiremock::MockServer::start().await;

    // Mock response for extension version query
    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/graphql"))
        .and(matchers::body_partial_json(serde_json::json!({
            "variables":{"requirements":[{"extensionName":"rest","version":"^0.3.0"},{"extensionName":"spicedb","version":"^2"}]},
        })))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "data": {
            "extensionVersionsByVersionRequirement": [
                { "__typename": "ExtensionVersion", "version": "0.3.0", "extension": { "name": "rest" } },
                { "__typename": "ExtensionVersion", "version": "2.7.0", "extension": { "name": "spicedb" } },
            ]
        }
        })))
        .mount(&mock_server)
        .await;

    // Now run grafbase install

    wiremock::Mock::given(matchers::method("GET"))
        .and(matchers::path_regex("/extensions.*"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json("{}"))
        .mount(&mock_server)
        .await;

    let mut install_command = process::Command::new(cargo_bin("grafbase"));
    install_command
        .args(["extension", "install"])
        .env("GRAFBASE_API_URL", format!("{}/graphql", mock_server.uri()))
        .env("GRAFBASE_ACCESS_TOKEN", "test-value-of-the-access-token")
        .env("EXTENSION_REGISTRY_URL", mock_server.uri())
        .current_dir(&project_path);

    let install_output = install_command.output().unwrap();

    if !install_output.status.success() {
        panic!("Install failed\n{install_output:#?}");
    }

    let updated_lockfile_contents = std::fs::read_to_string(project_path.join("grafbase-extensions.lock")).unwrap();

    insta::assert_snapshot!(updated_lockfile_contents, @r#"
    version = "1"

    [[extensions]]
    name = "echo"
    version = "1.1.0"

    [[extensions]]
    name = "jwt"
    version = "0.19.7"

    [[extensions]]
    name = "rest"
    version = "0.3.0"

    [[extensions]]
    name = "spicedb"
    version = "2.7.0"
    "#);
}
