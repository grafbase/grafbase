#![allow(unused_crate_dependencies)]
mod utils;

use backend::{api::consts::PROJECT_METADATA_FILE, project::ConfigType};
use common::consts::DOT_GRAFBASE_DIRECTORY_NAME;
use utils::environment::Environment;

#[test]
fn link_success() {
    let env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);

    let correct_ulid = "11H70FF572W29JXMG77JAB4KK0";

    let link_output = env.grafbase_link_non_interactive(correct_ulid);

    assert!(link_output.status.success());

    assert!(env
        .directory_path
        .join(DOT_GRAFBASE_DIRECTORY_NAME)
        .join(PROJECT_METADATA_FILE)
        .exists());

    assert!(std::fs::read_to_string(
        env.directory_path
            .join(DOT_GRAFBASE_DIRECTORY_NAME)
            .join(PROJECT_METADATA_FILE)
    )
    .unwrap()
    .contains(correct_ulid));
}

#[test]
fn link_invalid_ulid() {
    let env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);

    let link_output = env.grafbase_link_non_interactive("*1H70FF572W29JXMG77JAB4KK0");

    assert!(!link_output.status.success());

    assert!(std::str::from_utf8(&link_output.stderr).unwrap().contains("character"));

    let link_output = env.grafbase_link_non_interactive("11H7");

    assert!(!link_output.status.success());

    assert!(std::str::from_utf8(&link_output.stderr).unwrap().contains("length"));
}
