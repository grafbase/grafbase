use super::*;
use crate::{diagnostics::CompositeSchemasPreMergeValidationErrorCode, subgraphs::StringId};

pub(super) fn override_source_has_override(fields: &[subgraphs::FieldView<'_>], ctx: &mut Context<'_>) {
    use std::collections::BTreeMap;

    // Early exit if we don't have at least 2 overrides
    if fields
        .iter()
        .filter(|f| f.directives.r#override(ctx.subgraphs).is_some())
        .take(2)
        .count()
        < 2
    {
        return;
    }

    enum OverrideEntry<'a> {
        Single(subgraphs::FieldView<'a>),
        Multiple(Vec<subgraphs::FieldView<'a>>),
    }

    let mut overrides_by_source: BTreeMap<StringId, OverrideEntry<'_>> = BTreeMap::new();

    for field in fields {
        if let Some(override_directive) = field.directives.r#override(ctx.subgraphs) {
            use std::collections::btree_map::Entry;
            match overrides_by_source.entry(override_directive.from) {
                Entry::Vacant(e) => {
                    e.insert(OverrideEntry::Single(*field));
                }
                Entry::Occupied(mut e) => {
                    let val = e.get_mut();
                    match val {
                        OverrideEntry::Single(first) => {
                            *val = OverrideEntry::Multiple(vec![*first, *field]);
                        }
                        OverrideEntry::Multiple(vec) => {
                            vec.push(*field);
                        }
                    }
                }
            }
        }
    }

    for (source, entry) in overrides_by_source {
        if let OverrideEntry::Multiple(overriding_fields) = entry {
            let field_name = ctx.subgraphs[overriding_fields[0].name].as_ref();
            let type_name = ctx.subgraphs[ctx.subgraphs.at(overriding_fields[0].parent_definition_id).name].as_ref();
            let source_subgraph = ctx.subgraphs[source].as_ref();

            let source_has_override = fields
                .iter()
                .find(|f| {
                    ctx.subgraphs
                        .at(ctx.subgraphs.at(f.parent_definition_id).subgraph_id)
                        .name
                        == source
                })
                .and_then(|f| f.directives.r#override(ctx.subgraphs))
                .is_some();

            let message = if source_has_override {
                format!(
                    r#"Field "{}.{}" on subgraphs {} all override from "{}" which itself has an @override directive. Only one @override directive is allowed per field."#,
                    type_name,
                    field_name,
                    format_subgraph_list(ctx, &overriding_fields),
                    source_subgraph
                )
            } else {
                format!(
                    r#"Field "{}.{}" on subgraphs {} all override from "{}". Only one @override directive is allowed per field."#,
                    type_name,
                    field_name,
                    format_subgraph_list(ctx, &overriding_fields),
                    source_subgraph
                )
            };

            ctx.diagnostics.push_composite_schemas_pre_merge_validation_error(
                message,
                CompositeSchemasPreMergeValidationErrorCode::OverrideSourceHasOverride,
            );
        }
    }
}

fn format_subgraph_list(ctx: &mut Context<'_>, fields: &[subgraphs::FieldView<'_>]) -> String {
    match fields {
        [] => String::new(),
        [field] => {
            let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
            let subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);
            ctx.subgraphs[subgraph.name].to_string()
        }
        [a, b] => {
            let [subgraph_a, subgraph_b] = [a, b].map(|field| {
                let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
                let subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);
                ctx.subgraphs[subgraph.name].as_ref()
            });

            format!("{subgraph_a} and {subgraph_b}",)
        }
        _ => {
            let mut result = String::new();
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    result.push_str(if i == fields.len() - 1 { " and " } else { ", " });
                }

                let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
                let subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);
                let subgraph_name = ctx.subgraphs[subgraph.name].as_ref();

                result.push_str(subgraph_name);
            }
            result
        }
    }
}
