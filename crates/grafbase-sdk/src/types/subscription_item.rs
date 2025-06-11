use crate::{types::Response, wit};

/// Represents one or multiple items from a subscription.
#[derive(Debug)]
pub enum SubscriptionItem {
    /// Single response item
    Single(Response),
    /// Multiple response items at once
    Multiple(Vec<Response>),
}

impl From<Response> for SubscriptionItem {
    fn from(value: Response) -> Self {
        SubscriptionItem::Single(value)
    }
}

impl From<Vec<Response>> for SubscriptionItem {
    fn from(value: Vec<Response>) -> Self {
        SubscriptionItem::Multiple(value)
    }
}

impl From<SubscriptionItem> for wit::SubscriptionItem {
    fn from(value: SubscriptionItem) -> Self {
        match value {
            SubscriptionItem::Single(response) => wit::SubscriptionItem::Single(response.into()),
            SubscriptionItem::Multiple(responses) => {
                wit::SubscriptionItem::Multiple(responses.into_iter().map(Into::into).collect())
            }
        }
    }
}
