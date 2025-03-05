mod dedup;
mod unique;

use std::sync::Arc;

use futures::stream::BoxStream;
use runtime::{error::PartialGraphqlError, extension::Data};

pub(super) type SubscriptionStream<'f> = BoxStream<'f, Result<Arc<Data>, PartialGraphqlError>>;

pub(super) use dedup::DeduplicatedSubscription;
pub(super) use unique::UniqueSubscription;
