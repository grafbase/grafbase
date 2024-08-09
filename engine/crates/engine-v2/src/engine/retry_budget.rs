use schema::{sources::graphql::GraphqlEndpointId, Schema};
use tower::retry::budget::Budget;

use super::Runtime;

pub(super) struct RetryBudgets {
    by_graphql_endpoints: Vec<Option<Budget>>,
}

id_newtypes::index! {
    RetryBudgets.by_graphql_endpoints[GraphqlEndpointId] => Option<Budget>,
}

impl RetryBudgets {
    pub fn build(schema: &Schema) -> Self {
        Self {
            by_graphql_endpoints: schema
                .walker()
                .graphql_endpoints()
                .map(|endpoint| {
                    let retry_config = endpoint.retry_config()?;

                    // Defaults: https://docs.rs/tower/0.4.13/src/tower/retry/budget.rs.html#137-139
                    let ttl = retry_config.ttl.unwrap_or(std::time::Duration::from_secs(10));
                    let min_per_second = retry_config.min_per_second.unwrap_or(10);
                    let retry_percent = retry_config.retry_percent.unwrap_or(0.2);

                    Some(Budget::new(ttl, min_per_second, retry_percent))
                })
                .collect(),
        }
    }
}

impl<R: Runtime> super::Engine<R> {
    pub(crate) fn get_retry_budget_for_non_mutation(&self, endpoint_id: GraphqlEndpointId) -> Option<&Budget> {
        self.retry_budgets[endpoint_id].as_ref()
    }

    pub(crate) fn get_retry_budget_for_mutation(&self, endpoint_id: GraphqlEndpointId) -> Option<&Budget> {
        if self
            .schema
            .walk(endpoint_id)
            .retry_config()
            .map(|config| config.retry_mutations)
            .unwrap_or_default()
        {
            self.retry_budgets[endpoint_id].as_ref()
        } else {
            None
        }
    }
}
