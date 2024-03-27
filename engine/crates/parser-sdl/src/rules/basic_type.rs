//! For basic types
//!
//! When a basic type is stubble uppon on the definition of the schema, if it
//! got no specialized behavior, we apply this behavior uppon it.
//!

use engine::registry::{
    self,
    federation::FederationKey,
    resolvers::{custom::CustomResolver, transformer::Transformer, Resolver},
    MetaField, MetaInputValue, MetaType, ObjectType,
};
use engine_parser::{
    types::{FieldDefinition, TypeKind},
    Pos, Positioned,
};
use itertools::Itertools;

use super::{
    deprecated_directive::DeprecatedDirective,
    federation::{
        ExternalDirective, InaccessibleDirective, KeyDirective, OverrideDirective, ProvidesDirective,
        ShareableDirective, TagDirective,
    },
    join_directive::JoinDirective,
    requires_directive::RequiresDirective,
    resolver_directive::ResolverDirective,
    visitor::{RuleError, Visitor, VisitorContext},
};
use crate::{
    directive_de::parse_directive, parser_extensions::FieldExtension, registry::add_input_type_non_primitive,
    rules::cache_directive::CacheDirective, schema_coord::SchemaCoord,
};

pub struct BasicType;

impl<'a> Visitor<'a> for BasicType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;

        if ["Query", "Mutation"].contains(&type_definition.node.name.node.as_str())
            || directives.iter().any(|directive| directive.is_model())
            || type_definition.node.extend
        {
            return;
        }

        let TypeKind::Object(object) = &type_definition.node.kind else {
            return;
        };

        let type_name = type_definition.node.name.node.to_string();

        let fields = object
            .fields
            .iter()
            .map(|field| {
                let name = field.name().to_string();
                let mapped_name = field.mapped_name().map(ToString::to_string);

                let mut resolver = field_resolver(field, mapped_name.as_deref());

                let mut requires =
                    RequiresDirective::from_directives(&field.directives, ctx).map(RequiresDirective::into_fields);

                let external = ExternalDirective::from_directives(&field.directives, ctx).is_some();
                let shareable = ShareableDirective::from_directives(&field.directives, ctx).is_some();
                let r#override =
                    OverrideDirective::from_directives(&field.directives, ctx).map(|directive| directive.from);
                let provides =
                    ProvidesDirective::from_directives(&field.directives, ctx).map(|directive| directive.fields);
                let deprecation = DeprecatedDirective::from_directives(&field.directives, ctx);
                let inaccessible = InaccessibleDirective::from_directives(&field.directives, ctx);
                let tags = TagDirective::from_directives(&field.directives, ctx);

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
                    external,
                    shareable,
                    provides,
                    r#override,
                    deprecation,
                    inaccessible,
                    tags,
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

        let external = ExternalDirective::from_directives(&type_definition.directives, ctx).is_some();
        let shareable = ShareableDirective::from_directives(&type_definition.directives, ctx).is_some();

        ctx.registry.get_mut().create_type(
            |_| {
                registry::ObjectType::new(type_name.clone(), fields)
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .with_cache_control(CacheDirective::parse(&type_definition.node.directives))
                    .with_external(external)
                    .with_shareable(shareable)
                    .into()
            },
            &type_name,
            &type_name,
        );

        // If the type is a non primitive and also not modelized, it means we need to
        // create the Input version of it.
        // If the input is non used by other queries/mutation, it'll be removed from the
        // final schema.
        add_input_type_non_primitive(ctx, object, &type_name);

        handle_key_directives(directives, &type_name, ctx)
    }
}

impl KeyDirective {
    fn into_key(self) -> FederationKey {
        match (self.resolvable, &self.select) {
            (true, None) => FederationKey::basic_type(self.fields.0),
            (false, _) => FederationKey::unresolvable(self.fields.0),
            (_, Some(select)) => FederationKey::join(self.fields.0, select.to_join_resolver()),
        }
    }
}

pub(super) fn handle_key_directives(
    directives: &[Positioned<engine_parser::types::ConstDirective>],
    type_name: &str,
    ctx: &mut VisitorContext<'_>,
) {
    // We also need to parse any @key directives
    let key_directives = directives
        .iter()
        .filter(|directive| directive.node.name.node == "key")
        .collect::<Vec<_>>();

    let (oks, errors) = key_directives
        .into_iter()
        .map(|directive| {
            Ok((
                directive.pos,
                parse_directive::<KeyDirective>(directive, ctx.variables)?,
            ))
        })
        .partition_result::<Vec<_>, Vec<_>, _, _>();

    ctx.append_errors(errors);

    ctx.append_errors(validate_keys(&oks, {
        let registry = ctx.registry.borrow();
        let Some(MetaType::Object(object)) = registry.types.get(type_name) else {
            // Apparently this can happen in the face of duplicate types.
            // Which is annoying but ok
            return;
        };
        object.clone()
    }));

    for (_, directive) in oks {
        ctx.registry
            .borrow_mut()
            .federation_entities
            .entry(type_name.to_owned())
            .or_default()
            .keys
            .push(directive.into_key());
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

fn validate_keys(key_directives: &[(Pos, KeyDirective)], object: ObjectType) -> Vec<RuleError> {
    let mut errors = Vec::new();

    // First make sure all the keys are actually fields
    for (pos, key) in key_directives {
        errors.extend(
            key.validate()
                .into_iter()
                .map(|error| RuleError::new(vec![*pos], error)),
        );

        for field in &key.fields.0 .0 {
            if object.field_by_name(&field.field).is_none() {
                errors.push(RuleError::new(
                    vec![*pos],
                    format!(
                        "The object {} has a key that requires the field {} but that field isn't present",
                        object.name, &field.field,
                    ),
                ));
            }
            // In an ideal world we'd also validate any nested keys, but we don't really have
            // access to any of the other types involved at this point :(
        }
    }

    // Ideally I'd like to do some validation of the fields as well:
    // Fields should usually be either part of a key or provided by a custom resolver.
    // But i'm going to leave that out for now as I'd rather not get it wrong
    // will try to revisit this.

    errors
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_validation_error;

    #[test]
    fn test_errors_if_missing_field_used_as_key() {
        assert_validation_error!(
            r#"
                extend schema @federation(version: "2.3")

                type User @key(fields: "id blah") {
                    id: ID!
                }
            "#,
            "The object User has a key that requires the field blah but that field isn't present"
        );
    }
}
