use futures::FutureExt;
use runtime::hooks::{EdgeDefinition, Hooks};
use schema::FieldDefinitionWalker;

use crate::{operation::FieldArgumentsView, response::GraphqlError, Runtime};

impl<'ctx, R: Runtime> super::RequestHooks<'ctx, R> {
    pub async fn authorize_edge_pre_execution(
        &self,
        definition: FieldDefinitionWalker<'_>,
        arguments: FieldArgumentsView<'_>,
    ) -> Result<(), GraphqlError> {
        let future = self.0.engine.runtime.hooks().authorize_edge_pre_execution(
            &self.0.request_context.hooks_context,
            EdgeDefinition {
                parent_type_name: definition.parent_entity().name(),
                field_name: definition.name(),
            },
            arguments,
            serde_json::Value::Null,
        );
        // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
        //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
        //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
        let result = future.boxed().await;
        tracing::debug!("Authorized results: {result:#?}");
        result.map_err(Into::into)
    }
}
