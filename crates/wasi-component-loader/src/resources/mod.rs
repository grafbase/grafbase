pub use crate::access_log::AccessLogSender;
pub use crate::context::SharedContext;

pub type Headers = crate::WasmOwnedOrBorrowed<http::HeaderMap>;

#[derive(Clone)]
pub struct SharedResources {
    pub access_log: AccessLogSender,
}

pub type NatsClient = async_nats::Client;
