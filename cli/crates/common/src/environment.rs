#![allow(dead_code)]

use crate::consts::{AUTHORIZERS_DIRECTORY_NAME, GENERATED_SCHEMAS_DIR, GRAFBASE_DIRECTORY_NAME};
use crate::errors::BunNotFound;
use crate::types::UdfKind;
use crate::{
    consts::{
        BUN_DIRECTORY_NAME, DOT_GRAFBASE_DIRECTORY_NAME, GRAFBASE_HOME, GRAFBASE_SCHEMA_FILE_NAME,
        GRAFBASE_TS_CONFIG_FILE_NAME, PACKAGE_JSON_DEV_DEPENDENCIES, PACKAGE_JSON_FILE_NAME, REGISTRY_FILE,
        RESOLVERS_DIRECTORY_NAME,
    },
    errors::CommonError,
};
use serde_json::{Map, Value};
use std::fs::File;
use std::io::BufReader;
use std::{
    borrow::Cow,
    env, fs, io,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, OnceLock},
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

/// Points to the location of the Grafbase schema file.
#[derive(Debug)]
pub struct GrafbaseSchemaPath {
    location: SchemaLocation,
}

impl GrafbaseSchemaPath {
    /// The location of the schema file.
    #[must_use]
    pub fn location(&self) -> &SchemaLocation {
        &self.location
    }

    fn ts_config(path: PathBuf) -> Self {
        Self {
            location: SchemaLocation::TsConfig(path),
        }
    }

    fn graphql(path: PathBuf) -> Self {
        Self {
            location: SchemaLocation::Graphql(path),
        }
    }

    pub fn parent(&self) -> Option<&Path> {
        self.path().parent()
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        use SchemaLocation::{Graphql, TsConfig};
        match self.location() {
            TsConfig(path) | Graphql(path) => path,
        }
    }
}

#[derive(Debug)]
pub struct Warning {
    message: Cow<'static, str>,
    hint: Option<Cow<'static, str>>,
}

