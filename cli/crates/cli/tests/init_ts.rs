#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::ConfigType;
use serde_json::json;
use utils::environment::Environment;

#[test]
fn init_ts() {
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

    env.grafbase_init_output(ConfigType::TypeScript);

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
        "@grafbase/sdk": "~0.0.20"
      }
    }
    "###);
}
