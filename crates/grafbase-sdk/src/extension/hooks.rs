use url::Url;

use crate::types::{Configuration, Error, ErrorResponse, HttpHeaders};

#[allow(unused_variables)]
pub trait HooksExtension: Sized + 'static {
    fn new(config: Configuration) -> Result<Self, Error>;

    fn on_request(&mut self, url: Url, method: http::Method, headers: HttpHeaders) -> Result<(), ErrorResponse>;

    fn on_response(&mut self, status: http::StatusCode, headers: HttpHeaders) -> Result<(), ErrorResponse>;
}
