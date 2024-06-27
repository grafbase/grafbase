use std::sync::Arc;

use tracing::instrument;

use crate::{operation::FieldArgumentsView, response::GraphqlError};

#[derive(serde::Serialize)]
pub(crate) struct Input<'a> {
    pub arguments: FieldArgumentsView<'a>,
}

impl<'ctx> super::RequestHooks<'ctx> {
    #[instrument(skip_all)]
    pub async fn authorized(&self, rule: &str, input: Input<'_>) -> Option<GraphqlError> {
        let results = self
            .0
            .engine
            .env
            .hooks
            .authorized(
                Arc::clone(&self.0.request_metadata.context),
                rule.to_string(),
                vec![serde_json::to_string(&input).unwrap()],
            )
            .await;
        tracing::debug!("Authorized results: {results:#?}");
        match results {
            Ok(authorization_errors) => authorization_errors
                .into_iter()
                .next()
                .map(|maybe_error| maybe_error.map(Into::into))
                .unwrap_or_else(|| Some(GraphqlError::internal_server_error())),
            Err(err) => {
                if !err.is_user_error() {
                    tracing::error!("Hook error: {err:?}");
                }
                Some(err.into())
            }
        }
    }
}
