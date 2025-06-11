use grafbase_sdk::{
    HooksExtension,
    host_io::event_queue::EventQueue,
    host_io::http::{HeaderValue, Method, StatusCode},
    types::{Configuration, Error, ErrorResponse, GatewayHeaders},
};

#[derive(HooksExtension)]
struct Hooks {
    config: TestConfig,
}

#[derive(serde::Deserialize)]
struct TestConfig {
    incoming_header: Option<HeaderTest>,
    outgoing_header: Option<HeaderTest>,
}

#[derive(serde::Deserialize)]
struct HeaderTest {
    key: String,
    value: String,
}

impl HooksExtension for Hooks {
    fn new(config: Configuration) -> Result<Self, Error> {
        let config = config.deserialize::<TestConfig>()?;

        Ok(Self { config })
    }

    fn on_request(&mut self, _: &str, _: Method, headers: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
        if let Some(ref header_test) = self.config.incoming_header {
            headers.append(
                header_test.key.as_str(),
                HeaderValue::from_str(&header_test.value).unwrap(),
            );
        }

        Ok(())
    }

    fn on_response(&mut self, _: StatusCode, headers: &mut GatewayHeaders, _: EventQueue) -> Result<(), String> {
        if let Some(ref header_test) = self.config.outgoing_header {
            headers.append(
                header_test.key.as_str(),
                HeaderValue::from_str(&header_test.value).unwrap(),
            );
        }

        Ok(())
    }
}
