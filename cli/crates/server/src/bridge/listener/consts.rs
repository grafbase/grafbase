use std::time::Duration;

pub const CLI_API_KEY: &str = "CLI_API_KEY";
pub const DEFAULT_AWS_REGION: &str = "us-east-1";
pub const MODIFICATIONS_TABLE_NAME: &str = "modifications";
pub const RECORDS_TABLE_NAME: &str = "records";
pub const MODIFICATION_POLL_INTERVAL: Duration = Duration::from_millis(100);
