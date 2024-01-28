#![allow(clippy::let_underscore_untyped)] // derivative

use crate::{environment::Environment, errors::CommonError};
use chrono::{DateTime, Utc};
use core::panic;
use derivative::Derivative;
use rudderanalytics::{
    client::RudderAnalytics,
    message::{Message, Track},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::{
    fmt::{self, Display},
    path::PathBuf,
};
use ulid::Ulid;

#[derive(Derivative, Serialize)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct Analytics {
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    client: RudderAnalytics,
    session_id: Ulid,
    start_time: DateTime<Utc>,
    version: String,
}

// TODO move this to [`Environment`]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::module_name_repetitions)]
pub struct GrafbaseConfig {
    pub enable_analytics: bool,
    pub anonymous_id: Option<Ulid>,
}

impl Display for GrafbaseConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&serde_json::to_string(&self).expect("must parse"))
    }
}

const GRAFBASE_CONFIG_FILE: &str = "config.json";

fn reverse_option<T>(option: &Option<T>) -> Option<()> {
    match option {
        Some(_) => None,
        None => Some(()),
    }
}

impl Analytics {
    pub fn init() -> Result<(), CommonError> {
        let write_key = option_env!("GRAFBASE_RUDDERSTACK_WRITE_KEY");
        let dataplane_url = option_env!("GRAFBASE_RUDDERSTACK_DATAPLANE_URL");
        let do_not_track = std::env::var("DO_NOT_TRACK").ok();

        let data = Self::read_data()?;

        let analytics_data_and_variables = write_key
            .zip(dataplane_url)
            .zip(reverse_option(&do_not_track))
            .zip(reverse_option(&data.filter(|data| !data.enable_analytics)));

        if analytics_data_and_variables.is_some() && !Self::config_exists()? {
            Self::init_data()?;
        }

        ANALYTICS
            .set(
                analytics_data_and_variables.map(|(((write_key, dataplane_url), ()), ())| Analytics {
                    client: RudderAnalytics::load(write_key.to_owned(), dataplane_url.to_owned()),
                    session_id: Ulid::new(),
                    start_time: Utc::now(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                }),
            )
            .expect("cannot set analytics twice");

        Ok(())
    }

    fn write_data(data: &GrafbaseConfig) -> Result<(), CommonError> {
        let data_location = Self::get_config_location();
        if let Some(parent) = data_location.parent() {
            std::fs::create_dir_all(parent).map_err(CommonError::CreateUserDotGrafbaseFolder)?;
        }
        std::fs::write(data_location, data.to_string()).map_err(CommonError::WriteAnalyticsDataFile)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("must parse")
    }

    #[must_use]
    pub fn get_context(&self) -> Value {
        self.to_value()
    }

    fn init_data() -> Result<(), CommonError> {
        Self::write_data(&GrafbaseConfig {
            enable_analytics: true,
            anonymous_id: Some(Ulid::new()),
        })
    }

    fn get_config_location() -> PathBuf {
        Environment::get().user_dot_grafbase_path.join(GRAFBASE_CONFIG_FILE)
    }

    fn config_exists() -> Result<bool, CommonError> {
        match Self::get_config_location().try_exists() {
            Ok(result) => Ok(result),
            Err(error) => Err(CommonError::ReadAnalyticsDataFile(error)),
        }
    }

    fn read_data() -> Result<Option<GrafbaseConfig>, CommonError> {
        if !Self::config_exists()? {
            return Ok(None);
        }

        let data_path = Self::get_config_location();

        let data_string = std::fs::read_to_string(data_path).map_err(CommonError::ReadAnalyticsDataFile)?;

        let data: GrafbaseConfig =
            serde_json::from_str(&data_string).map_err(|_| CommonError::CorruptAnalyticsDataFile)?;

        Ok(Some(data))
    }

    /// # Panics
    ///
    /// panics if the static [`ANALYTICS`] object was not previously initialized
    pub fn get() -> &'static Option<Self> {
        match ANALYTICS.get() {
            Some(analytics) => analytics,
            // must be initialized in `main`
            #[allow(clippy::panic)]
            None => panic!("the analytics object is uninitialized"),
        }
    }

    pub fn track(event_name: &str, properties: Option<Value>) {
        let event_name = event_name.to_owned();

        let Some(analytics) = Self::get() else {
            return;
        };
        let Some(anonymous_id) = Self::read_data().ok().flatten().and_then(|data| data.anonymous_id) else {
            return;
        };

        // FIXME: This should all be happening within an async context…
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("must succeed");
        runtime.block_on(async move {
            // Purposely ignoring errors.
            let _ = analytics
                .client
                .send(&Message::Track(Track {
                    event: event_name,
                    anonymous_id: Some(anonymous_id.to_string()),
                    properties,
                    context: Some(analytics.get_context()),
                    ..Default::default()
                }))
                .await;
            ()
        });
    }

    pub fn command_executed(command_name: &str, command_arguments: Option<Vec<&'static str>>) {
        let command_arguments = command_arguments.map(|arguments| arguments.join(","));
        Self::track(
            "Command Executed",
            Some(json!({ "commandName": command_name, "commandArguments": command_arguments })),
        );
    }
}

static ANALYTICS: OnceLock<Option<Analytics>> = OnceLock::new();
