use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentDateTime(SystemTime);

impl CurrentDateTime {
    pub fn new() -> Self {
        Self(SystemTime::now())
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for CurrentDateTime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash is not implemented for web_time::SystemTime on wasm
        Into::<DateTime<Utc>>::into(self.clone()).hash(state);
    }
}

impl Display for CurrentDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Into::<DateTime<Utc>>::into(self.clone()).to_rfc3339_opts(SecondsFormat::Millis, true)
        )
    }
}

impl From<CurrentDateTime> for DateTime<Utc> {
    fn from(value: CurrentDateTime) -> Self {
        let duration = value
            .0
            .duration_since(UNIX_EPOCH)
            .expect("UNIX_EPOCH is always before current SystemTime");
        #[allow(clippy::cast_possible_wrap)]
        let (sec, nano_sec) = (duration.as_secs() as i64, duration.subsec_nanos());
        Utc.timestamp_opt(sec, nano_sec).unwrap()
    }
}

impl From<CurrentDateTime> for SystemTime {
    fn from(value: CurrentDateTime) -> Self {
        value.0
    }
}

// Suggested by Clippy
impl Default for CurrentDateTime {
    fn default() -> Self {
        Self::new()
    }
}
