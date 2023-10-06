use crate::registry::context::{InputContext, OutputContext};
use common_types::auth::Operations;
use engine::registry::{resolvers::transformer::Transformer, MetaField, ObjectType};

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let object = ObjectType::new(
        input_ctx.type_name("PageInfo"),
        [
            MetaField {
                name: "hasPreviousPage".to_string(),
                ty: "Boolean!".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("has_previous_page")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: "hasNextPage".to_string(),
                ty: "Boolean!".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("has_next_page")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: "startCursor".to_string(),
                ty: "String".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("start_cursor")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: "endCursor".to_string(),
                ty: "String".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("end_cursor")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
        ],
    );

    output_ctx.create_object_type(object);
}
