#![allow(dead_code)]

use crate::{
    consts::{
        DATABASE_DIRECTORY, DOT_GRAFBASE_DIRECTORY, GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME,
        GRAFBASE_TS_CONFIG_FILE_NAME, REGISTRY_FILE, RESOLVERS_DIRECTORY_NAME,
    },
    errors::CommonError,
};
use once_cell::sync::OnceCell;
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum SchemaLocation {
    /// The path of `$PROJECT/grafbase/grafbase.config.ts`,
    /// if exits.
    TsConfig(PathBuf),
    /// The path of `$PROJECT/grafbase/schema.graphql`, the
    /// Grafbase schema, in the nearest ancestor directory
    /// with said directory and file
    Graphql(PathBuf),
}

/// Points to the location of the Grafabase schema file.
#[derive(Debug)]
pub struct GrafbaseSchemaPath {
    location: SchemaLocation,
    has_both_files: bool,
}

impl GrafbaseSchemaPath {
    /// The location of the schema file.
    pub fn location(&self) -> &SchemaLocation {
        &self.location
    }

    /// True, if we find both `schema.graphql` and `grafbase.config.ts`.
    pub fn has_both_files(&self) -> bool {
        self.has_both_files
    }

    fn ts_config(path: PathBuf) -> Self {
        Self {
            location: SchemaLocation::TsConfig(path),
            has_both_files: false,
        }
    }

    fn graphql(path: PathBuf) -> Self {
        Self {
            location: SchemaLocation::Graphql(path),
            has_both_files: false,
        }
    }

    fn parent(&self) -> Option<&Path> {
        use SchemaLocation::{Graphql, TsConfig};

        match self.location() {
            TsConfig(path) | Graphql(path) => path.parent(),
        }
    }
}

/// a static representation of the current environment
///
/// must be initialized before use
#[derive(Debug)]
pub struct Environment {
    /// the path of the (assumed) user project root (`$PROJECT`), the nearest ancestor directory
    /// with a `grafbase/schema.graphql` file
    pub project_path: PathBuf,
    /// the path of `$PROJECT/.grafbase/`, the Grafbase local developer tool cache and database directory,
    /// in the nearest ancestor directory with `grafbase/schema.graphql`
    pub project_dot_grafbase_path: PathBuf,
    /// the path of `$PROJECT/grafbase/`, the Grafbase schema directory in the nearest ancestor directory
    /// with `grafbase/schema.graphql`
    pub project_grafbase_path: PathBuf,
    /// the path of the Grafbase schema, in the nearest ancestor directory with
    /// said directory and file
    pub project_grafbase_schema_path: GrafbaseSchemaPath,
    /// the path of `$HOME/.grafbase`, the user level local developer tool cache directory
    pub user_dot_grafbase_path: PathBuf,
    /// the path of `$PROJECT/.grafbase/registry.json`, the registry derived from `schema.graphql`,
    /// in the nearest ancestor directory with a `grabase/schema.graphql` file
    pub project_grafbase_registry_path: PathBuf,
    /// the path of the `grafbase/resolvers` directory.
    pub resolvers_source_path: PathBuf,
    /// the path within `$PROJECT/.grafbase/` containing build artifacts for custom resolvers.
    pub resolvers_build_artifact_path: PathBuf,
    /// the path within '$PROJECT/.grafbase' containing the database
    pub database_directory_path: PathBuf,
}

/// static singleton for the environment struct
static ENVIRONMENT: OnceCell<Environment> = OnceCell::new();

#[must_use]
pub fn get_user_dot_grafbase_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(DOT_GRAFBASE_DIRECTORY))
}

