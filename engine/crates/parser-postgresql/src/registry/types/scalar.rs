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
    "PhoneNumber",
    "String",
    "URL",
    "Uuid",
    "IPAddress",
    "NaiveDateTime",
    "Time",
];

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    static SCALAR_FILTERS: &[(&str, &str, &str)] = &[
        ("eq", "=", "The value is exactly the one given"),
        ("ne", "<>", "The value is not the one given"),
        ("gt", ">", "The value is greater than the one given"),
        ("lt", "<", "The value is less than the one given"),
        ("gte", ">=", "The value is greater than, or equal to the one given"),
        ("lte", "<=", "The value is less than, or equal to the one given"),
    ];

    for scalar in SCALARS {
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
    }

    // arrays
    for scalar in SCALARS {
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
}
