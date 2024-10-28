use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    consts::{CREDENTIALS_FILE, DOT_GRAFBASE_DIRECTORY_NAME, GRAFBASE_HOME},
    errors::CommonError,
};

use std::{borrow::Cow, env, fs::read_to_string, path::PathBuf, sync::OnceLock};

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub access_token: String,
    // should not be used directly, use `api_url` on `PlatformData` instead
    api_url: Option<String>,
}

impl Credentials {
    pub fn new(access_token: String, api_url: String) -> Self {
        Self {
            access_token,
            api_url: Some(api_url),
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Credentials {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}

pub enum LoginState {
    LoggedOut,
    LoggedIn(Credentials),
}

/// a static representation of the current environment
///
/// must be initialized before use
pub struct Environment {
    /// the path of `$HOME/.grafbase`, the user level local developer tool cache directory
    pub user_dot_grafbase_path: PathBuf,
    pub warnings: Vec<Warning>,
}

impl Environment {
    /// initializes the static Environment instance
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
            .map_err(|_| ())
            .expect("cannot) set environment twice");

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

pub struct PlatformData {
    pub api_url: String,
    pub dashboard_url: String,
    pub auth_url: String,
    pub login_state: LoginState,
}

impl PlatformData {
    /// initializes the static ApiData instance
    pub fn try_init() -> Result<(), CommonError> {
        let dashboard_url = std::env::var("GRAFBASE_DASHBOARD_URL")
            .ok()
            .unwrap_or_else(|| "https://app.grafbase.com".to_string());

        let mut auth_url = Url::parse(&dashboard_url).map_err(|_| CommonError::InvalidDashboardUrl)?;

        auth_url
            .path_segments_mut()
            .map_err(|_| CommonError::InvalidDashboardUrl)?
            .extend(["auth", "cli"]);

        let auth_url = auth_url.to_string();

        let login_state = get_login_state()?;

        let api_url = std::env::var("GRAFBASE_API_URL")
            .ok()
            .or_else(|| match login_state {
                LoginState::LoggedOut => None,
                LoginState::LoggedIn(ref credentials) => credentials.api_url.clone(),
            })
            .unwrap_or_else(|| "https://api.grafbase.com/graphql".to_string());

        PLATFORM_DATA
            .set(Self {
                dashboard_url,
                api_url,
                auth_url,
                login_state,
            })
            .map_err(|_| ())
            .expect("cannot) set platform data twice");

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
        match PLATFORM_DATA.get() {
            Some(api_data) => api_data,
            // must be initialized in `main`
            #[allow(clippy::panic)]
            None => panic!("the platform data object is uninitialized"),
        }
    }
}

/// static singleton for the environment struct
static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

/// static singleton for the api data struct
static PLATFORM_DATA: OnceLock<PlatformData> = OnceLock::new();

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

fn get_login_state() -> Result<LoginState, CommonError> {
    // if there's an access token in the environment, completely ignore the credentials file
    // including any previous api_url (we can make this more granular later if needed)
    if let Ok(token) = env::var("GRAFBASE_ACCESS_TOKEN") {
        return Ok(LoginState::LoggedIn(Credentials {
            access_token: token,
            api_url: None,
        }));
    }

    let environment = Environment::get();

    match environment.user_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Ok(LoginState::LoggedOut),
        Err(error) => return Err(CommonError::ReadUserDotGrafbaseFolder(error)),
    }

    let credentials_file_path = environment.user_dot_grafbase_path.join(CREDENTIALS_FILE);

    match credentials_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Ok(LoginState::LoggedOut),
        Err(error) => return Err(CommonError::ReadCredentialsFile(error)),
    }

    let credential_file = read_to_string(environment.user_dot_grafbase_path.join(CREDENTIALS_FILE))
        .map_err(CommonError::ReadCredentialsFile)?;

    let credentials: Credentials = match serde_json::from_str(&credential_file) {
        Ok(credentials) => credentials,
        Err(_) => return Err(CommonError::CorruptCredentialsFile),
    };

    Ok(LoginState::LoggedIn(credentials))
}
