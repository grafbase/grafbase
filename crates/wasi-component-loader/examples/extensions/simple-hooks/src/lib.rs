use grafbase_sdk::{
    HooksExtension,
    host_io::{
        event_queue::EventQueue,
        http::{Method, StatusCode},
    },
    types::{Configuration, Error, ErrorResponse, Headers},
};

#[derive(HooksExtension)]
struct SimpleHooks;

impl HooksExtension for SimpleHooks {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    #[allow(refining_impl_trait)]
    fn on_request(&mut self, _: &str, _: Method, _: &mut Headers) -> Result<(), ErrorResponse> {
        Ok(())
    }

    fn on_response(&mut self, _: StatusCode, _: &mut Headers, _: EventQueue) -> Result<(), Error> {
        Ok(())
    }
}
