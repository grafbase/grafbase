use std::str::FromStr;

use crate::{InputValueError, LegacyInputType};

pub fn url<T: AsRef<str> + LegacyInputType>(value: &T) -> Result<(), InputValueError<T>> {
    if let Ok(true) =
        http::uri::Uri::from_str(value.as_ref()).map(|uri| uri.scheme().is_some() && uri.authority().is_some())
    {
        Ok(())
    } else {
        Err("invalid url".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert!(url(&"http".to_string()).is_err());
        assert!(url(&"https://google.com".to_string()).is_ok());
        assert!(url(&"http://localhost:80".to_string()).is_ok());
        assert!(url(&"ftp://localhost:80".to_string()).is_ok());
    }
}
