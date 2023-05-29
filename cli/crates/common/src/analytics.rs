#![allow(clippy::let_underscore_untyped)] // derivative

use crate::{environment::Environment, errors::CommonError};
use chrono::{DateTime, Utc};
use core::panic;
use derivative::Derivative;
use once_cell::sync::OnceCell;
use rudderanalytics::{
    client::RudderAnalytics,
    message::{Message, Track},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::PathBuf, thread};
use ulid::Ulid;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Analytics {
    #[derivative(Debug = "ignore")]
    client: RudderAnalytics,
    session_id: Ulid,
    start_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::module_name_repetitions)]
pub struct AnalyticsData {
    pub opt_out: bool,
    pub anonymous_id: Option<Ulid>,
}

impl ToString for AnalyticsData {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}

const ANALYTICS_DATA_FILE: &str = "analytics.json";

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
            .zip(do_not_track)
            .zip(reverse_option(&data.filter(|data| data.opt_out)));

        if analytics_data_and_variables.is_some() && !Self::data_exists()? {
            Self::init_data()?;
        }

        ANALYTICS
            .set(
                analytics_data_and_variables.map(|(((write_key, dataplane_url), _), _)| Analytics {
                    client: RudderAnalytics::load(write_key.to_owned(), dataplane_url.to_owned()),
                    session_id: Ulid::new(),
                    start_time: Utc::now(),
                }),
            )
            .expect("cannot set analytics twice");

        Ok(())
    }

    fn write_data(data: &AnalyticsData) -> Result<(), CommonError> {
        let data_location = Self::get_data_location();
        if let Some(parent) = data_location.parent() {
            std::fs::create_dir_all(parent).map_err(CommonError::CreateUserDotGrafbaseFolder)?;
        }
        std::fs::write(data_location, data.to_string()).map_err(CommonError::WriteAnalyticsDataFile)
    }

    fn init_data() -> Result<(), CommonError> {
        Self::write_data(&AnalyticsData {
            opt_out: false,
            anonymous_id: Some(Ulid::new()),
        })
    }

    fn get_data_location() -> PathBuf {
        Environment::get().user_dot_grafbase_path.join(ANALYTICS_DATA_FILE)
    }

    fn data_exists() -> Result<bool, CommonError> {
        match Self::get_data_location().try_exists() {
            Ok(result) => Ok(result),
            Err(error) => Err(CommonError::ReadAnalyticsDataFile(error)),
        }
    }

    fn read_data() -> Result<Option<AnalyticsData>, CommonError> {
        if !Self::data_exists()? {
            return Ok(None);
        }

        let data_path = Self::get_data_location();

        let data_string = std::fs::read_to_string(data_path).map_err(CommonError::ReadAnalyticsDataFile)?;

        let data: AnalyticsData =
            serde_json::from_str(&data_string).map_err(|_| CommonError::CorruptAnalyticsDataFile)?;

        Ok(Some(data))
    }

    pub fn opt_out() -> Result<(), CommonError> {
        Self::write_data(&AnalyticsData {
            opt_out: true,
            anonymous_id: None,
        })
    }

    pub fn opt_in() -> Result<(), CommonError> {
        Self::init_data()
    }

    pub fn reset_identifier() -> Result<(), CommonError> {
        Self::init_data()
    }

    /// # Panics
    ///
    /// - panics if the static ANALYTICS object was not previously initialized
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
        Self::get()
            .as_ref()
            .zip(Self::read_data().ok().flatten().and_then(|data| data.anonymous_id))
            .map(|(analytics, anonymous_id)| {
                // purposely ignoring errors
                // TODO possibly change this to a long lived thread once we add more events
                thread::spawn(move || {
                    let _: Result<_, _> = analytics.client.send(&Message::Track(Track {
                        event: event_name,
                        anonymous_id: Some(anonymous_id.to_string()),
                        properties,
                        context: Some(json!({
                            "startTime": analytics.start_time,
                            "sessionId": analytics.session_id
                        })),
                        ..Default::default()
                    }));
                })
            });
    }

    pub fn command_executed(name: &str, arguments: &[&'static str]) {
        Self::track(
            "Command Executed",
            Some(json!({ "name": name, "arguments": arguments })),
        );
    }
}

static ANALYTICS: OnceCell<Option<Analytics>> = OnceCell::new();
