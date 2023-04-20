use common::environment::Environment;

use crate::consts::DOT_ENV_FILE;

#[allow(deprecated)] // https://github.com/dotenv-rs/dotenv/pull/54
pub fn variables() -> impl Iterator<Item = (String, String)> {
    let environment = Environment::get();
    let dot_env_file_path = environment.project_grafbase_path.join(DOT_ENV_FILE);
    // We don't use dotenv::dotenv() as we don't want to pollute the process' environment.
    // Doing otherwise would make us unable to properly refresh it whenever any of the .env files
    // changes which is something we may want to do in the future.
    std::env::vars()
        .chain(
            dotenv::from_path_iter(dot_env_file_path)
                .into_iter()
                .flatten()
                .filter_map(Result::ok),
        )
        .chain(std::iter::once(("GRAFBASE_ENV".to_string(), "dev".to_string())))
}
