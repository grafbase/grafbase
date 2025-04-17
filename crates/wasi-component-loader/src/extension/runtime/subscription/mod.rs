mod dedup;
mod unique;

use engine_error::GraphqlError;
use futures::stream::BoxStream;
use runtime::extension::Data;

pub(super) type SubscriptionStream<'f> = BoxStream<'f, Result<Data, GraphqlError>>;

pub(super) use dedup::DeduplicatedSubscription;
pub(super) use unique::UniqueSubscription;
