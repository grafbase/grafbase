use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt, TryStreamExt};
use rskafka::{client::consumer::StreamConsumer, record::RecordAndOffset};

type ConsumerStream = Pin<Box<dyn Stream<Item = Result<(RecordAndOffset, i64), String>> + Send + 'static>>;

pub struct KafkaConsumer {
    inner: ConsumerStream,
}

impl KafkaConsumer {
    pub fn new(consumers: Vec<StreamConsumer>) -> Self {
        let streams = consumers.into_iter().map(|c| c.map_err(|e| e.to_string()));
        let inner = Box::pin(futures::stream::select_all(streams));

        Self { inner }
    }
}

impl Stream for KafkaConsumer {
    type Item = Result<(RecordAndOffset, i64), String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}
