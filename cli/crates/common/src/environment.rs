use crate::{
    consts::{DOT_GRAFBASE_DIRECTORY_NAME, GRAFBASE_HOME},
    errors::CommonError,
};

use std::{borrow::Cow, env, path::PathBuf, sync::OnceLock};

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

/// a static representation of the current environment
///
/// must be initialized before use
#[derive(Debug)]
pub struct Environment {
    /// the path of `$HOME/.grafbase`, the user level local developer tool cache directory
    pub user_dot_grafbase_path: PathBuf,
    pub warnings: Vec<Warning>,
}

impl Environment {
    /// initializes the static Environment instance, outside the context of a project
    ///
    /// # Errors
    ///
    /// returns [`CommonError::FindHomeDirectory`] if the home directory is not found
    pub fn try_init(home_override: Option<PathBuf>) -> Result<(), CommonError> {
        let user_dot_grafbase_path = get_user_dot_grafbase_path(home_override).ok_or(CommonError::FindHomeDirectory)?;

        ENVIRONMENT
            .set(Self {
                user_dot_grafbase_path,
                warnings: Vec::new(),
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

/// static singleton for the environment struct
static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

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
