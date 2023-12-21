use engine::registry::{EnumType, InputObjectType, MetaEnumValue, MetaInputValue};

use crate::rules::{
    mongodb_directive::{DATE_TIME_SCALARS, MONGODB_SCALARS, NUMERIC_SCALARS},
    visitor::VisitorContext,
};

pub(crate) const MONGO_POP_POSITION: &str = "MongoDBPopPosition";

pub(crate) fn filter_type_name(scalar: &str) -> String {
    format!("MongoDB{scalar}SearchFilterInput")
}

pub(crate) fn required_update_type_name(scalar: &str) -> String {
    format!("MongoDBRequired{scalar}UpdateInput")
}

pub(crate) fn optional_update_type_name(scalar: &str) -> String {
    format!("MongoDBOptional{scalar}UpdateInput")
}

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>) {
    register_pop_input(visitor_ctx);

    for scalar in MONGODB_SCALARS {
        register_singular_type(visitor_ctx, scalar);
        register_array_type(visitor_ctx, scalar, true);

        if NUMERIC_SCALARS.contains(scalar) {
            register_numeric_update_type(visitor_ctx, scalar, true);
            register_numeric_update_type(visitor_ctx, scalar, false);
        } else {
            register_update_type(visitor_ctx, scalar, true);
            register_update_type(visitor_ctx, scalar, false);
        }
    }
}

pub(crate) fn register_array_type(visitor_ctx: &mut VisitorContext<'_>, field_type: &str, is_scalar: bool) {
    let type_name = filter_type_name(&format!("{field_type}Array"));
    dbg!(&type_name);

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
        let mut input = MetaInputValue::new("not", type_name.clone());
        input.description = Some(String::from("The value does not match the filters."));
        input.rename = Some(String::from("$not"));

        input
    });

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

pub(crate) fn register_update_type(visitor_ctx: &mut VisitorContext<'_>, scalar: &str, optional_field: bool) {
    let type_name = if optional_field {
        optional_update_type_name(scalar)
    } else {
        required_update_type_name(scalar)
    };

    let mut fields = Vec::new();

    fields.push({
        let mut input = MetaInputValue::new("set", scalar);
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input.rename = Some(String::from("$set"));

        input
    });

    if optional_field {
        fields.push({
            let mut input = MetaInputValue::new("unset", "Boolean");
            input.description = Some(String::from("Deletes a particular field."));
            input.rename = Some(String::from("$unset"));

            input
        });
    }

    if DATE_TIME_SCALARS.contains(&scalar) {
        fields.push({
            let mut input = MetaInputValue::new("currentDate", "Boolean");
            input.description = Some(format!("Sets the field value to the current {scalar}."));
            input.rename = Some(format!("$current{scalar}"));

            input
        });
    }

    let description = format!("Update input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);
}

pub(crate) fn register_numeric_update_type(visitor_ctx: &mut VisitorContext<'_>, scalar: &str, optional_field: bool) {
    let type_name = if optional_field {
        optional_update_type_name(scalar)
    } else {
        required_update_type_name(scalar)
    };

    let mut fields = Vec::new();

    fields.push({
        let mut input = MetaInputValue::new("increment", scalar);
        input.description = Some(String::from(
            "Increments the value of the field by the specified amount.",
        ));
        input.rename = Some(String::from("$inc"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("minimum", scalar);
        input.description = Some(String::from(
            "Only updates the field if the specified value is less than the existing field value.",
        ));
        input.rename = Some(String::from("$min"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("maximum", scalar);
        input.description = Some(String::from(
            "Only updates the field if the specified value is greater than the existing field value.",
        ));
        input.rename = Some(String::from("$max"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("multiply", scalar);
        input.description = Some(String::from(
            "Multiplies the value of the field by the specified amount.",
        ));
        input.rename = Some(String::from("$mul"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("set", scalar);
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input.rename = Some(String::from("$set"));

        input
    });

    if optional_field {
        fields.push({
            let mut input = MetaInputValue::new("unset", "Boolean");
            input.description = Some(String::from("Deletes a particular field."));
            input.rename = Some(String::from("$unset"));

            input
        });
    }

    let description = format!("Update input for {scalar} type.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);
}

fn register_pop_input(visitor_ctx: &mut VisitorContext<'_>) {
    let type_name = MONGO_POP_POSITION;
    let mut variants = Vec::new();

    variants.push({
        let mut variant = MetaEnumValue::new("FIRST".to_string());
        variant.value = Some("-1".to_string());
        variant.description = Some("Removes the first element.".to_string());

        variant
    });

    variants.push({
        let mut variant = MetaEnumValue::new("LAST".to_string());
        variant.value = Some("1".to_string());
        variant.description = Some("Removes the last element.".to_string());

        variant
    });

    let r#enum = EnumType::new(type_name.to_string(), variants);

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| r#enum.into(), type_name, type_name);
}
