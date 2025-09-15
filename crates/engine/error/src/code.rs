use strum::EnumCount;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::AsRefStr,
    strum::IntoStaticStr,
    strum::FromRepr,
    strum_macros::EnumCount,
    strum::EnumIter,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ErrorCode {
    BadRequest,
    InternalServerError,
    TrustedDocumentError,
    // Used for APQ
    PersistedQueryError,
    PersistedQueryNotFound,
    // Subgraph errors
    SubgraphError,
    SubgraphInvalidResponseError,
    SubgraphRequestError,
    // Auth
    Unauthenticated,
    Unauthorized,
    // Operation preparation phases
    OperationParsingError,
    OperationValidationError,
    OperationPlanningError,
    VariableError,
    // Runtime
    ExtensionError,
    // Rate limit
    RateLimited,
    // Timeouts
    GatewayTimeout,
}

impl From<ErrorCode> for http::StatusCode {
    fn from(code: ErrorCode) -> http::StatusCode {
        code.into_http_status_code_with_priority().0
    }
}

impl From<operation::ErrorKind> for ErrorCode {
    fn from(kind: operation::ErrorKind) -> Self {
        match kind {
            operation::ErrorKind::Parsing => ErrorCode::OperationParsingError,
            operation::ErrorKind::Validation => ErrorCode::OperationValidationError,
        }
    }
}

impl ErrorCode {
    pub fn into_http_status_code_with_priority(self) -> (http::StatusCode, usize) {
        match self {
            ErrorCode::OperationParsingError
            | ErrorCode::OperationValidationError
            | ErrorCode::OperationPlanningError
            | ErrorCode::VariableError
            | ErrorCode::PersistedQueryNotFound
            | ErrorCode::PersistedQueryError
            | ErrorCode::TrustedDocumentError
            | ErrorCode::BadRequest => (http::StatusCode::BAD_REQUEST, 1000),
            ErrorCode::Unauthenticated => (http::StatusCode::UNAUTHORIZED, 600),
            ErrorCode::Unauthorized => (http::StatusCode::FORBIDDEN, 600),
            ErrorCode::RateLimited => (http::StatusCode::TOO_MANY_REQUESTS, 500),
            ErrorCode::SubgraphError | ErrorCode::SubgraphInvalidResponseError | ErrorCode::SubgraphRequestError => {
                (http::StatusCode::BAD_GATEWAY, 300)
            }
            ErrorCode::GatewayTimeout => (http::StatusCode::GATEWAY_TIMEOUT, 200),
            // least helpful error codes
            ErrorCode::ExtensionError | ErrorCode::InternalServerError => (http::StatusCode::INTERNAL_SERVER_ERROR, 0),
        }
    }
}

#[derive(Debug, Default)]
pub struct ErrorCodeCounter([u16; ErrorCode::COUNT]);

impl ErrorCodeCounter {
    pub fn from_errors(errors: &[super::GraphqlError]) -> Self {
        let mut counter = Self::default();
        for error in errors {
            counter.increment(error.code);
        }
        counter
    }

    pub fn increment(&mut self, code: ErrorCode) {
        self.0[code as usize] += 1;
    }

    pub fn increment_by(&mut self, code: ErrorCode, count: u16) {
        self.0[code as usize] += count;
    }

    pub fn add(&mut self, other: &Self) {
        for (index, count) in other.0.iter().enumerate() {
            self.0[index] += count;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (ErrorCode, u16)> + '_ {
        self.0.iter().copied().enumerate().filter_map(|(index, count)| {
            if count > 0 {
                Some((ErrorCode::from_repr(index).unwrap(), count))
            } else {
                None
            }
        })
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        for i in self.0 {
            count += i as usize;
        }
        count
    }

    pub fn to_vec(&self) -> Vec<(ErrorCode, u16)> {
        self.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::ErrorCode;

    #[test]
    fn santity_check_discriminant() {
        for value in ErrorCode::iter() {
            assert_eq!(value, ErrorCode::from_repr(value as usize).unwrap());
        }
    }
}
