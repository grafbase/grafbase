use engine::registry::{InputObjectType, MetaInputValue};

use crate::registry::context::{InputContext, OutputContext};

static SCALARS: &[&str] = &[
    "Boolean",
    "BigInt",
    "UnsignedBigInt",
    "Bytes",
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
    ("eq", "=", "The value is exactly the one given"),
    ("ne", "<>", "The value is not the one given"),
    ("gt", ">", "The value is greater than the one given"),
    ("lt", "<", "The value is less than the one given"),
    ("gte", ">=", "The value is greater than, or equal to the one given"),
    ("lte", "<=", "The value is less than, or equal to the one given"),
];

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    for scalar in SCALARS {
        create_filter_types(input_ctx, scalar, output_ctx);
        create_scalar_update_type(input_ctx, scalar, output_ctx);
        create_array_update_type(input_ctx, scalar, output_ctx);
    }
}

fn create_array_update_type(input_ctx: &InputContext<'_>, scalar: &str, output_ctx: &mut OutputContext) {
    let type_name = input_ctx.update_type_name(&format!("{scalar}Array"));
    let mut fields = Vec::new();

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

fn create_scalar_update_type(input_ctx: &InputContext<'_>, scalar: &str, output_ctx: &mut OutputContext) {
    let type_name = input_ctx.update_type_name(scalar);
    let mut fields = Vec::new();

    fields.push({
        let mut input = MetaInputValue::new("set", scalar);
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input
    });

    if NUMERIC_SCALARS.contains(&scalar) {
        fields.push({
            let mut input = MetaInputValue::new("increment", scalar);

            input.description = Some(String::from(
                "Increments the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("decrement", scalar);

            input.description = Some(String::from(
                "Decrements the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("multiply", scalar);

            input.description = Some(String::from(
                "Multiplies the value of the field by the specified amount.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("divide", scalar);

            input.description = Some(String::from("Divides the value of the field with the given value."));

            input
        });
    }

    if scalar == "JSON" {
        fields.push({
            let mut input = MetaInputValue::new("append", scalar);
            input.description = Some(String::from("Append json value to the column."));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("prepend", scalar);
            input.description = Some(String::from("Prepend json value to the column."));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("deleteKey", "String");

            input.description = Some(String::from(
                "Deletes a key (and its value) from a JSON object, or matching string value(s) from a JSON array.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("deleteElem", "Int");

            input.description = Some(String::from(
                "Deletes the array element with specified index (negative integers count from the end). Throws an error if JSON value is not an array.",
            ));

            input
        });

        fields.push({
            let mut input = MetaInputValue::new("deleteAtPath", "[String!]");

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

fn create_filter_types(input_ctx: &InputContext<'_>, scalar: &&str, output_ctx: &mut OutputContext) {
    let type_name = input_ctx.filter_type_name(scalar);
    let mut fields = Vec::with_capacity(SCALAR_FILTERS.len() + 2);

    for (filter, mapped_name, description) in SCALAR_FILTERS {
        let mut input = MetaInputValue::new(*filter, *scalar);
        input.description = Some(String::from(*description));
        input.rename = Some((*mapped_name).to_string());

        fields.push(input);
    }

    fields.push({
        let mut input = MetaInputValue::new("in", format!("[{scalar}]"));
        input.description = Some(String::from("The value is in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("nin", format!("[{scalar}]"));
        input.description = Some(String::from("The value is not in the given array of values"));

        input
    });

    fields.push(MetaInputValue::new("not", type_name.as_str()));

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
        let mut input = MetaInputValue::new("in", format!("[[{scalar}]]"));
        input.description = Some(String::from("The value is in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("nin", format!("[[{scalar}]]"));
        input.description = Some(String::from("The value is not in the given array of values"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("contains", format!("[{scalar}]"));
        input.description = Some(String::from("The column contains all elements from the given array."));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("contained", format!("[{scalar}]"));
        input.description = Some(String::from("The given array contains all elements from the column."));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("overlaps", format!("[{scalar}]"));
        input.description = Some(String::from("Do the arrays have any elements in common."));

        input
    });

    fields.push(MetaInputValue::new("not", type_name.as_str()));

    let description = format!("Search filter input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    output_ctx.create_input_type(input_type);
}
