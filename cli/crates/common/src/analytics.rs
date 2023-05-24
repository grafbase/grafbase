#![allow(clippy::let_underscore_untyped)] // derivative

use chrono::{DateTime, Utc};
use core::panic;
use derivative::Derivative;
use once_cell::sync::OnceCell;
use rudderanalytics::{
    client::RudderAnalytics,
    message::{Message, Track},
};
use serde_json::{json, Value};
use std::thread;
use ulid::Ulid;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Analytics {
    #[derivative(Debug = "ignore")]
    client: RudderAnalytics,
    anonymous_id: Ulid,
    start_time: DateTime<Utc>,
}

impl Analytics {
    pub fn init() {
        ANALYTICS
            .set(
                option_env!("GRAFBASE_RUDDERSTACK_WRITE_KEY")
                    .zip(option_env!("GRAFBASE_RUDDERSTACK_DATAPLANE_URL"))
                    .map(|(write_key, dataplane_url)| Analytics {
                        client: RudderAnalytics::load(write_key.to_owned(), dataplane_url.to_owned()),
                        anonymous_id: Ulid::new(),
                        start_time: Utc::now(),
                    }),
            )
            .expect("cannot set analytics twice");
    }

    /// # Panics
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
        Self::get().as_ref().map(|analytics| {
            // purposely ignoring errors
            // TODO possibly change this to a long lived thread once we add more events
            thread::spawn(move || {
                let _ = analytics.client.send(&Message::Track(Track {
                    event: event_name,
                    anonymous_id: Some(analytics.anonymous_id.to_string()),
                    properties,
                    context: Some(json!({ "startTime": analytics.start_time })),
                    ..Default::default()
                }));
            })
        });
    }

    pub fn subcommand(name: &str, arguments: &[&'static str]) {
        Self::track("subcommand", Some(json!({ "name": name, "arguments": arguments })));
    }

    pub fn end(status: bool) {
        Self::track("end", Some(json!({ "status": status })));
    }
}

static ANALYTICS: OnceCell<Option<Analytics>> = OnceCell::new();
