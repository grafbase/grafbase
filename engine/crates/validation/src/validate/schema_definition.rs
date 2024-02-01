use super::*;
use crate::context::SchemaDefinition;

pub(crate) fn validate_schema_definition<'a>(def: &'a Positioned<ast::SchemaDefinition>, ctx: &mut Context<'a>) {
    if let Some(previous_def) = ctx.schema_definition.take() {
        let labels = vec![
            miette::LabeledSpan::new_with_span(
                Some("Previous definition".to_owned()),
                (
                    ctx.miette_pos(previous_def.pos),
                    miette::SourceOffset::from("schema".len()),
                ),
            ),
            miette::LabeledSpan::new_with_span(
                Some("Second definition".to_owned()),
                (ctx.miette_pos(def.pos), miette::SourceOffset::from("schema".len())),
            ),
        ];

        ctx.push_error(miette::miette! {
            labels = labels,
            "Duplicate schema definition",
        });
    }

    ctx.schema_definition = Some(SchemaDefinition {
        pos: def.pos,
        directives: &def.node.directives,
        query: def.node.query.as_ref().map(|node| node.node.as_str()),
        mutation: def.node.mutation.as_ref().map(|node| node.node.as_str()),
        subscription: def.node.subscription.as_ref().map(|node| node.node.as_str()),
    });
}

pub(crate) fn validate_schema_definition_references(ctx: &mut Context<'_>) {
    let Some(def) = ctx.schema_definition.as_ref().cloned() else {
        return;
    };
    let pos = def.pos;

    validate_directives(def.directives, ast::DirectiveLocation::Schema, ctx);

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

        if actual == default {
            continue;
        }

        match ctx.definition_names.get(actual) {
            None => {
                let labels = vec![miette::LabeledSpan::new_with_span(
                    None,
                    miette::SourceSpan::new(ctx.miette_pos(pos), "schema".len().into()),
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
    // validate that if there is no schema definition, check that Query, Mutation, Subscription are
    // object types
    if ctx.schema_definition.is_some() {
        return;
    }

    for name in ["Query", "Mutation", "Subscription"] {
        if let Some(def) = ctx.definition_names.get(name) {
            if !matches!(def.node.kind, ast::TypeKind::Object(_)) {
                ctx.push_error(miette::miette!("{name} should be an object"));
            }
        }
    }
}