impl Warning {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            hint: None,
        }
    }

    #[must_use]
    pub fn with_hint(mut self, message: impl Into<Cow<'static, str>>) -> Self {
        self.hint = Some(message.into());
        self
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

#[derive(Debug)]
pub struct Project {
    /// the path of the (assumed) user project root (`$PROJECT`), the nearest ancestor directory
    /// with a `schema.graphql` file or a `grafbase.config.ts` file
    // FIXME: Temporarily, it can also be the parent directory of the `grafbase` directory, until we phase that out.
    pub path: PathBuf,
    /// the path of the Grafbase schema, in the nearest ancestor directory with
    /// said directory and file
    pub schema_path: GrafbaseSchemaPath,
    /// the path of `$PROJECT/.grafbase/`, the Grafbase local developer tool cache and database directory,
    /// in the nearest ancestor directory with `grafbase/schema.graphql`
    pub dot_grafbase_directory_path: PathBuf,
    /// the path of `$PROJECT/.grafbase/registry.json`, the registry derived from `schema.graphql`,
    /// in the nearest ancestor directory with a `grafbase/schema.graphql` file
    pub registry_path: PathBuf,
    /// the location of package.json in '$PROJECT' or its parent
    pub package_json_path: Option<PathBuf>,
}

impl Project {
    /// the path of the directory containing the sources corresponding to the UDF type (resolvers, authorizers).
    #[must_use]
    pub fn udfs_source_path(&self, kind: UdfKind) -> std::path::PathBuf {
        let subdirectory_name = match kind {
            UdfKind::Resolver => RESOLVERS_DIRECTORY_NAME,
            UdfKind::Authorizer => AUTHORIZERS_DIRECTORY_NAME,
        };
        self.schema_path
            .parent()
            .expect("must have a parent")
            .join(subdirectory_name)
    }

    /// the path of the directory containing the build artifacts corresponding to the UDF type (resolvers, authorizers).
    #[must_use]
    pub fn udfs_build_artifact_path(&self, kind: UdfKind) -> std::path::PathBuf {
        let subdirectory_name = match kind {
            UdfKind::Resolver => RESOLVERS_DIRECTORY_NAME,
            UdfKind::Authorizer => AUTHORIZERS_DIRECTORY_NAME,
        };
        self.dot_grafbase_directory_path.join(subdirectory_name)
    }

    // reads and deserializes to json the contents of `&self.registry_path` in a blocking fashion
    pub fn registry(&self) -> Result<Value, CommonError> {
        let file = File::open(&self.registry_path)
            .map_err(|err| CommonError::RegistryRead(self.registry_path.clone(), err))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|err| CommonError::RegistryDeserialization(self.registry_path.clone(), err))
    }

    pub fn sdl_location(&self) -> PathBuf {
        match self.schema_path.location() {
            SchemaLocation::TsConfig(_) => self
                .dot_grafbase_directory_path
                .join(GENERATED_SCHEMAS_DIR)
                .join(GRAFBASE_SCHEMA_FILE_NAME),
            SchemaLocation::Graphql(path) => path.clone(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub project_id: String,
}

/// a static representation of the current environment
///
/// must be initialized before use
#[derive(Debug)]
pub struct Environment {
    /// data related to the current project
    pub project: Option<Project>,
    /// the path of `$HOME/.grafbase`, the user level local developer tool cache directory
    pub user_dot_grafbase_path: PathBuf,
    /// warnings when loading the environment
    pub warnings: Vec<Warning>,
    /// the path within `$HOME/.grafbase` where bun gets installed
    pub bun_installation_path: PathBuf,
}

impl Environment {
    /// the path within `$HOME/.grafbase` where the bun executable is located
    pub fn bun_executable_path(&self) -> Result<PathBuf, BunNotFound> {
        if cfg!(windows) {
            return Ok(self.bun_installation_path.join("bun.exe"));
        }

        if is_nixos() {
            nixos_check_bun_is_available()?;
            return Ok("bun".into());
        }

        Ok(self.bun_installation_path.join("bun"))
    }
}

/// static singleton for the environment struct
static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

fn nixos_check_bun_is_available() -> Result<(), BunNotFound> {
    static CHECKED: AtomicBool = AtomicBool::new(false);

    if CHECKED.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    if which::which("bun").is_err() {
        return Err(BunNotFound);
    }

    Ok(())
}

/// Returns `true` if we can detect that we are running on NixOS. This is relevant here because downloading binaries from projects will almost never work out the box on NixOS.
pub fn is_nixos() -> bool {
    static IS_NIXOS: OnceLock<bool> = OnceLock::new();

    *IS_NIXOS.get_or_init(|| Path::new("/etc/NIXOS").exists())
}

pub fn system_bun_path() -> Result<PathBuf, which::Error> {
    which::which("bun")
}

#[must_use]
pub fn get_default_user_dot_grafbase_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(DOT_GRAFBASE_DIRECTORY_NAME))
}

pub fn get_user_dot_grafbase_path_from_env() -> Option<PathBuf> {
    env::var(GRAFBASE_HOME)
        .ok()
        .map(PathBuf::from)
        .map(|env_override| env_override.join(DOT_GRAFBASE_DIRECTORY_NAME))
}

pub fn get_user_dot_grafbase_path(r#override: Option<PathBuf>) -> Option<PathBuf> {
    r#override
        .map(|r#override| r#override.join(DOT_GRAFBASE_DIRECTORY_NAME))
        .or_else(get_user_dot_grafbase_path_from_env)
        .or_else(get_default_user_dot_grafbase_path)
}

impl Project {
    fn try_init(warnings: &mut Vec<Warning>) -> Result<Self, CommonError> {
        let (path, schema_path) = get_project_grafbase_path(warnings)?.ok_or(CommonError::FindGrafbaseDirectory)?;

        let dot_grafbase_directory_path = path.join(DOT_GRAFBASE_DIRECTORY_NAME);
        let registry_path = dot_grafbase_directory_path.join(REGISTRY_FILE);
        let package_json_path = [path.as_path(), path.parent().expect("must have a parent")]
            .into_iter()
            .map(|candidate| candidate.join(PACKAGE_JSON_FILE_NAME))
            .find(|candidate| candidate.exists());

        Ok(Project {
            path,
            schema_path,
            dot_grafbase_directory_path,
            registry_path,
            package_json_path,
        })
    }

    /// returns a reference to the static Project instance
    ///
    /// # Panics
    ///
    /// panics if the Environment object was not previously initialized using `Environment::try_init_with_project()`
    #[must_use]
    #[track_caller]
    pub fn get() -> &'static Project {
        Environment::get()
            .project
            .as_ref()
            .expect("Environment must be initialized with try_init_with_project to access the project details")
    }
}

impl Environment {
    /// initializes the static Environment instance, including the current project details
    ///
    /// # Errors
    ///
    /// returns [`CommonError::FindGrafbaseDirectory`] if the grafbase directory path cannot be read
    ///
    /// returns [`CommonError::FindHomeDirectory`] if the home directory is not found
    pub fn try_init_with_project(home_override: Option<PathBuf>) -> Result<(), CommonError> {
        let mut warnings = Vec::new();

        let user_dot_grafbase_path = get_user_dot_grafbase_path(home_override).ok_or(CommonError::FindHomeDirectory)?;

        let bun_installation_path = user_dot_grafbase_path.join(BUN_DIRECTORY_NAME);

        let project = Project::try_init(&mut warnings)?;

        ENVIRONMENT
            .set(Self {
                project: Some(project),
                user_dot_grafbase_path,
                warnings,
                bun_installation_path,
            })
            .expect("cannot set environment twice");

        Ok(())
    }

    /// initializes the static Environment instance, outside the context of a project
    ///
    /// # Errors
    ///
    /// returns [`CommonError::FindHomeDirectory`] if the home directory is not found
    pub fn try_init(home_override: Option<PathBuf>) -> Result<(), CommonError> {
        let user_dot_grafbase_path = get_user_dot_grafbase_path(home_override).ok_or(CommonError::FindHomeDirectory)?;

        let bun_installation_path = user_dot_grafbase_path.join(BUN_DIRECTORY_NAME);

        ENVIRONMENT
            .set(Self {
                project: None,
                user_dot_grafbase_path,
                warnings: Vec::new(),
                bun_installation_path,
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
    #[track_caller]
    pub fn get() -> &'static Self {
        match ENVIRONMENT.get() {
            Some(environment) => environment,
            // must be initialized in `main`
            #[allow(clippy::panic)]
            None => panic!("the environment object is uninitialized"),
        }
    }
}

/// searches for the closest ancestor directory which contains either
/// a "grafbase.config.ts" or a "schema.graphql" file.
///
/// # Errors
///
/// returns [`CommonError::ReadCurrentDirectory`] if the current directory path cannot be read
fn get_project_grafbase_path(
    warnings: &mut Vec<Warning>,
) -> Result<Option<(PathBuf, GrafbaseSchemaPath)>, CommonError> {
    Ok(env::current_dir()
        .map_err(|_| CommonError::ReadCurrentDirectory)?
        .ancestors()
        .find_map(|ancestor| {
            find_grafbase_configuration(ancestor, warnings)
                .map(|grafbase_schema_path| (ancestor.to_owned(), grafbase_schema_path))
        }))
}

fn find_grafbase_configuration(path: &Path, warnings: &mut Vec<Warning>) -> Option<GrafbaseSchemaPath> {
    // FIXME: Deprecate the last look-up path and remove it.
    let search_paths: [std::borrow::Cow<'_, Path>; 2] = [path.into(), path.join(GRAFBASE_DIRECTORY_NAME).into()];

    search_paths.into_iter().find_map(|search_path| {
        let tsconfig_file_path = search_path.join(GRAFBASE_TS_CONFIG_FILE_NAME);
        let schema_graphql_file_path = search_path.join(GRAFBASE_SCHEMA_FILE_NAME);
        match (tsconfig_file_path.is_file(), schema_graphql_file_path.is_file()) {
            (true, true) => {
                let warning = Warning::new("Found both grafbase.config.ts and schema.graphql files")
                    .with_hint("Delete one of them to avoid conflicts");
                warnings.push(warning);
                Some(GrafbaseSchemaPath::ts_config(tsconfig_file_path))
            }
            (true, false) => Some(GrafbaseSchemaPath::ts_config(tsconfig_file_path)),
            (false, true) => Some(GrafbaseSchemaPath::graphql(schema_graphql_file_path)),
            (false, false) => None,
        }
    })
}

pub fn add_dev_dependency_to_package_json(project_dir: &Path, package: &str, version: &str) -> Result<(), CommonError> {
    let package_json_path = project_dir.join(PACKAGE_JSON_FILE_NAME);

    let mut package_json = if package_json_path.exists() {
        let file = fs::File::open(&package_json_path).map_err(CommonError::AccessPackageJson)?;
        let Ok(Value::Object(package_json)) = serde_json::from_reader(&file) else {
            return Err(CommonError::AccessPackageJson(io::Error::new(
                io::ErrorKind::InvalidData,
                "the file is not a JSON object",
            )));
        };
        package_json
    } else {
        let name = project_dir
            .file_name()
            .map_or(Cow::Borrowed("grafbase-project"), std::ffi::OsStr::to_string_lossy);
        match serde_json::json!(
        {
          "name": name,
          "version": "1.0.0",
          "description": "",
          "main": "index.js",
          "scripts": {
            "test": "echo \"Error: no test specified\" && exit 1"
          },
          "keywords": [],
          "author": "",
          "license": "ISC",
        }
        ) {
            Value::Object(package_json) => package_json,
            _ => unreachable!("must be an object"),
        }
    };

    match package_json
        .entry(PACKAGE_JSON_DEV_DEPENDENCIES)
        .or_insert_with(|| Value::Object(Map::new()))
    {
        Value::Object(ref mut obj) if !obj.contains_key(package) => {
            obj.insert(package.to_string(), Value::String(version.to_string()));
        }
        Value::Object(_) => return Ok(()),
        _ => {
            return Err(CommonError::AccessPackageJson(io::Error::new(
                io::ErrorKind::InvalidData,
                "the devDependencies value is not an object",
            )));
        }
    }

    let file = fs::File::create(&package_json_path).map_err(CommonError::AccessPackageJson)?;
    serde_json::to_writer_pretty(&file, &package_json).map_err(CommonError::SerializePackageJson)?;

    Ok(())
}
