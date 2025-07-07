use std::{fmt, ops::Deref};

/// A validated access token that is guaranteed to be a valid HTTP header value
#[derive(Clone)]
pub struct AccessToken(String);

const GRAFBASE_ACCESS_TOKEN: &str = "GRAFBASE_ACCESS_TOKEN";

impl AccessToken {
    /// Creates an AccessToken from the `GRAFBASE_ACCESS_TOKEN` environment variable.
    ///
    /// This function reads the `GRAFBASE_ACCESS_TOKEN` environment variable and validates
    /// it as a proper access token.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(AccessToken))` if the environment variable is set and contains a valid token
    /// - `Ok(None)` if the environment variable is not set
    /// - `Err(&'static str)` if the environment variable is set but contains an invalid token
    pub fn from_env() -> Result<Option<AccessToken>, &'static str> {
        ::std::env::var(GRAFBASE_ACCESS_TOKEN)
            .ok()
            .as_deref()
            .map(|token| AccessToken::new(token))
            .transpose()
    }

    /// Checks if the `GRAFBASE_ACCESS_TOKEN` environment variable is defined.
    ///
    /// This function returns `true` if the environment variable exists (regardless of its value),
    /// and `false` if it doesn't exist.
    ///
    /// # Returns
    ///
    /// - `true` if the `GRAFBASE_ACCESS_TOKEN` environment variable is set
    /// - `false` if the environment variable is not set
    pub fn is_defined_in_env() -> bool {
        ::std::env::var(GRAFBASE_ACCESS_TOKEN).is_ok()
    }

    /// Creates a new AccessToken, validating that it can be used in an HTTP header
    pub fn new(token: impl AsRef<str>) -> Result<Self, &'static str> {
        // First, trim any whitespace (including newlines) from the token
        let trimmed = token.as_ref().trim();

        // Validate that the trimmed token is not empty
        if trimmed.is_empty() {
            return Err("Access token cannot be empty");
        }

        // Validate that when combined with "Bearer ", it forms a valid header value
        let header_value = format!("Bearer {trimmed}");

        if http::HeaderValue::from_str(&header_value).is_err() {
            return Err("Invalid access token: contains characters that are not allowed in HTTP headers");
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccessToken").field(&"<REDACTED>").finish()
    }
}

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for AccessToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for AccessToken {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_access_token() {
        let token = AccessToken::new("valid-token-123").unwrap();
        assert_eq!(&*token, "valid-token-123");
    }

    #[test]
    fn test_access_token_with_special_chars() {
        // Test various valid characters
        let token = AccessToken::new("token_with-special.chars!123").unwrap();
        assert_eq!(&*token, "token_with-special.chars!123");
    }

    #[test]
    fn test_access_token_trims_whitespace() {
        let token = AccessToken::new("  token-with-spaces  \n").unwrap();
        assert_eq!(&*token, "token-with-spaces");
    }

    #[test]
    fn test_empty_token_fails() {
        let result = AccessToken::new("");
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err, "Access token cannot be empty");
        }
    }

    #[test]
    fn test_whitespace_only_token_fails() {
        let result = AccessToken::new("   \n\t  ");
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err, "Access token cannot be empty");
        }
    }

    #[test]
    fn test_token_with_invalid_header_chars() {
        // Test tokens with characters that are invalid in HTTP headers
        let result = AccessToken::new("token\nwith\nnewlines");
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.contains("Invalid access token"));
        }

        let result = AccessToken::new("token\rwith\rcarriage\rreturns");
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.contains("Invalid access token"));
        }

        // Null bytes are not allowed in HTTP headers
        let result = AccessToken::new("token\0with\0null");
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(
                err,
                "Invalid access token: contains characters that are not allowed in HTTP headers"
            );
        }
    }

    #[test]
    fn test_deref_implementation() {
        let token = AccessToken::new("test-token").unwrap();

        // Test that we can use string methods through deref
        assert_eq!(token.len(), 10);
        assert!(token.starts_with("test"));
        assert!(token.contains("-"));
    }

    #[test]
    fn test_as_ref_str() {
        let token = AccessToken::new("test-token").unwrap();
        let token_ref: &str = token.as_ref();
        assert_eq!(token_ref, "test-token");
    }

    #[test]
    fn test_as_ref_bytes() {
        let token = AccessToken::new("test-token").unwrap();
        let token_bytes: &[u8] = token.as_ref();
        assert_eq!(token_bytes, b"test-token");
    }

    #[test]
    fn test_clone() {
        let token1 = AccessToken::new("cloneable-token").unwrap();
        let token2 = token1.clone();
        assert_eq!(&*token1, &*token2);
    }

    #[test]
    fn test_long_token() {
        let long_token = "a".repeat(1000);
        let token = AccessToken::new(long_token.clone()).unwrap();
        assert_eq!(&*token, &long_token);
    }

    #[test]
    fn test_unicode_token() {
        // Some unicode characters are valid in HTTP headers
        let token = AccessToken::new("token-café-123").unwrap();
        assert_eq!(&*token, "token-café-123");
    }

    #[test]
    fn test_base64_token() {
        // Common format for access tokens
        let token = AccessToken::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9").unwrap();
        assert_eq!(&*token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_bearer_header_validation() {
        // The actual validation happens when forming "Bearer {token}"
        // So we should test tokens that might be problematic when prefixed with "Bearer "
        let token = AccessToken::new("valid-after-bearer").unwrap();

        // This should have passed validation with "Bearer " prefix
        let header_value = format!("Bearer {}", &*token);
        assert!(http::HeaderValue::from_str(&header_value).is_ok());
    }
}
