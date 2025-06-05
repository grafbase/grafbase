use runtime::hooks::Hooks;
use schema::{FieldDefinition, SchemaInputValue, TypeDefinition};

use crate::{
    prepare::PlanFieldArgumentsQueryView,
    response::{GraphqlError, ParentObjects},
};

// TODO: Remove unused @authorized
#[allow(unused)]
impl<H: Hooks> super::RequestHooks<'_, H> {
    pub async fn authorize_edge_pre_execution(
        &self,
        definition: FieldDefinition<'_>,
        arguments: PlanFieldArgumentsQueryView<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<(), GraphqlError> {
        unreachable!()
    }

    pub async fn authorize_parent_edge_post_execution(
        &self,
        definition: FieldDefinition<'_>,
        parents: &ParentObjects<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
        unreachable!()
    }

    pub async fn authorize_edge_node_post_execution(
        &self,
        definition: FieldDefinition<'_>,
        nodes: &ParentObjects<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
        unreachable!()
    }

    pub async fn authorize_node_pre_execution(
        &self,
        definition: TypeDefinition<'_>,
        metadata: Option<SchemaInputValue<'_>>,
    ) -> Result<(), GraphqlError> {
        unreachable!()
    }
}
