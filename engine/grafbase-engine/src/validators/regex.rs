use regex::Regex;

use crate::{InputValueError, LegacyInputType};

pub fn regex<T: AsRef<str> + LegacyInputType>(value: &T, regex: &'static str) -> Result<(), InputValueError<T>> {
    if Regex::new(regex).map(|re| re.is_match(value.as_ref())) == Ok(true) {
        Ok(())
    } else {
        Err(format_args!("value doesn't match expected format '{regex}'").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert!(regex(&"123".to_string(), "^[0-9]+$").is_ok());
        assert!(regex(&"12a3".to_string(), "^[0-9]+$").is_err());
    }
}
