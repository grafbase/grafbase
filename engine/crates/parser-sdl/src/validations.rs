use std::sync::OnceLock;

use regex::Regex;

static NAME_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn validate_connector_name(name: &str) -> Result<(), String> {
    let name_regex = NAME_REGEX.get_or_init(|| Regex::new("^[A-Za-z_][A-Za-z0-9_]*$").unwrap());

    if name.is_empty() {
        return Err("Connector names cannot be empty".into());
    }

    if !name_regex.is_match(name) {
        return Err("Connector names must be alphanumeric and cannot start with a number".into());
    }

    Ok(())
}
