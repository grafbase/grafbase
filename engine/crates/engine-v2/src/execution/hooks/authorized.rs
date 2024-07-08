use futures::FutureExt;
use runtime::hooks::{EdgeDefinition, Hooks, NodeDefinition};
use schema::{EntityWalker, FieldDefinitionWalker, SchemaInputValueWalker};
use tracing::{instrument, Level};

use crate::{operation::FieldArgumentsView, response::GraphqlError, Runtime};

impl<'ctx, R: Runtime> super::RequestHooks<'ctx, R> {
    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn authorize_edge_pre_execution(
        &self,
        definition: FieldDefinitionWalker<'_>,
        arguments: FieldArgumentsView<'_>,
        metadata: Option<SchemaInputValueWalker<'_>>,
    ) -> Result<(), GraphqlError> {
        self.0
            .engine
            .runtime
            .hooks()
            .authorize_edge_pre_execution(
                &self.0.request_context.hooks_context,
                EdgeDefinition {
                    parent_type_name: definition.parent_entity().name(),
                    field_name: definition.name(),
                },
                arguments,
                metadata,
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
            .boxed()
            .await
            .map_err(Into::into)
    }

    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn authorize_node_pre_execution(
        &self,
        entity: EntityWalker<'_>,
        metadata: Option<SchemaInputValueWalker<'_>>,
    ) -> Result<(), GraphqlError> {
        self.0
            .engine
            .runtime
            .hooks()
            .authorize_node_pre_execution(
                &self.0.request_context.hooks_context,
                NodeDefinition {
                    type_name: entity.name(),
                },
                metadata,
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
            .boxed()
            .await
            .map_err(Into::into)
    }
}
