use std::{borrow::Cow, fmt};

use engine::registry::{InputObjectType, MetaInputValue};
use inflector::Inflector;
use parser_sdl::{
    INPUT_FIELD_COLLECTION_OP_APPEND, INPUT_FIELD_COLLECTION_OP_DELETE_AT_PATH, INPUT_FIELD_COLLECTION_OP_DELETE_ELEM,
    INPUT_FIELD_COLLECTION_OP_DELETE_KEY, INPUT_FIELD_COLLECTION_OP_PREPEND, INPUT_FIELD_NUM_OP_DECREMENT,
    INPUT_FIELD_NUM_OP_DIVIDE, INPUT_FIELD_NUM_OP_INCREMENT, INPUT_FIELD_NUM_OP_MULTIPLY, INPUT_FIELD_NUM_OP_SET,
    INPUT_FIELD_OP_CONTAINED, INPUT_FIELD_OP_CONTAINS, INPUT_FIELD_OP_EQ, INPUT_FIELD_OP_GT, INPUT_FIELD_OP_GTE,
    INPUT_FIELD_OP_IN, INPUT_FIELD_OP_LT, INPUT_FIELD_OP_LTE, INPUT_FIELD_OP_NE, INPUT_FIELD_OP_NIN,
    INPUT_FIELD_OP_NOT, INPUT_FIELD_OP_OVERLAPS,
};

use crate::registry::context::{InputContext, OutputContext};

static SCALARS: &[&str] = &[
    "Boolean",
    "BigInt",
    "UnsignedBigInt",
    "HexBytes",
    "Decimal",
    "Date",
    "DateTime",
    "Float",
    "ID",
    "Int",
    "JSON",
    // virtual type for non-JSONB operations (only set)
    "SimpleJSON",
    "PhoneNumber",
    "String",
    "URL",
    "Uuid",
    "IPAddress",
    "NaiveDateTime",
    "Time",
];

static NUMERIC_SCALARS: &[&str] = &["BigInt", "Float", "Decimal", "Int"];

static SCALAR_FILTERS: &[(&str, &str, &str)] = &[
    (INPUT_FIELD_OP_EQ, "=", "The value is exactly the one given"),
    (INPUT_FIELD_OP_NE, "<>", "The value is not the one given"),
    (INPUT_FIELD_OP_GT, ">", "The value is greater than the one given"),
    (INPUT_FIELD_OP_LT, "<", "The value is less than the one given"),
    (
        INPUT_FIELD_OP_GTE,
        ">=",
        "The value is greater than, or equal to the one given",
    ),
    (
        INPUT_FIELD_OP_LTE,
        "<=",
        "The value is less than, or equal to the one given",
    ),
];

#[derive(Clone, Copy)]
pub(super) enum TypeKind<'a> {
    Scalar(&'a str),
    Enum(&'a str),
}

impl<'a> TypeKind<'a> {
    fn prefixed(&'a self, input_ctx: &InputContext<'_>) -> Cow<'a, str> {
        match (self, input_ctx.namespace()) {
            (Self::Enum(r#type), Some(namespace)) => Cow::Owned(format!("{namespace}_{type}").to_pascal_case()),
            _ => Cow::Borrowed(self.as_ref()),
        }
    }
}

impl<'a> AsRef<str> for TypeKind<'a> {
    fn as_ref(&self) -> &str {
        match self {
            TypeKind::Enum(s) | TypeKind::Scalar(s) => s,
        }
    }
}

impl<'a> fmt::Display for TypeKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    for scalar in SCALARS {
        create_filter_types(input_ctx, TypeKind::Scalar(scalar), output_ctx);
        create_scalar_update_type(input_ctx, TypeKind::Scalar(scalar), output_ctx);
        create_array_update_type(input_ctx, TypeKind::Scalar(scalar), output_ctx);
    }
}

pub(super) fn create_array_update_type(
    input_ctx: &InputContext<'_>,
    scalar: TypeKind<'_>,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.update_input_name(&format!("{scalar}Array"));
    let mut fields = Vec::new();
    let scalar = scalar.prefixed(input_ctx);

    fields.push({
        let mut input = MetaInputValue::new("set", format!("[{scalar}]"));
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input
    });

    fields.push({
        let mut input = MetaInputValue::new("append", format!("[{scalar}]"));
        input.description = Some(String::from("Append an array value to the column."));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("prepend", format!("[{scalar}]"));
        input.description = Some(String::from("Prepend an array value to the column."));

        input
    });

    let description = format!("Update input for {scalar} array type.");
    let input_type = InputObjectType::new(type_name, fields).with_description(Some(description));

    output_ctx.create_input_type(input_type);
}

