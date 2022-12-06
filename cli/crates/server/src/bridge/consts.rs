use std::time::Duration;

pub const DB_FILE: &str = "data.sqlite";
pub const DB_URL_PREFIX: &str = "sqlite://";
pub const PREPARE: &str = include_str!("../sql/prepare.sql");
pub const MODIFICATION_POLL_INTERVAL: Duration = Duration::from_millis(100);
pub const MODIFICATIONS_TABLE_NAME: &str = "modifications";
