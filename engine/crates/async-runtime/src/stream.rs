use futures_util::{
    future::{self, BoxFuture},
    stream::{self, BoxStream},
    Future, FutureExt, Stream, StreamExt as _,
};

pub trait StreamExt<'a> {
    type Item;

    /// Joins a future onto the execution of a stream returning a stream that also polls
    /// the given future.
    ///
    /// If the future ends the stream will still continue till completion but if the stream
    /// ends the future will be cancelled.
    ///
    /// This can be used when you have the receivng side of a channel and a future that sends
    /// on that channel - combining the two into a single stream that'll run till the channel
    /// is exhausted.  If you drop the stream you also cancel the underlying process.
    fn join<F>(self, future: F) -> impl Stream<Item = Self::Item> + 'a
    where
        F: Future<Output = ()> + Send + 'a;
}

impl<'a, T, Item> StreamExt<'a> for T
where
    T: Stream<Item = Item> + Send + 'a,
    Item: 'static,
{
    type Item = Item;

    fn join<F>(self, future: F) -> impl Stream<Item = Self::Item> + 'a
    where
        F: Future<Output = ()> + Send + 'a,
    {
        let stream: BoxStream<'a, Item> = Box::pin(self);
        let future: BoxFuture<'a, ()> = Box::pin(future);

        futures_util::stream::unfold(
            ProducerState::Running(stream.fuse(), future.fuse()),
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
}

enum ProducerState<'a, Item> {
    Running(stream::Fuse<BoxStream<'a, Item>>, future::Fuse<BoxFuture<'a, ()>>),
    Draining(stream::Fuse<BoxStream<'a, Item>>),
}
