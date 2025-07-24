use futures::StreamExt;

pub enum NatsSubscriber {
    Stream(Box<async_nats::jetstream::consumer::pull::Stream>),
    Subject(async_nats::Subscriber),
}

impl NatsSubscriber {
    pub async fn next(&mut self) -> Result<Option<async_nats::Message>, String> {
        match self {
            NatsSubscriber::Stream(stream) => match stream.as_mut().next().await {
                Some(Ok(message)) => Ok(Some(message.into())),
                Some(Err(err)) => Err(err.to_string()),
                None => Ok(None),
            },
            NatsSubscriber::Subject(subject) => Ok(subject.next().await),
        }
    }
}
