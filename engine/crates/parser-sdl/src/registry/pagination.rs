use common_types::auth::Operations;

use engine::registry::{self, resolvers::transformer::Transformer, MetaField, NamedType, Registry};

use crate::registry::names::{
    PAGE_INFO_FIELD_END_CURSOR, PAGE_INFO_FIELD_HAS_NEXT_PAGE, PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE,
    PAGE_INFO_FIELD_START_CURSOR, PAGE_INFO_TYPE,
};

pub(crate) fn register_page_info_type(registry: &mut Registry) -> NamedType<'static> {
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                PAGE_INFO_TYPE.to_string(),
                [
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                        ty: "Boolean!".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("has_previous_page")),
                        required_operation: Some(Operations::LIST),
                        // TODO: Auth should be propagated down during resolution from the parent
                        // type. PageInfo type is not specific to any data model, what matters is
                        // the model authorization of the model on which we iterate over.
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_NEXT_PAGE.to_string(),
                        ty: "Boolean!".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("has_next_page")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_START_CURSOR.to_string(),
                        ty: "String".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("start_cursor")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_END_CURSOR.to_string(),
                        ty: "String".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("end_cursor")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                ],
            )
            .into()
        },
        PAGE_INFO_TYPE,
        PAGE_INFO_TYPE,
    );
    PAGE_INFO_TYPE.into()
}