impl Environment {
    /// initializes the static Environment instance
    ///
    /// # Errors
    ///
    /// returns [`CommonError::ReadCurrentDirectory`] if the current directory path cannot be read
    ///
    /// returns [`CommonError::FindGrafbaseDirectory`] if the grafbase directory is not found
    pub fn try_init() -> Result<(), CommonError> {
        let project_grafbase_schema_path =
            Self::get_project_grafbase_path()?.ok_or(CommonError::FindGrafbaseDirectory)?;

        let project_grafbase_path = project_grafbase_schema_path
            .parent()
            .expect("the schema directory must have a parent by definiton")
            .to_path_buf();

        let project_path = project_grafbase_path
            .parent()
            .expect("the grafbase directory must have a parent directory by definition")
            .to_path_buf();

        let user_dot_grafbase_path =
            get_user_dot_grafbase_path().unwrap_or_else(|| project_grafbase_path.join(DOT_GRAFBASE_DIRECTORY));

        let project_dot_grafbase_path = project_path.join(DOT_GRAFBASE_DIRECTORY);
        let project_grafbase_registry_path = project_dot_grafbase_path.join(REGISTRY_FILE);
        let resolvers_source_path = project_grafbase_path.join(RESOLVERS_DIRECTORY_NAME);
        let resolvers_build_artifact_path = project_dot_grafbase_path.join(RESOLVERS_DIRECTORY_NAME);
        let database_directory_path = project_dot_grafbase_path.join(DATABASE_DIRECTORY);

        ENVIRONMENT
            .set(Self {
                project_path,
                project_dot_grafbase_path,
                project_grafbase_path,
                project_grafbase_schema_path,
                user_dot_grafbase_path,
                project_grafbase_registry_path,
                resolvers_source_path,
                resolvers_build_artifact_path,
                database_directory_path,
            })
            .expect("cannot set environment twice");

        Ok(())
    }

    /// returns a reference to the static Environment instance
    ///
    /// # Panics
    ///
    /// panics if the Environment object was not previously initialized using `Environment::try_init()`
    #[must_use]
    pub fn get() -> &'static Self {
        match ENVIRONMENT.get() {
            Some(environment) => environment,
            // must be initialized in `main`
            #[allow(clippy::panic)]
            None => panic!("the environment object is uninitialized"),
        }
    }

    /// searches for the closest ancestor directory named "grafbase" which
    /// contains either a "grafbase.config.ts" or a "schema.graphql" file. if
    /// already inside a `grafbase` directory, looks for `schema.graphql` inside
    /// the current ancestor as well
    ///
    /// # Errors
    ///
    /// returns [`CommonError::ReadCurrentDirectory`] if the current directory path cannot be read
    fn get_project_grafbase_path() -> Result<Option<GrafbaseSchemaPath>, CommonError> {
        let ts_config = get_path_to_file_in_project_grafbase(GRAFBASE_TS_CONFIG_FILE_NAME)?;
        let gql = get_path_to_file_in_project_grafbase(GRAFBASE_SCHEMA_FILE_NAME)?;

        if let Some(path) = ts_config {
            let mut path = GrafbaseSchemaPath::ts_config(path);
            path.has_both_files = gql.is_some();

            Ok(Some(path))
        } else {
            Ok(gql.map(GrafbaseSchemaPath::graphql))
        }
    }
}

/// Find a path to the given file in the project's grafbase directory.
fn get_path_to_file_in_project_grafbase(file_name: &str) -> Result<Option<PathBuf>, CommonError> {
    let path_to_file = env::current_dir()
        .map_err(|_| CommonError::ReadCurrentDirectory)?
        .ancestors()
        .find_map(|ancestor| {
            let mut path = PathBuf::from(ancestor);

            // if we're looking at a directory called `grafbase`,
            // also check for the file in the current directory
            if let Some(first) = path.components().next() {
                if Path::new(&first) == PathBuf::from(GRAFBASE_DIRECTORY_NAME) {
                    path.push(file_name);

                    if path.is_file() {
                        return Some(path);
                    }

                    path.pop();
                }
            }

            path.push([GRAFBASE_DIRECTORY_NAME, file_name].iter().collect::<PathBuf>());

            if path.is_file() {
                Some(path)
            } else {
                None
            }
        });

    Ok(path_to_file)
}
