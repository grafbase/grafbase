use grafbase_sdk::{
    HooksExtension,
    host_io::http::{Method, StatusCode},
    types::{Configuration, Error, ErrorResponse, GatewayHeaders},
};

#[derive(HooksExtension)]
struct SimpleHooks;

impl HooksExtension for SimpleHooks {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn on_request(&mut self, _: &str, _: Method, _: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
        Ok(())
    }

    fn on_response(&mut self, _: StatusCode, _: &mut GatewayHeaders) -> Result<(), String> {
        Ok(())
    }
}
