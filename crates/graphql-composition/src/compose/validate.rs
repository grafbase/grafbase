use super::*;
use crate::{
    diagnostics::CompositeSchemasPreMergeValidationErrorCode,
    subgraphs::{FieldWalker, StringId},
};

pub(super) fn override_source_has_override(fields: &[FieldWalker<'_>], ctx: &mut Context<'_>) {
    use std::collections::BTreeMap;

    // Early exit if we don't have at least 2 overrides
    if fields
        .iter()
        .filter(|f| f.directives().r#override().is_some())
        .take(2)
        .count()
        < 2
    {
        return;
    }

    enum OverrideEntry<'a> {
        Single(&'a FieldWalker<'a>),
        Multiple(Vec<&'a FieldWalker<'a>>),
    }

    let mut overrides_by_source: BTreeMap<StringId, OverrideEntry<'_>> = BTreeMap::new();

    for field in fields {
        if let Some(override_directive) = field.directives().r#override() {
            use std::collections::btree_map::Entry;
            match overrides_by_source.entry(override_directive.from) {
                Entry::Vacant(e) => {
                    e.insert(OverrideEntry::Single(field));
                }
                Entry::Occupied(mut e) => {
                    let val = e.get_mut();
                    match val {
                        OverrideEntry::Single(first) => {
                            *val = OverrideEntry::Multiple(vec![first, field]);
                        }
                        OverrideEntry::Multiple(vec) => {
                            vec.push(field);
                        }
                    }
                }
            }
        }
    }

    for (source, entry) in overrides_by_source {
        if let OverrideEntry::Multiple(overriding_fields) = entry {
            let field_name = overriding_fields[0].name().as_str();
            let type_name = overriding_fields[0].parent_definition().name().as_str();
            let source_subgraph = ctx.subgraphs.walk(source).as_str();

            let source_has_override = fields
                .iter()
                .find(|f| f.parent_definition().subgraph().name().id == source)
                .and_then(|f| f.directives().r#override())
                .is_some();

            let message = if source_has_override {
                format!(
                    r#"Field "{}.{}" on subgraphs {} all override from "{}" which itself has an @override directive. Only one @override directive is allowed per field."#,
                    type_name,
                    field_name,
                    format_subgraph_list(&overriding_fields),
                    source_subgraph
                )
            } else {
                format!(
                    r#"Field "{}.{}" on subgraphs {} all override from "{}". Only one @override directive is allowed per field."#,
                    type_name,
                    field_name,
                    format_subgraph_list(&overriding_fields),
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

fn format_subgraph_list(fields: &[&FieldWalker<'_>]) -> String {
    match fields.len() {
        0 => String::new(),
        1 => fields[0].parent_definition().subgraph().name().as_str().to_string(),
        2 => format!(
            "{} and {}",
            fields[0].parent_definition().subgraph().name().as_str(),
            fields[1].parent_definition().subgraph().name().as_str()
        ),
        _ => {
            let mut result = String::new();
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    result.push_str(if i == fields.len() - 1 { " and " } else { ", " });
                }
                result.push_str(field.parent_definition().subgraph().name().as_str());
            }
            result
        }
    }
}
