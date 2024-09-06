use cynic_parser::type_system::SchemaDefinition;

use crate::ChangeKind;

use super::{directives::patch_directives, paths::Paths, INDENTATION};

pub(super) fn patch_schema_definition<T: AsRef<str>>(
    definition: SchemaDefinition<'_>,
    schema: &mut String,
    paths: &Paths<'_, T>,
) {
    let mut new_query_type = None;
    let mut new_mutation_type = None;
    let mut new_subscription_type = None;

    for change in paths.iter_exact([""; 3]) {
        match change.kind() {
            ChangeKind::ChangeQueryType => {
                new_query_type = Some(change.resolved_str());
            }
            ChangeKind::ChangeMutationType => {
                new_mutation_type = Some(change.resolved_str());
            }
            ChangeKind::ChangeSubscriptionType => {
                new_subscription_type = Some(change.resolved_str());
            }
            ChangeKind::RemoveSchemaDefinition => return,
            _ => (),
        }
    }

    schema.push_str("schema");

    patch_directives(definition.directives(), schema, paths);

    schema.push_str(" {\n");

    for (operation_name, maybe_replacement, in_source) in [
        ("query", new_query_type, definition.query_type()),
        ("mutation", new_mutation_type, definition.mutation_type()),
        ("subscription", new_subscription_type, definition.subscription_type()),
    ] {
        if let Some(type_name) = maybe_replacement
            .or_else(|| in_source.map(|ty| ty.named_type()))
            .filter(|name| !name.is_empty())
        {
            schema.push_str(INDENTATION);
            schema.push_str(operation_name);
            schema.push_str(": ");
            schema.push_str(type_name);
            schema.push('\n')
        }
    }

    schema.push_str("}\n\n");
}
