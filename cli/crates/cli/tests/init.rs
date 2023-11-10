#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::ConfigType;
use utils::environment::Environment;

#[test]
fn init_1() {
    let env = Environment::init();

    env.grafbase_init(ConfigType::GraphQL);
    assert!(env.directory_path.exists());
    assert!(env.directory_path.join("schema.graphql").exists());
}

#[test]
fn init_2() {
    let env = Environment::init();

    env.grafbase_init(ConfigType::GraphQL);
    assert!(env.directory_path.exists());
    assert!(env.directory_path.join("schema.graphql").exists());

    let output = env.grafbase_init_output(ConfigType::GraphQL);
    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr).unwrap().contains("already exists"));
}

#[test]
fn init_3() {
    let env = Environment::init();

    env.grafbase_init_template(None, "graphql-github");
    assert!(env.directory_path.exists());
    assert!(env.directory_path.join("grafbase/grafbase.config.ts").exists());
    assert!(env.directory_path.join("package.json").exists());
}

#[test]
fn init_4() {
    let env = Environment::init();

    env.grafbase_init_template(Some("new-project"), "graphql-github");
    let directory_path = env.directory_path.join("new-project");
    assert!(directory_path.exists());
    assert!(directory_path.join("grafbase/grafbase.config.ts").exists());
}

#[test]
fn init_5() {
    let env = Environment::init();

    env.grafbase_init_template(
        None,
        "https://github.com/grafbase/grafbase/tree/main/templates/graphql-github",
    );
    assert!(env.directory_path.join("grafbase/grafbase.config.ts").exists());
    assert!(env.directory_path.join("package.json").exists());
}

#[test]
fn init_6() {
    let env = Environment::init();

    let output = env.grafbase_init_template_output(
        None,
        "https://example.com/grafbase/grafbase/tree/main/templates/graphql-github",
    );
    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("is not a supported template URL"));
}

#[test]
fn init_7() {
    let env = Environment::init();

    let output = env.grafbase_init_template_output(None, "https://github.com/grafbase/grafbase/tree/main/templates");
    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("could not find the provided template within the template repository"));
}

#[test]
fn init_8() {
    let env = Environment::init();

    // FIXME: this error message will change once we check for existing templates before downloading
    let output = env.grafbase_init_template_output(None, "not_a_template");
    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("could not find the provided template within the template repository"));
}
