mod utils;

use utils::environment::Environment;

#[test]
fn dev() {
    let env = Environment::init(4016);

    env.grafbase_init();

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    let output = env.grafbase_init_output();

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr).unwrap().contains("already exists"));

    env.remove_grafbase_dir();

    env.grafbase_init_template("todo");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir();

    env.grafbase_init_template("https://github.com/grafbase/grafbase/tree/main/templates/blog");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir();

    let output = env.grafbase_init_template_output("https://example.com/grafbase/grafbase/tree/main/templates/blog");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("is not a supported template URL"));

    let output = env.grafbase_init_template_output("https://github.com/grafbase/grafbase/tree/main/templates");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("no files were extracted from the template repository"));

    // FIXME: this error message will change once we check for existing templates before downloading
    let output = env.grafbase_init_template_output("not_a_template");

    assert!(!output.stderr.is_empty());
    assert!(std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("no files were extracted from the template repository"));
}
