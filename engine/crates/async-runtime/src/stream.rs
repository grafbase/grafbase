use futures_util::{
    future::{self, BoxFuture},
    stream::{self, BoxStream},
    Future, FutureExt, Stream, StreamExt,
};

/// Combines a stream and a "producer" future into a single stream
///
/// This can be used when you have the receivng side of a channel and a future that sends
/// on that channel - combining the two into a single stream that'll run till the channel
/// is exhausted.  If you drop the stream you also cancel the underlying process.
pub fn producer_stream<'a, S, P, Item>(stream: S, producer: P) -> impl Stream<Item = Item> + 'a
where
    S: Stream<Item = Item> + Send + 'a,
    P: Future<Output = ()> + Send + 'a,
    Item: 'static,
{
    let stream: BoxStream<'a, Item> = Box::pin(stream);
    let producer: BoxFuture<'a, ()> = Box::pin(producer);

    futures_util::stream::unfold(
        ProducerState::Running(stream.fuse(), producer.fuse()),
        |mut state| async {
            loop {
                match state {
                    ProducerState::Running(mut stream, mut producer) => {
                        futures_util::select! {
                            output = stream.next() => {
                                return Some((output?, ProducerState::Running(stream, producer)));
                            }
                            _ = producer => {
                                state = ProducerState::Draining(stream);
                                continue;
                            }
                        }
                    }
                    ProducerState::Draining(mut stream) => {
                        return Some((stream.next().await?, ProducerState::Draining(stream)))
                    }
                }
            }
        },
    )
}

enum ProducerState<'a, Item> {
    Running(stream::Fuse<BoxStream<'a, Item>>, future::Fuse<BoxFuture<'a, ()>>),
    Draining(stream::Fuse<BoxStream<'a, Item>>),
}
