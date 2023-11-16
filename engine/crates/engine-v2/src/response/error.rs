use super::Response;
use crate::request::Pos;

#[derive(Debug, serde::Serialize)]
pub struct GraphqlError {
    pub message: String,
    pub locations: Vec<Pos>,
    pub path: Vec<String>,
}

// Needs to be reworked later
impl Response {
    pub fn add_simple_error(&mut self, message: impl Into<String>) {
        self.add_error(Vec::new(), message, Vec::new());
    }
    pub fn add_error(&mut self, path: Vec<String>, message: impl Into<String>, locations: Vec<Pos>) {
        let error = GraphqlError {
            message: message.into(),
            locations,
            path,
        };
        self.errors.push(error);
    }
}
