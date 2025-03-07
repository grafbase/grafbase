use serde::Serialize;

use crate::{cbor, wit, SdkError};

use super::Error;

/// List of items to be returned by a subscriptions.
/// If there are no items to return, use the default value.
pub struct SubscriptionOutput(wit::FieldOutput);

impl From<SubscriptionOutput> for wit::FieldOutput {
    fn from(output: SubscriptionOutput) -> Self {
        output.0
    }
}

impl SubscriptionOutput {
    /// Create a new builder
    pub fn builder() -> SubscriptionOutputBuilder {
        SubscriptionOutputBuilder { items: Vec::new() }
    }

    /// Create a new builder with a given capacity
    pub fn builder_with_capacity(capacity: usize) -> SubscriptionOutputBuilder {
        SubscriptionOutputBuilder {
            items: Vec::with_capacity(capacity),
        }
    }
}

/// Accumulator for setting the output individually for each `ResolverInput`.
pub struct SubscriptionOutputBuilder {
    items: Vec<Result<Vec<u8>, wit::Error>>,
}

impl SubscriptionOutputBuilder {
    /// Push the output for a given `ResolverInput`.
    pub fn push<T: Serialize>(&mut self, data: T) -> Result<(), SdkError> {
        let data = cbor::to_vec(data)?;
        self.items.push(Ok(data));
        Ok(())
    }

    /// Push an error for a given `ResolverInput`.
    pub fn push_error(&mut self, error: impl Into<Error>) {
        self.items.push(Err(Into::<Error>::into(error).into()));
    }

    /// Build the `SubscriptionOutput`.
    pub fn build(self) -> SubscriptionOutput {
        SubscriptionOutput(wit::FieldOutput { outputs: self.items })
    }
}
