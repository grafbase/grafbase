#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::ConfigType;
use serde_json::json;
use utils::environment::Environment;

#[test]
fn init_ts_existing_package_json() {
    let env = Environment::init();

    env.write_json_file_to_project(
        "package.json",
        &json!({
          "name": "test",
          "version": "1.0.0",
          "description": "",
          "main": "index.js",
          "keywords": [],
          "author": "",
          "license": "ISC"
        }),
    );

    let output = env.grafbase_init_output(ConfigType::TypeScript);
    println!("stdout: `{}`", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.stderr.is_empty(),
        "stderr should be empty, got: `{}`",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("grafbase.config.ts").exists());

    let package_json = env.load_file_from_project("package.json");

    insta::assert_snapshot!(&package_json, @r###"
    {
      "name": "test",
      "version": "1.0.0",
      "description": "",
      "main": "index.js",
      "keywords": [],
      "author": "",
      "license": "ISC",
      "devDependencies": {
        "@grafbase/sdk": "~0.3.1"
      }
    }
    "###);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn init_ts_new_project() {
    let env = Environment::init();
    let output = env.grafbase_init_output(ConfigType::TypeScript);
    println!("stdout: `{}`", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.stderr.is_empty(),
        "stderr should be empty, got: `{}`",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("grafbase.config.ts").exists());
    assert!(env.directory.join("package.json").exists());

    let package_json: serde_json::Value = serde_json::from_str(&env.load_file_from_project("package.json")).unwrap();
    let package_json = package_json.as_object().unwrap().get("devDependencies").unwrap();

    insta::assert_json_snapshot!(&package_json, @r###"
        {
          "@grafbase/sdk": "~0.3.1"
        }
    "###);
}
