use super::Response;
use crate::request::Pos;

#[derive(Debug, serde::Serialize)]
pub struct GraphqlError {
    pub message: String,
    pub locations: Vec<Pos>,
    pub path: Vec<String>,
}

impl Response {
    pub fn add_error(&mut self, path: Vec<String>, message: impl Into<String>, locations: Vec<Pos>) {
        let error = GraphqlError {
            message: message.into(),
            locations,
            path,
        };
        self.errors.push(error);
    }
}
