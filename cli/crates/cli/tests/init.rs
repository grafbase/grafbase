mod utils;

use utils::environment::Environment;

#[test]
fn dev() {
    let env = Environment::init(4016);

    env.grafbase_init();

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir();

    env.grafbase_init_template("todo");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());

    env.remove_grafbase_dir();

    env.grafbase_init_template("https://github.com/grafbase/grafbase/tree/main/templates/blog");

    assert!(env.directory.join("grafbase").exists());
    assert!(env.directory.join("grafbase").join("schema.graphql").exists());
}
