use common::environment::Project;

use crate::{consts::DOT_ENV_FILE_NAME, servers::EnvironmentName};

#[allow(deprecated)] // https://github.com/dotenv-rs/dotenv/pull/54
pub fn variables(environment_name: EnvironmentName) -> impl Iterator<Item = (String, String)> {
    let project = Project::get();
    let dot_env_file_path = project
        .schema_path
        .path()
        .parent()
        .expect("must be defined")
        .join(DOT_ENV_FILE_NAME);
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
        .chain(std::iter::once((
            "GRAFBASE_ENV".to_string(),
            match environment_name {
                EnvironmentName::Production => "production".to_string(),
                EnvironmentName::Dev => "dev".to_string(),
                EnvironmentName::None => String::new(),
            },
        )))
}
