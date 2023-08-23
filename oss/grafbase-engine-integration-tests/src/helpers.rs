use grafbase_engine::Response;
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

        let Some(mut data) = response.data else { panic!("Response had no data") };

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
