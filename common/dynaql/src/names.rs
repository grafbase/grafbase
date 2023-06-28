// Merged in the field namespace
pub const INPUT_FIELD_FILTER_ALL: &str = "ALL";
pub const INPUT_FIELD_FILTER_ANY: &str = "ANY";
pub const INPUT_FIELD_FILTER_NONE: &str = "NONE";
pub const INPUT_FIELD_FILTER_NOT: &str = "NOT";

pub const INPUT_FIELD_FILTER_EQ: &str = "eq";
pub const INPUT_FIELD_FILTER_NEQ: &str = "neq";
pub const INPUT_FIELD_FILTER_GT: &str = "gt";
pub const INPUT_FIELD_FILTER_GTE: &str = "gte";
pub const INPUT_FIELD_FILTER_LTE: &str = "lte";
pub const INPUT_FIELD_FILTER_LT: &str = "lt";
pub const INPUT_FIELD_FILTER_IN: &str = "in";
pub const INPUT_FIELD_FILTER_NOT_IN: &str = "notIn";
pub const INPUT_FIELD_FILTER_IS_NULL: &str = "isNull";

pub const INPUT_FIELD_FILTER_LIST_INCLUDES: &str = "includes";
pub const INPUT_FIELD_FILTER_LIST_INCLUDES_NONE: &str = "includesNone";
pub const INPUT_FIELD_FILTER_LIST_IS_EMPTY: &str = "isEmpty";

pub const OUTPUT_FIELD_ID: &str = "id";
pub const OUTPUT_FIELD_DELETED_ID: &str = "deletedId";

pub const DELETE_PAYLOAD_RETURN_TY_SUFFIX: &str = "DeletePayload";

/// Creates the return type name for deletions
pub fn deletion_return_type_name(type_name: &str) -> String {
    use case::CaseExt;

    format!(
        "{}{}",
        type_name.to_camel(),
        DELETE_PAYLOAD_RETURN_TY_SUFFIX
    )
}
