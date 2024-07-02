use engine::registry::{
    self,
    resolvers::{custom::CustomResolver, Resolver},
    MetaField,
};
use engine_parser::{
    types::{FieldDefinition, TypeKind},
    Positioned,
};
use registry_v2::resolvers::transformer::Transformer;

use super::{
    join_directive::JoinDirective,
    requires_directive::RequiresDirective,
    resolver_directive::ResolverDirective,
    visitor::{Visitor, VisitorContext},
};
use crate::{parser_extensions::FieldExtension, rules::cache_directive::CacheDirective, schema_coord::SchemaCoord};

pub struct Interface;

impl<'a> Visitor<'a> for Interface {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let TypeKind::Interface(interface) = &type_definition.node.kind else {
            return;
        };

        let type_name = type_definition.node.name.node.to_string();

        let fields = interface
            .fields
            .iter()
            .map(|field| {
                let name = field.name().to_string();
                let mapped_name = field.mapped_name().map(ToString::to_string);

                let mut resolver = field_resolver(field, mapped_name.as_deref());

                let mut requires =
                    RequiresDirective::from_directives(&field.directives, ctx).map(RequiresDirective::into_fields);

                if let Some(join_directive) = JoinDirective::from_directives(&field.node.directives, ctx) {
                    if resolver.is_custom() {
                        ctx.report_error(vec![field.pos], "A field can't have a join and a custom resolver on it");
                    }
                    if requires.is_some() {
                        // We could support this by merging the requires, but I don't want to implement it right now.
                        // If someone asks we could do it
                        ctx.report_error(vec![field.pos], "A field can't have a join and a requires on it");
                    }
                    requires = join_directive.select.required_fieldset(&field.arguments);

                    ctx.warnings.extend(
                        join_directive
                            .validate_arguments(&field.arguments, SchemaCoord::Field(type_name.as_str(), field.name())),
                    );

                    resolver = Resolver::Join(join_directive.select.to_join_resolver());
                }

                MetaField {
                    name: name.clone(),
                    mapped_name,
                    description: field.node.description.clone().map(|x| x.node),
                    ty: field.node.ty.clone().node.to_string().into(),
                    cache_control: CacheDirective::parse(&field.node.directives),
                    args: field.converted_arguments(),
                    resolver,
                    requires,
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

        ctx.registry.get_mut().create_type(
            |_| {
                registry::InterfaceType::new(type_name.clone(), fields)
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .with_cache_control(CacheDirective::parse(&type_definition.node.directives))
                    .into()
            },
            &type_name,
            &type_name,
        );

        ctx.registry
            .get_mut()
            .implements
            .entry(type_name)
            .or_default()
            .extend(interface.implements.iter().map(|name| name.to_string()));
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