pub(super) fn create_scalar_update_type(
    input_ctx: &InputContext<'_>,
    scalar: TypeKind<'_>,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.update_input_name(scalar.as_ref());
    let mut fields = Vec::new();
    let scalar = scalar.prefixed(input_ctx);

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_NUM_OP_SET, scalar.as_ref());
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input
    });

    if NUMERIC_SCALARS.contains(&scalar.as_ref()) {
        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_NUM_OP_INCREMENT, scalar.as_ref());

            input.description = Some(String::from(
                "Increments the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_NUM_OP_DECREMENT, scalar.as_ref());

            input.description = Some(String::from(
                "Decrements the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_NUM_OP_MULTIPLY, scalar.as_ref());

            input.description = Some(String::from(
                "Multiplies the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_NUM_OP_DIVIDE, scalar.as_ref());

            input.description = Some(String::from("Divides the value of the field with the given value."));

            input
        });
    }

    if scalar == "JSON" {
        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_COLLECTION_OP_APPEND, scalar.as_ref());
            input.description = Some(String::from("Append JSON value to the column."));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_COLLECTION_OP_PREPEND, scalar.as_ref());
            input.description = Some(String::from("Prepend JSON value to the column."));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_COLLECTION_OP_DELETE_KEY, "String");

            input.description = Some(String::from(
                "Deletes a key (and its value) from a JSON object, or matching string value(s) from a JSON array.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_COLLECTION_OP_DELETE_ELEM, "Int");

            input.description = Some(String::from(
                "Deletes the array element with specified index (negative integers count from the end). Throws an error if JSON value is not an array.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new(INPUT_FIELD_COLLECTION_OP_DELETE_AT_PATH, "[String!]");

            input.description = Some(String::from(
                "Deletes the field or array element at the specified path, where path elements can be either field keys or array indexes.",
            ));

            input
        });
    }

    let description = format!("Update input for {scalar} type.");

    let input_type = InputObjectType::new(type_name.clone(), fields)
        .with_description(Some(description))
        .with_oneof(true);

    output_ctx.create_input_type(input_type);
}

pub(super) fn create_filter_types(input_ctx: &InputContext<'_>, scalar: TypeKind<'_>, output_ctx: &mut OutputContext) {
    let type_name = input_ctx.filter_type_name(scalar.as_ref());
    let mut fields = Vec::with_capacity(SCALAR_FILTERS.len() + 2);
    let scalar = scalar.prefixed(input_ctx);

    for (filter, mapped_name, description) in SCALAR_FILTERS {
        let mut input = MetaInputValue::new(*filter, scalar.as_ref());
        input.description = Some(String::from(*description));
        input.rename = Some((*mapped_name).to_string());

        fields.push(input);
    }

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_IN, format!("[{scalar}]"));
        input.description = Some(String::from("The value is in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_NIN, format!("[{scalar}]"));
        input.description = Some(String::from("The value is not in the given array of values"));

        input
    });

    fields.push(MetaInputValue::new(INPUT_FIELD_OP_NOT, type_name.as_str()));

    let description = format!("Search filter input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    output_ctx.create_input_type(input_type);

    let type_name = input_ctx.filter_type_name(&format!("{scalar}Array"));
    let mut fields = Vec::with_capacity(SCALAR_FILTERS.len() + 2);

    for (filter, mapped_name, description) in SCALAR_FILTERS {
        let mut input = MetaInputValue::new(*filter, format!("[{scalar}]"));
        input.description = Some(String::from(*description));
        input.rename = Some((*mapped_name).to_string());

        fields.push(input);
    }

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_IN, format!("[[{scalar}]]"));
        input.description = Some(String::from("The value is in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_NIN, format!("[[{scalar}]]"));
        input.description = Some(String::from("The value is not in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_CONTAINS, format!("[{scalar}]"));
        input.description = Some(String::from("The column contains all elements from the given array."));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_CONTAINED, format!("[{scalar}]"));
        input.description = Some(String::from("The given array contains all elements from the column."));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new(INPUT_FIELD_OP_OVERLAPS, format!("[{scalar}]"));
        input.description = Some(String::from("Do the arrays have any elements in common."));

        input
    });

    fields.push(MetaInputValue::new(INPUT_FIELD_OP_NOT, type_name.as_str()));

    let description = format!("Search filter input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    output_ctx.create_input_type(input_type);
}
