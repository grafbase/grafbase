//! For basic types
//!
//! When a basic type is stubble uppon on the definition of the schema, if it
//! got no specialized behavior, we apply this behavior uppon it.
//!
use grafbase_engine::registry::{
    self,
    resolvers::{custom::CustomResolver, transformer::Transformer, Resolver},
    MetaField,
};
use grafbase_engine_parser::{
    types::{FieldDefinition, TypeKind},
    Positioned,
};
use if_chain::if_chain;

use super::{
    resolver_directive::ResolverDirective,
    visitor::{Visitor, VisitorContext},
};
use crate::{registry::add_input_type_non_primitive, rules::cache_directive::CacheDirective};

pub struct BasicType;

impl<'a> Visitor<'a> for BasicType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a grafbase_engine::Positioned<grafbase_engine_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            if !["Query", "Mutation"].contains(&type_definition.node.name.node.as_str());
            if !directives.iter().any(|directive| directive.is_model());
            if let TypeKind::Object(object) = &type_definition.node.kind;
            then {
                let type_name = type_definition.node.name.node.to_string();
                // If it's a modeled Type, we create the associated type into the registry.
                // Without more data, we infer it's from our modelization.
                ctx.registry.get_mut().create_type(|_| registry::ObjectType::new(
                    type_name.clone(),
                    object.fields.iter().map(|field| {
                        let name = field.name().to_string();
                        let mapped_name = field.mapped_name().map(ToString::to_string);

                        let resolver = field_resolver(field, mapped_name.as_deref());

                        MetaField {
                            name: name.clone(),
                            mapped_name,
                            description: field.node.description.clone().map(|x| x.node),
                            ty: field.node.ty.clone().node.to_string().into(),
                            cache_control: CacheDirective::parse(&field.node.directives),
                            resolver,
                            ..Default::default()
                        }
                    })
                )
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .with_cache_control(CacheDirective::parse(&type_definition.node.directives))
                    .into()
                ,&type_name,
                &type_name
                );

                // If the type is a non primitive and also not modelized, it means we need to
                // create the Input version of it.
                // If the input is non used by other queries/mutation, it'll be removed from the
                // final schema.
                add_input_type_non_primitive(ctx, object, &type_name);
            }
        }
    }
}

fn field_resolver(field: &Positioned<FieldDefinition>, mapped_name: Option<&str>) -> Resolver {
    if let Some(resolver_name) = ResolverDirective::resolver_name(&field.node) {
        return Resolver::CustomResolver(CustomResolver {
            resolver_name: resolver_name.to_owned(),
        });
    }

    Transformer::select(mapped_name.unwrap_or_else(|| field.name())).into()
}
