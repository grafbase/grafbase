use common_types::auth::Operations;
use parser_sdl::{
    PAGE_INFO_FIELD_END_CURSOR, PAGE_INFO_FIELD_HAS_NEXT_PAGE, PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE,
    PAGE_INFO_FIELD_START_CURSOR, PAGE_INFO_TYPE,
};
use registry_v1::{resolvers::transformer::Transformer, MetaField, ObjectType};

use crate::registry::context::{InputContext, OutputContext};

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let object = ObjectType::new(
        input_ctx.type_name(PAGE_INFO_TYPE),
        [
            MetaField {
                name: PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                ty: "Boolean!".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("has_previous_page")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: PAGE_INFO_FIELD_HAS_NEXT_PAGE.to_string(),
                ty: "Boolean!".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("has_next_page")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: PAGE_INFO_FIELD_START_CURSOR.to_string(),
                ty: "String".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("start_cursor")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
            MetaField {
                name: PAGE_INFO_FIELD_END_CURSOR.to_string(),
                ty: "String".into(),
                resolver: Transformer::PaginationData.and_then(Transformer::select("end_cursor")),
                required_operation: Some(Operations::LIST),
                ..Default::default()
            },
        ],
    );

    output_ctx.create_object_type(object);
}
