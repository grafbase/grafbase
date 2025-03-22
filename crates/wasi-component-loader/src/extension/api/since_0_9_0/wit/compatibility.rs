use engine_error::ErrorCode;

use crate::extension::api::since_0_8_0::wit::grafbase::sdk::types;

use super::{
    access_log::LogError,
    authorization::{AuthorizationDecisions, AuthorizationDecisionsDenySome},
    error::{Error, ErrorResponse},
    http_client::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpVersion},
    resolver::FieldOutput,
    token::Token,
};

impl From<HttpRequest> for types::HttpRequest {
    fn from(value: HttpRequest) -> Self {
        Self {
            method: value.method.into(),
            url: value.url,
            headers: value.headers,
            body: value.body,
            timeout_ms: value.timeout_ms,
        }
    }
}

impl From<types::HttpRequest> for HttpRequest {
    fn from(value: types::HttpRequest) -> Self {
        Self {
            method: value.method.into(),
            url: value.url,
            headers: value.headers,
            body: value.body,
            timeout_ms: value.timeout_ms,
        }
    }
}

impl From<HttpMethod> for types::HttpMethod {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => Self::Get,
            HttpMethod::Post => Self::Post,
            HttpMethod::Put => Self::Put,
            HttpMethod::Delete => Self::Delete,
            HttpMethod::Head => Self::Head,
            HttpMethod::Options => Self::Options,
            HttpMethod::Patch => Self::Patch,
            HttpMethod::Connect => Self::Connect,
            HttpMethod::Trace => Self::Trace,
        }
    }
}

impl From<types::HttpMethod> for HttpMethod {
    fn from(value: types::HttpMethod) -> Self {
        match value {
            types::HttpMethod::Get => Self::Get,
            types::HttpMethod::Post => Self::Post,
            types::HttpMethod::Put => Self::Put,
            types::HttpMethod::Delete => Self::Delete,
            types::HttpMethod::Head => Self::Head,
            types::HttpMethod::Options => Self::Options,
            types::HttpMethod::Patch => Self::Patch,
            types::HttpMethod::Connect => Self::Connect,
            types::HttpMethod::Trace => Self::Trace,
        }
    }
}

impl From<types::HttpResponse> for HttpResponse {
    fn from(value: types::HttpResponse) -> Self {
        Self {
            status: value.status,
            version: value.version.into(),
            headers: value.headers,
            body: value.body,
        }
    }
}

impl From<HttpResponse> for types::HttpResponse {
    fn from(value: HttpResponse) -> Self {
        Self {
            status: value.status,
            version: value.version.into(),
            headers: value.headers,
            body: value.body,
        }
    }
}

impl From<types::HttpVersion> for HttpVersion {
    fn from(value: types::HttpVersion) -> Self {
        match value {
            types::HttpVersion::Http09 => Self::Http09,
            types::HttpVersion::Http10 => Self::Http10,
            types::HttpVersion::Http11 => Self::Http11,
            types::HttpVersion::Http20 => Self::Http20,
            types::HttpVersion::Http30 => Self::Http30,
        }
    }
}

impl From<HttpVersion> for types::HttpVersion {
    fn from(value: HttpVersion) -> Self {
        match value {
            HttpVersion::Http09 => Self::Http09,
            HttpVersion::Http10 => Self::Http10,
            HttpVersion::Http11 => Self::Http11,
            HttpVersion::Http20 => Self::Http20,
            HttpVersion::Http30 => Self::Http30,
        }
    }
}

impl From<types::HttpError> for HttpError {
    fn from(value: types::HttpError) -> Self {
        match value {
            types::HttpError::Timeout => Self::Timeout,
            types::HttpError::Request(error) => Self::Request(error),
            types::HttpError::Connect(error) => Self::Connect(error),
        }
    }
}

