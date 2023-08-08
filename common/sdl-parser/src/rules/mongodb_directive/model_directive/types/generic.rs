use dynaql::registry::{InputObjectType, MetaInputValue};

use crate::rules::{mongodb_directive::MONGODB_SCALARS, visitor::VisitorContext};

pub(crate) fn filter_type_name(scalar: &str) -> String {
    format!("MongoDB{scalar}SearchFilterInput")
}

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>) {
    for scalar in MONGODB_SCALARS {
        register_singular_type(visitor_ctx, scalar);
        register_array_type(visitor_ctx, scalar, true);
    }
}

pub(crate) fn register_array_type(visitor_ctx: &mut VisitorContext<'_>, field_type: &str, is_scalar: bool) {
    let type_name = filter_type_name(&format!("{field_type}Array"));
    let mut fields = Vec::new();

    let input_type = if is_scalar {
        format!("[{field_type}]")
    } else {
        format!("[{field_type}Input]")
    };

    if is_scalar {
        let mut input = MetaInputValue::new("all", input_type);
        input.description = Some(String::from("The array must have all the fields"));
        input.rename = Some(String::from("$all"));
        fields.push(input);
    }

    let mut input = MetaInputValue::new("elemMatch", filter_type_name(field_type));
    input.description = Some(String::from("At least one of the elements matches the given criteria"));
    input.rename = Some(String::from("$elemMatch"));
    fields.push(input);

    let mut input = MetaInputValue::new("size", "Int");
    input.description = Some(String::from("The array must be of given length"));
    input.rename = Some(String::from("$size"));
    fields.push(input);

    let input_type = InputObjectType::new(type_name.clone(), fields)
        .with_description(Some(format!("Search filter input for [{field_type}] type.")));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);
}

pub(crate) fn register_singular_type(visitor_ctx: &mut VisitorContext<'_>, scalar: &str) {
    static SCALAR_FILTERS: &[(&str, &str, &str)] = &[
        ("eq", "$eq", "The value is exactly the one given"),
        ("ne", "$ne", "The value is not the one given"),
        ("gt", "$gt", "The value is greater than the one given"),
        ("lt", "$lt", "The value is less than the one given"),
        ("gte", "$gte", "The value is greater than, or equal to the one given"),
        ("lte", "$lte", "The value is less than, or equal to the one given"),
    ];

    let type_name = filter_type_name(scalar);
    let mut fields = Vec::new();

    for (filter, mapped_name, description) in SCALAR_FILTERS {
        let mut input = MetaInputValue::new(*filter, scalar);
        input.description = Some(String::from(*description));
        input.rename = Some((*mapped_name).to_string());

        fields.push(input);
    }

    fields.push({
        let mut input = MetaInputValue::new("in", format!("[{scalar}]"));
        input.description = Some(String::from("The value is in the given array of values"));
        input.rename = Some(String::from("$in"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("nin", format!("[{scalar}]"));
        input.description = Some(String::from("The value is not in the given array of values"));
        input.rename = Some(String::from("$nin"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("exists", "Boolean");
        input.description = Some(String::from("The value exists in the document and is not null."));
        input.rename = Some(String::from("$exists"));

        input
    });

    let description = format!("Search filter input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);
}
