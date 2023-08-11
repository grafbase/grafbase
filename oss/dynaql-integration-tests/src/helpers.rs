use dynaql::Response;
use serde_json::Value;

use crate::ResponseData;

pub trait GetPath {
    fn get_string(&self, path: &str) -> String;
}

impl GetPath for Response {
    #[allow(clippy::panic)]
    fn get_string(&self, path: &str) -> String {
        // we want to panic early if the path is wrong, and not having a result
        // to deal with in the tests.
        let string = serde_json::to_string(&self.to_graphql_response()).expect("Serializing GraphQL response as JSON");

        let response: ResponseData<Value> =
            serde_json::from_str(&string).expect("Parsing GraphQL response as ResponseData");

        if let Some(errors) = response.errors {
            let error = errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join(",");

            panic!("{error}");
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
                    Some(_) => panic!("Referenced value is not a string."),
                    None => panic!("Invalid path."),
                },
                _ => {
                    panic!("Invalid path.");
                }
            }
        }

        result.expect("Invalid path.")
    }
}