impl From<HttpError> for types::HttpError {
    fn from(value: HttpError) -> Self {
        match value {
            HttpError::Timeout => Self::Timeout,
            HttpError::Request(error) => Self::Request(error),
            HttpError::Connect(error) => Self::Connect(error),
        }
    }
}

impl From<LogError> for types::LogError {
    fn from(err: LogError) -> Self {
        match err {
            LogError::ChannelFull(data) => Self::ChannelFull(data),
            LogError::ChannelClosed => Self::ChannelClosed,
        }
    }
}

impl From<types::FieldOutput> for FieldOutput {
    fn from(value: types::FieldOutput) -> Self {
        Self {
            outputs: value.outputs.into_iter().map(|v| v.map_err(Into::into)).collect(),
        }
    }
}

impl From<types::Token> for Token {
    fn from(value: types::Token) -> Self {
        Self::Bytes(value.raw)
    }
}

impl From<types::Token> for runtime::extension::Token {
    fn from(token: types::Token) -> Self {
        use runtime::extension::Token;

        Token::Bytes(token.raw)
    }
}

impl From<runtime::extension::Token> for Token {
    fn from(value: runtime::extension::Token) -> Self {
        match value {
            runtime::extension::Token::Anonymous => Self::Anonymous,
            runtime::extension::Token::Bytes(items) => Self::Bytes(items),
        }
    }
}

impl From<Token> for runtime::extension::Token {
    fn from(value: Token) -> Self {
        match value {
            Token::Anonymous => Self::Anonymous,
            Token::Bytes(items) => Self::Bytes(items),
        }
    }
}

impl From<types::AuthorizationDecisions> for AuthorizationDecisions {
    fn from(value: types::AuthorizationDecisions) -> Self {
        match value {
            types::AuthorizationDecisions::GrantAll => Self::GrantAll,
            types::AuthorizationDecisions::DenyAll(error) => Self::DenyAll(error.into()),
            types::AuthorizationDecisions::SparseDeny(sparse_deny_authorization_decisions) => {
                Self::DenySome(sparse_deny_authorization_decisions.into())
            }
        }
    }
}

impl From<types::SparseDenyAuthorizationDecisions> for AuthorizationDecisionsDenySome {
    fn from(value: types::SparseDenyAuthorizationDecisions) -> Self {
        Self {
            element_to_error: value.element_to_error,
            errors: value.errors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<types::AuthorizationDecisions> for runtime::extension::AuthorizationDecisions {
    fn from(decisions: types::AuthorizationDecisions) -> Self {
        match decisions {
            types::AuthorizationDecisions::GrantAll => runtime::extension::AuthorizationDecisions::GrantAll,
            types::AuthorizationDecisions::DenyAll(error) => {
                runtime::extension::AuthorizationDecisions::DenyAll(error.into_graphql_error(ErrorCode::Unauthorized))
            }
            types::AuthorizationDecisions::SparseDeny(types::SparseDenyAuthorizationDecisions {
                element_to_error,
                errors,
            }) => {
                let errors = errors
                    .into_iter()
                    .map(|err| err.into_graphql_error(ErrorCode::Unauthorized))
                    .collect();

                runtime::extension::AuthorizationDecisions::DenySome {
                    element_to_error,
                    errors,
                }
            }
        }
    }
}

impl From<types::ErrorResponse> for crate::ErrorResponse {
    fn from(value: types::ErrorResponse) -> Self {
        crate::ErrorResponse::Guest(ErrorResponse {
            status_code: value.status_code,
            errors: value.errors.into_iter().map(Into::into).collect(),
        })
    }
}

impl From<types::Error> for crate::Error {
    fn from(value: types::Error) -> Self {
        Self::Guest(value.into())
    }
}

impl From<types::Error> for Error {
    fn from(value: types::Error) -> Self {
        Self {
            extensions: value.extensions,
            message: value.message,
        }
    }
}

impl From<Error> for types::Error {
    fn from(value: Error) -> Self {
        Self {
            extensions: value.extensions,
            message: value.message,
        }
    }
}
