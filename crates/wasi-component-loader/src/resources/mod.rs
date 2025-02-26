use futures::StreamExt;

pub use crate::access_log::AccessLogSender;
pub use crate::context::SharedContext;

pub type Headers = crate::WasmOwnedOrBorrowed<http::HeaderMap>;

#[derive(Clone)]
pub struct SharedResources {
    pub access_log: AccessLogSender,
}

pub type NatsClient = async_nats::Client;

pub enum NatsSubscriber {
    Stream(async_nats::jetstream::consumer::pull::Stream),
    Subject(async_nats::Subscriber),
}

impl NatsSubscriber {
    pub async fn next(&mut self) -> Result<Option<async_nats::Message>, String> {
        match self {
            NatsSubscriber::Stream(stream) => match stream.next().await {
                Some(Ok(message)) => Ok(Some(message.into())),
                Some(Err(err)) => Err(err.to_string()),
                None => Ok(None),
            },
            NatsSubscriber::Subject(subject) => Ok(subject.next().await),
        }
    }
}
