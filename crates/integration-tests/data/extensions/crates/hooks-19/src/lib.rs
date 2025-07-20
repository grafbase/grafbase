use grafbase_sdk::{
    HooksExtension,
    host_io::{
        event_queue::EventQueue,
        http::{Method, StatusCode},
    },
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, HttpHeaders as _, OnRequestOutput},
};

#[derive(HooksExtension)]
struct Hooks;

impl HooksExtension for Hooks {
    fn new(_config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    #[allow(refining_impl_trait)]
    fn on_request(
        &mut self,
        _: &str,
        _: Method,
        headers: &mut GatewayHeaders,
    ) -> Result<OnRequestOutput, ErrorResponse> {
        let mut output = OnRequestOutput::new();
        if let Some(value) = headers.get("contract-key") {
            output.contract_key(value.to_str().unwrap().to_owned());
        }

        Ok(output)
    }

    fn on_response(&mut self, _: StatusCode, _headers: &mut GatewayHeaders, _queue: EventQueue) -> Result<(), Error> {
        Ok(())
    }
}
