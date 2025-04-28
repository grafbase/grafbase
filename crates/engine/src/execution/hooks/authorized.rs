use futures::FutureExt;
use runtime::hooks::{AuthorizedHooks, EdgeDefinition, Hooks, NodeDefinition};
use schema::{FieldDefinition, SchemaInputValue, TypeDefinition};

use crate::{
    prepare::PlanFieldArgumentsQueryView,
    response::{GraphqlError, ResponseObjectsView},
};

impl<H: Hooks> super::RequestHooks<'_, H> {
    pub async fn authorize_edge_pre_execution(
        &self,
        definition: FieldDefinition<'_>,
        arguments: PlanFieldArgumentsQueryView<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<(), GraphqlError> {
        self.hooks
            .authorized()
            .authorize_edge_pre_execution(
                self.context,
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
    }

    pub async fn authorize_parent_edge_post_execution(
        &self,
        definition: FieldDefinition<'_>,
        parents: ResponseObjectsView<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
        self.hooks
            .authorized()
            .authorize_parent_edge_post_execution(
                self.context,
                EdgeDefinition {
                    parent_type_name: definition.parent_entity().name(),
                    field_name: definition.name(),
                },
                parents.iter_with_id().map(|(_, parent)| parent),
                metadata,
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
            .boxed()
            .await
    }

    pub async fn authorize_edge_node_post_execution(
        &self,
        definition: FieldDefinition<'_>,
        nodes: ResponseObjectsView<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
        self.hooks
            .authorized()
            .authorize_edge_node_post_execution(
                self.context,
                EdgeDefinition {
                    parent_type_name: definition.parent_entity().name(),
                    field_name: definition.name(),
                },
                nodes.iter_with_id().map(|(_, node)| node),
                metadata,
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
            .boxed()
            .await
    }

    pub async fn authorize_node_pre_execution(
        &self,
        definition: TypeDefinition<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<(), GraphqlError> {
        self.hooks
            .authorized()
            .authorize_node_pre_execution(
                self.context,
                NodeDefinition {
                    type_name: definition.name(),
                },
                metadata,
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due to the impl IntoIterator
            .boxed()
            .await
    }
}
