use super::*;
use crate::context::SchemaDefinition;

pub(crate) fn validate_schema_definitions<'a>(schema_definitions: &[SchemaDefinition<'a>], ctx: &mut Context<'a>) {
    let mut first_definition_pos = None;

    for schema_definition in schema_definitions {
        validate_directives(schema_definition.directives, ast::DirectiveLocation::Schema, ctx);
        validate_schema_definition_references(schema_definition, ctx);

        if !schema_definition.is_extension {
            match &mut first_definition_pos {
                Some(pos) => {
                    let labels = vec![
                        miette::LabeledSpan::new_with_span(
                            Some("Previous definition".to_owned()),
                            miette::SourceSpan::new(ctx.miette_pos(*pos), "schema".len()),
                        ),
                        miette::LabeledSpan::new_with_span(
                            Some("Second definition".to_owned()),
                            miette::SourceSpan::new(ctx.miette_pos(schema_definition.pos), "schema".len()),
                        ),
                    ];
                    ctx.push_error(miette::miette!(labels = labels, "Duplicate schema definition"));
                }
                pos @ None => {
                    *pos = Some(schema_definition.pos);
                }
            }
        }
    }
}

pub(crate) fn validate_schema_definition_references<'a>(def: &SchemaDefinition<'a>, ctx: &mut Context<'a>) {
    let pos = def.pos;

    let names = [
        (def.query, "Query"),
        (def.mutation, "Mutation"),
        (def.subscription, "Subscription"),
    ];

    for idx in 0..(names.len()) {
        let name = &names[idx];
        let name = name.0.unwrap_or(name.1);
        for other_name in names[idx..].iter().skip(1) {
            let other_name = other_name.0.unwrap_or(other_name.1);
            if name == other_name {
                ctx.push_error(miette::miette!("Type used twice in schema definition: `{name}`"));
            }
        }
    }

    for (actual, default) in &names {
        let Some(actual) = actual else { continue };

        match ctx.definition_names.get(actual) {
            None => {
                let labels = vec![miette::LabeledSpan::new_with_span(
                    None,
                    miette::SourceSpan::new(ctx.miette_pos(pos), "schema".len()),
                )];
                ctx.push_error(miette::miette!(
                    labels = labels,
                    "Cannot set schema {} root to unknown type `{actual}`",
                    default.to_lowercase()
                ));
            }
            // http://spec.graphql.org/draft/#sec-Root-Operation-Types
            Some(referenced) => match referenced.node.kind {
                ast::TypeKind::Object(_) => (),
                _ => {
                    let type_name = referenced.node.name.node.as_str();
                    ctx.push_error(miette::miette!(
                        "{default} root type must be an Object type, it cannot be set to {type_name}"
                    ));
                }
            },
        }
    }
}

pub(crate) fn validate_root_types(ctx: &mut Context<'_>) {
    for name in ["Query", "Mutation", "Subscription"] {
        if let Some(def) = ctx.definition_names.get(name) {
            if !matches!(def.node.kind, ast::TypeKind::Object(_)) {
                ctx.push_error(miette::miette!("{name} should be an object"));
            }
        }
    }
}
