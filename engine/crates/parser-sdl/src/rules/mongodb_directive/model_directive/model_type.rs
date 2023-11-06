mod resolver_data;

use engine::{
    indexmap::IndexMap,
    names::{MONGODB_OUTPUT_FIELD_ID, OUTPUT_FIELD_ID},
    registry::{self, resolvers::transformer::Transformer, MetaField, MetaType},
};
use resolver_data::ResolverData;

use super::CreateTypeContext;
use crate::rules::{
    auth_directive::AuthDirective, requires_directive::RequiresDirective, resolver_directive::ResolverDirective,
    visitor::VisitorContext,
};

pub(super) fn create(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) {
    let type_name = create_ctx.model_name().to_string();
    let mut fields = IndexMap::new();

    fields.insert(OUTPUT_FIELD_ID.to_string(), {
        let mut id = MetaField::new(OUTPUT_FIELD_ID, "ID!");
        id.mapped_name = Some(String::from(MONGODB_OUTPUT_FIELD_ID));
        id.description = Some(String::from("Unique identifier"));

        let transformer = Transformer::Select {
            key: String::from(MONGODB_OUTPUT_FIELD_ID),
        };
        id.resolver = transformer.into();
        id.auth = create_ctx.model_auth().clone();

        id
    });

    for field in create_ctx.fields() {
        let name = field.name().to_string();
        let mapped_name = field.mapped_name().map(ToString::to_string);

        let auth = match AuthDirective::parse(visitor_ctx, &field.directives, false) {
            Ok(auth) => auth,
            Err(err) => {
                visitor_ctx.report_error(err.locations, err.message);
                None
            }
        }
        .or_else(|| create_ctx.model_auth().clone());

        let resolver_data = match ResolverDirective::resolver_name(field) {
            Some(resolver_name) => ResolverData::resolver(resolver_name, field),
            None => ResolverData::projection(field),
        };

        let requires =
            RequiresDirective::from_directives(&field.directives, visitor_ctx).map(RequiresDirective::into_fields);

        let description = field.description.as_ref().map(|description| description.node.clone());

        let meta_field = MetaField {
            name: name.clone(),
            mapped_name,
            description,
            args: resolver_data.args,
            ty: resolver_data.field_type.into(),
            cache_control: resolver_data.cache_control,
            resolver: resolver_data.resolver,
            requires,
            auth,
            ..Default::default()
        };

        fields.insert(name.clone(), meta_field);
    }

    let description = create_ctx.type_description().map(ToString::to_string);
    let cache_control = create_ctx.model_cache().clone();
    let rust_typename = create_ctx.model_name().to_string();
    let constraints = create_ctx.unique_constraints().collect();

    let object = MetaType::Object(registry::ObjectType {
        name: type_name.clone(),
        description,
        fields,
        cache_control,
        extends: false,
        visible: None,
        is_subscription: false,
        is_node: true,
        rust_typename,
        constraints,
        external: false,
    });

    visitor_ctx
        .registry
        .get_mut()
        .create_type(|_| object, &type_name, &type_name);
}
