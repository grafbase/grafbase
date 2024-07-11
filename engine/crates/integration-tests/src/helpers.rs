use std::sync::Arc;

use engine::{InitialResponse, Response, StreamingPayload};
use http::HeaderMap;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::ResponseData;

#[derive(Debug)]
pub struct Error(pub String);

pub trait GetPath {
    fn get_string_opt(&self, path: &str) -> Result<String, Error>;

    fn as_json_string(&self) -> String;

    fn get_string(&self, path: &str) -> String {
        self.get_string_opt(path).unwrap()
    }
}

impl GetPath for Response {
    fn as_json_string(&self) -> String {
        serde_json::to_string(&self.to_graphql_response()).expect("Serializing GraphQL response as JSON")
    }

    #[allow(clippy::panic)]
    fn get_string_opt(&self, path: &str) -> Result<String, Error> {
        // we want to panic early if the path is wrong, and not having a result
        // to deal with in the tests.
        let string = self.as_json_string();

        let response: ResponseData<Value> =
            serde_json::from_str(&string).expect("Parsing GraphQL response as ResponseData");

        if let Some(errors) = response.errors {
            let error = errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join(",");

            return Err(Error(error));
        }

        let Some(mut data) = response.data else {
            panic!("Response had no data")
        };

        let mut path = path.split('.').peekable();
        let mut result = None;

        while let Some(key) = path.next() {
            match data {
                Value::Object(mut object) if path.peek().is_some() => {
                    data = Value::from(object.remove(key));
                }
                Value::Object(ref mut object) => match object.remove(key) {
                    Some(Value::String(value)) => result = Some(value),
                    Some(_) => return Err(Error("Referenced value is not a string.".to_string())),
                    None => return Err(Error("Invalid path.".to_string())),
                },
                _ => {
                    panic!("Invalid path.");
                }
            }
        }

        match result {
            Some(result) => Ok(result),
            None => Err(Error("Invalid path".to_string())),
        }
    }
}

pub trait ResponseExt: Sized {
    /// Asserts that there are no errors in this Response
    #[allow(clippy::return_self_not_must_use)]
    fn assert_success(self) -> Self;

    /// Converts the response into a serde_json Value
    #[must_use]
    fn into_value(self) -> Value;

    /// Asserts that there are no errors and then decodes the data within the response
    #[must_use]
    fn into_data<T: DeserializeOwned>(self) -> T {
        let this = self.assert_success();
        serde_json::from_value(this.into_value()["data"].clone()).unwrap()
    }
}

impl ResponseExt for Response {
    fn assert_success(self) -> Self {
        assert_eq!(self.errors, vec![]);
        self
    }

    fn into_value(self) -> Value {
        serde_json::to_value(self.to_graphql_response()).expect("response to be serializable")
    }
}

impl ResponseExt for StreamingPayload {
    fn assert_success(self) -> Self {
        match self {
            StreamingPayload::InitialResponse(InitialResponse { data, has_next, errors }) => {
                assert_eq!(errors, vec![]);
                StreamingPayload::InitialResponse(InitialResponse { data, has_next, errors })
            }
            StreamingPayload::Incremental(incremental) => {
                assert_eq!(incremental.errors, vec![]);
                StreamingPayload::Incremental(incremental)
            }
        }
    }

    fn into_value(self) -> Value {
        serde_json::to_value(self).expect("streaming payload to be serializable")
    }
}

impl ResponseExt for (Arc<engine::Response>, HeaderMap) {
    fn assert_success(self) -> Self {
        assert_eq!(self.0.errors, vec![]);
        self
    }

    fn into_value(self) -> Value {
        serde_json::to_value(self.0.to_graphql_response()).expect("response to be serializable")
    }
}
