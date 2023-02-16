#![allow(dead_code)]

use crate::{
    consts::{DOT_GRAFBASE_DIRECTORY, GRAFBASE_DIRECTORY, GRAFBASE_SCHEMA, REGISTRY_FILE, RESOLVERS_DIRECTORY_NAME},
    errors::CommonError,
};
use once_cell::sync::OnceCell;
use std::{
    env,
    path::{Path, PathBuf},
};

/// a static representation of the current environment
///
/// must be initialized before use
#[derive(Debug)]
pub struct Environment {
    /// the path of `$PROJECT/grafbase/schema.graphql`, the Grafbase schema,
    /// in the nearest ancestor directory with said directory and file
    pub project_grafbase_schema_path: PathBuf,
}

impl Environment {
    /// the path of `$PROJECT/grafbase/`, the Grafbase schema directory in the nearest ancestor directory
    /// with `grafbase/schema.graphql`
    #[must_use]
    pub fn project_grafbase_path(&self) -> &Path {
        self.project_grafbase_schema_path
            .parent()
            .expect("the schema directory must have a parent by definiton")
    }
    /// the path of the (assumed) user project root (`$PROJECT`), the nearest ancestor directory
    /// with a `grafbase/schema.graphql` file
    #[must_use]
    pub fn project_path(&self) -> &Path {
        self.project_grafbase_path()
            .parent()
            .expect("the grafbase directory must have a parent directory by definition")
    }
    /// the path of the `grafbase/resolvers` directory.
    #[must_use]
    pub fn resolvers_path(&self) -> PathBuf {
        self.project_grafbase_path().join(RESOLVERS_DIRECTORY_NAME)
    }
    /// the path of `$PROJECT/.grafbase/`, the Grafbase local developer tool cache and database directory,
    /// in the nearest ancestor directory with `grafbase/schema.graphql`
    #[must_use]
    pub fn project_dot_grafbase_path(&self) -> PathBuf {
        self.project_path().join(DOT_GRAFBASE_DIRECTORY)
    }
    /// the path of `$HOME/.grafbase`, the user level local developer tool cache directory
    pub fn user_dot_grafbase_path(&self) -> PathBuf {
        let home = dirs::home_dir().map_or_else(
            || std::borrow::Cow::Borrowed(self.project_grafbase_path()),
            std::borrow::Cow::Owned,
        );
        home.join(DOT_GRAFBASE_DIRECTORY)
    }
    /// the path of `$PROJECT/.grafbase/registry.json`, the registry derived from `schema.graphql`,
    /// in the nearest ancestor directory with a `grabase/schema.graphql` file
    #[must_use]
    pub fn project_grafbase_registry_path(&self) -> PathBuf {
        self.project_dot_grafbase_path().join(REGISTRY_FILE)
    }
}

/// static singleton for the environment struct
static ENVIRONMENT: OnceCell<Environment> = OnceCell::new();

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
        ENVIRONMENT
            .set(Self {
                project_grafbase_schema_path,
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

    /// searches for the closest ancestor directory
    /// named "grafbase" which contains a "schema.graphql" file.
    /// if already inside a `grafbase` directory, looks for `schema.graphql` inside the current ancestor as well
    ///
    /// # Errors
    ///
    /// returns [`CommonError::ReadCurrentDirectory`] if the current directory path cannot be read
    fn get_project_grafbase_path() -> Result<Option<PathBuf>, CommonError> {
        let project_grafbase_path = env::current_dir()
            .map_err(|_| CommonError::ReadCurrentDirectory)?
            .ancestors()
            .find_map(|ancestor| {
                let mut path = PathBuf::from(ancestor);

                // if we're looking at a directory called `grafbase`, also check for the schema in the current directory
                if let Some(first) = path.components().next() {
                    if Path::new(&first) == PathBuf::from(GRAFBASE_DIRECTORY) {
                        path.push(GRAFBASE_SCHEMA);
                        if path.is_file() {
                            return Some(path);
                        }
                        path.pop();
                    }
                }

                path.push([GRAFBASE_DIRECTORY, GRAFBASE_SCHEMA].iter().collect::<PathBuf>());

                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            });

        Ok(project_grafbase_path)
    }
}
