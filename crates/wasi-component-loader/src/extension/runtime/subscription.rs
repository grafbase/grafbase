mod dedup;
mod unique;

use std::sync::Arc;

use engine::GraphqlError;
use futures::stream::BoxStream;
use runtime::extension::Data;

pub(super) type SubscriptionStream<'f> = BoxStream<'f, Result<Arc<Data>, GraphqlError>>;

pub(super) use dedup::DeduplicatedSubscription;
pub(super) use unique::UniqueSubscription;
