use common_types::auth::Operations;
use engine::registry::{
    resolvers::{transformer::Transformer, Resolver},
    MetaField, ObjectType,
};

use crate::{
    registry::names::MetaNames,
    rules::{mongodb_directive::model_directive::create_type_context::CreateTypeContext, visitor::VisitorContext},
};

pub(crate) fn register_output(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let output_type_name = MetaNames::delete_payload_type(create_ctx.r#type);
    let mut output_field = MetaField::new("deletedCount", "Int");

    let transformer = Transformer::select("deletedCount");
    output_field.resolver = Resolver::from(transformer);

    output_field.required_operation = Some(Operations::DELETE);
    output_field.auth = create_ctx.model_auth().cloned();

    let object_type = ObjectType::new(&output_type_name, [output_field]);

    visitor_ctx
        .registry
        .get_mut()
        .create_type(|_| object_type.into(), &output_type_name, &output_type_name);

    output_type_name
}
