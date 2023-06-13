mod utils;

use backend::project::ConfigType;
use utils::environment::Environment;

#[test]
fn init() {
    let env = Environment::init();

    env.grafbase_init(ConfigType::GraphQL);

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    let output = env.grafbase_init_output(ConfigType::GraphQL);

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr).unwrap().contains("already exists"));

    env.remove_grafbase_dir(None);

    env.grafbase_init_template(None, "todo");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir(None);

    env.grafbase_init_template(Some("new-project"), "todo");

    let directory = env.directory.join("new-project").join("grafbase");

    assert!(directory.exists());
    assert!(directory.join("schema.graphql").exists());

    env.remove_grafbase_dir(Some("new-project"));

    env.grafbase_init_template(None, "https://github.com/grafbase/grafbase/tree/main/templates/blog");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir(None);

    env.grafbase_init_template(
        Some("new-project"),
        "https://github.com/grafbase/grafbase/tree/main/templates/blog",
    );

    let directory = env.directory.join("new-project").join("grafbase");

    assert!(directory.exists());
    assert!(directory.join("schema.graphql").exists());

    env.remove_grafbase_dir(Some("new-project"));

    let output =
        env.grafbase_init_template_output(None, "https://example.com/grafbase/grafbase/tree/main/templates/blog");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("is not a supported template URL"));

    let output = env.grafbase_init_template_output(None, "https://github.com/grafbase/grafbase/tree/main/templates");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("could not find the provided template within the template repository"));

    // FIXME: this error message will change once we check for existing templates before downloading
    let output = env.grafbase_init_template_output(None, "not_a_template");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("could not find the provided template within the template repository"));
}
