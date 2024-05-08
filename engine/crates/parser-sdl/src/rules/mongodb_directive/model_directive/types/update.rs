use common_types::auth::Operations;
use engine::registry::{MetaField, ObjectType};
use registry_v2::resolvers::transformer::Transformer;

use crate::{
    registry::names::MetaNames,
    rules::{mongodb_directive::model_directive::create_type_context::CreateTypeContext, visitor::VisitorContext},
};

pub(crate) fn register_output(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let output_type_name = MetaNames::update_payload_type(create_ctx.r#type);

    let fields = ["matchedCount", "modifiedCount"].iter().map(|name| {
        let mut field = MetaField::new(*name, "Int");

        field.resolver = Transformer::select(name).into();
        field.required_operation = Some(Operations::DELETE);
        field.auth = create_ctx.model_auth().cloned();

        field
    });

    let object_type = ObjectType::new(&output_type_name, fields);

    visitor_ctx
        .registry
        .get_mut()
        .create_type(|_| object_type.into(), &output_type_name, &output_type_name);

    output_type_name
}
