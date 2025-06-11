mod dedup;
mod unique;

use futures::stream::BoxStream;
use runtime::extension::Response;

pub(super) type SubscriptionStream<'f> = BoxStream<'f, Response>;

pub(super) use dedup::DeduplicatedSubscription;
pub(super) use unique::UniqueSubscription;
