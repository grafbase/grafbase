use crate::{SdkError, wit};

use super::{Data, Error};

/// Represents a response from the Grafbase SDK, which can contain data or errors.
#[derive(Debug, Default)]
pub struct Response {
    /// The data returned by the response, if any.
    pub data: Option<Data>,
    /// A list of errors that occurred during processing, if any.
    pub errors: Vec<Error>,
}

impl From<Data> for Response {
    fn from(data: Data) -> Self {
        Response {
            data: Some(data),
            errors: Vec::new(),
        }
    }
}

impl From<Error> for Response {
    fn from(error: Error) -> Self {
        Response {
            data: None,
            errors: vec![error],
        }
    }
}

impl From<SdkError> for Response {
    fn from(error: SdkError) -> Self {
        Response {
            data: None,
            errors: vec![error.into()],
        }
    }
}

impl From<Vec<Error>> for Response {
    fn from(errors: Vec<Error>) -> Self {
        Response { data: None, errors }
    }
}

impl<T, E> From<Result<T, E>> for Response
where
    Response: From<T> + From<E>,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => data.into(),
            Err(error) => error.into(),
        }
    }
}

impl Response {
    /// Creates a new empty response with no data and no errors.
    pub fn null() -> Self {
        Self::default()
    }

    /// Creates a new response with JSON data.
    pub fn json(bytes: Vec<u8>) -> Self {
        Response {
            data: Some(Data::Json(bytes)),
            errors: Vec::new(),
        }
    }

    /// Creates a new response with CBOR data.
    pub fn cbor(bytes: Vec<u8>) -> Self {
        Response {
            data: Some(Data::Cbor(bytes)),
            errors: Vec::new(),
        }
    }

    /// Creates a new response with an error.
    pub fn error<E: Into<Error>>(error: E) -> Self {
        Response {
            data: None,
            errors: vec![error.into()],
        }
    }

    /// Creates a new response with serialized data.
    pub fn data<T: serde::Serialize>(data: T) -> Self {
        match crate::cbor::to_vec(&data) {
            Ok(data) => Response {
                data: Some(Data::Cbor(data)),
                errors: Vec::new(),
            },
            Err(err) => Response {
                data: None,
                errors: vec![SdkError::from(err).into()],
            },
        }
    }
}

impl From<Response> for wit::Response {
    fn from(response: Response) -> Self {
        Self {
            data: response.data.map(Into::into),
            errors: response.errors.into_iter().map(Into::into).collect(),
        }
    }
}

impl<E: Into<wit::Error>> From<Result<Response, E>> for wit::Response {
    fn from(result: Result<Response, E>) -> Self {
        match result {
            Ok(response) => response.into(),
            Err(error) => Self {
                data: None,
                errors: vec![error.into()],
            },
        }
    }
}

impl From<Error> for wit::Response {
    fn from(error: Error) -> Self {
        Self {
            data: None,
            errors: vec![error.into()],
        }
    }
}
