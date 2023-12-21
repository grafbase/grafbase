use case::CaseExt;
use engine::registry::{InputObjectType, MetaInputValue};
use engine_parser::types::{BaseType, ObjectType, Type, TypeDefinition};

use super::generic::{self, filter_type_name, MONGO_POP_POSITION};
use crate::{
    registry::names::MetaNames,
    rules::{mongodb_directive::CreateTypeContext, visitor::VisitorContext},
};

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    register_type_input(visitor_ctx, create_ctx.object, create_ctx.r#type)
}

pub(crate) fn register_type_input(
    visitor_ctx: &mut VisitorContext<'_>,
    object: &ObjectType,
    r#type: &TypeDefinition,
) -> String {
    let input_type_name = MetaNames::update_input(r#type);

    let input_fields = object.fields.iter().map(|field| {
        let base = field.r#type().base.to_base_type_str();
        let is_list = field.r#type().base.is_list();
        let is_optional = field.r#type().nullable;

        let r#type = visitor_ctx.types.get(base);
        let composite_type = r#type.map(|r#type| r#type.is_composite()).unwrap_or_default();

        let r#type = match (r#type, composite_type) {
            (Some(r#type), false) if matches!(r#type.kind, engine_parser::types::TypeKind::Enum(_)) => base.to_owned(),
            (Some(r#type), false) if matches!(r#type.kind, engine_parser::types::TypeKind::Enum(_)) && is_list => {
                format!("[{base}]")
            }
            (Some(_), true) if is_list => {
                register_list_input(visitor_ctx, field.r#type(), field.name(), &input_type_name, true)
            }
            (Some(r#type), true) => MetaNames::update_input(&r#type.node),
            _ if is_list => register_list_input(visitor_ctx, field.r#type(), field.name(), &input_type_name, false),
            _ if is_optional => generic::optional_update_type_name(base),
            _ => generic::required_update_type_name(base),
        };

        let mut input = MetaInputValue::new(field.node.name.node.to_string(), r#type);
        input.description = field.description().map(ToString::to_string);
        input.rename = field.mapped_name().map(ToString::to_string);

        input
    });

    let input_type = InputObjectType::new(input_type_name.to_string(), input_fields);

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &input_type_name, &input_type_name);

    input_type_name
}

fn register_list_input(
    visitor_ctx: &mut VisitorContext<'_>,
    field_type: &Type,
    field_name: &str,
    container_name: &str,
    is_composite: bool,
) -> String {
    let composite_input = is_composite.then(|| {
        let name = format!("{}", field_type.base.to_base_type_str());
        Type::nullable(field_type.override_base(BaseType::named(&name)).base)
    });

    let optional_field_type = Type::nullable(field_type.base.clone());
    let type_name = format!("MongoDB{container_name}{}", field_name.to_camel());

    let add_to_set_type_name = register_add_to_set_input(
        visitor_ctx,
        &optional_field_type,
        field_name,
        container_name,
        is_composite,
    );

    let push_type_name = register_push_input(
        visitor_ctx,
        &optional_field_type,
        field_name,
        container_name,
        is_composite,
    );

    let mut fields = Vec::new();

    fields.push({
        let r#type = composite_input
            .as_ref()
            .map(|input| format!("{input}"))
            .unwrap_or_else(|| format!("{optional_field_type}"));

        let mut input = MetaInputValue::new("set", r#type);
        input.description = Some(String::from("Replaces the value of a field with the specified value."));
        input.rename = Some(String::from("$set"));

        input
    });

    if field_type.nullable {
        fields.push({
            let mut input = MetaInputValue::new("unset", "String");
            input.description = Some(String::from("Deletes a particular field."));
            input.rename = Some(String::from("$unset"));

            input
        });
    }

    fields.push({
        let mut input = MetaInputValue::new("addToSet", add_to_set_type_name);

        input.description = Some(String::from(
            "Adds a values to the array unless the value is already present, in which case does nothing.",
        ));

        input.rename = Some(String::from("$addToSet"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("pop", MONGO_POP_POSITION);
        input.description = Some(String::from("Removes the first or last element of an array."));
        input.rename = Some(String::from("$pop"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("pull", filter_type_name(field_type.base.to_base_type_str()));

        input.description = Some(String::from(
            "Removes from an existing array all instances of a value or values that match a specified condition.",
        ));

        input.rename = Some(String::from("$pull"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("push", push_type_name);
        input.description = Some(String::from("Appends specified values to the array."));
        input.rename = Some(String::from("$push"));

        input
    });

    fields.push({
        let r#type = composite_input
            .as_ref()
            .map(|input| format!("{input}"))
            .unwrap_or_else(|| format!("{optional_field_type}"));

        let mut input = MetaInputValue::new("pullAll", r#type);

        input.description = Some(String::from(
            "Removes all instances of the specified values from the array.",
        ));

        input.rename = Some(String::from("$pullAll"));

        input
    });

    let description = format!("Update input for {field_name}.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);

    type_name
}

fn register_push_input(
    visitor_ctx: &mut VisitorContext<'_>,
    field_type: &Type,
    field_name: &str,
    container_name: &str,
    is_composite: bool,
) -> String {
    let composite_input = is_composite.then(|| {
        Type::nullable(
            field_type
                .override_base(BaseType::named(&format!("{}", field_type.base.to_base_type_str())))
                .base,
        )
    });

    let type_name = format!("MongoDB{container_name}{}PushInput", field_name.to_camel());
    let mut fields = Vec::new();

    fields.push({
        let r#type = composite_input
            .map(|input| format!("{input}!"))
            .unwrap_or_else(|| format!("{field_type}!"));

        let mut input = MetaInputValue::new("each", r#type);
        input.description = Some(String::from("Add multiple elements to the array"));
        input.rename = Some(String::from("$each"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("sort", "MongoOrderByDirection");
        input.description = Some(String::from("Order the elements of the array"));
        input.rename = Some(String::from("$sort"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("slice", "Int");
        input.description = Some(String::from("Keep only the first N sorted elements of the array"));
        input.rename = Some(String::from("$slice"));

        input
    });

    fields.push({
        let mut input = MetaInputValue::new("position", "Int");

        input.description = Some(String::from("Indicates the position in the array, based on zero-based array index. Negative number corresponds to the position in the array, counting from the end."));

        input.rename = Some(String::from("$position"));

        input
    });

    let description = format!("Add values to {field_name} array.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);

    type_name
}

fn register_add_to_set_input(
    visitor_ctx: &mut VisitorContext<'_>,
    field_type: &Type,
    field_name: &str,
    container_name: &str,
    is_composite: bool,
) -> String {
    let composite_input = is_composite.then(|| {
        Type::nullable(
            field_type
                .override_base(BaseType::named(&format!("{}", field_type.base.to_base_type_str())))
                .base,
        )
    });

    let type_name = format!("MongoDB{container_name}{}AddToSetInput", field_name.to_camel());
    let mut fields = Vec::new();

    fields.push({
        let r#type = composite_input
            .map(|input| format!("{input}!"))
            .unwrap_or_else(|| format!("{field_type}!"));

        let mut input = MetaInputValue::new("each", r#type);
        input.description = Some(String::from("Add multiple elements to the array"));
        input.rename = Some(String::from("$each"));

        input
    });

    let description = format!("Add values to {field_name} array field if the array doesn't include them already.");
    let input_type = InputObjectType::new(type_name.clone(), fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &type_name, &type_name);

    type_name
}
