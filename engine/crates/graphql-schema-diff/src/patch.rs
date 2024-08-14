use std::collections::HashMap;

use crate::Change;

/// Apply a diff to a source schema. The spans in the diff from the original target schema must have been resolved by [resolve_spans()] and not sorted.
pub fn patch<S>(source: &str, diff: &[Change], resolved_spans: &[S]) -> Result<PatchedSchema, cynic_parser::Error>
where
    S: AsRef<str>,
{
    let parsed = cynic_parser::parse_type_system_document(source)?;
    let mut schema = String::with_capacity(source.len() / 2);
    let mut paths: HashMap<[&str; 3], usize> = diff
        .iter()
        .enumerate()
        .map(|(idx, diff)| (split_path(&diff.path), idx))
        .collect();

    for definition in parsed.definitions() {
        // TODO: add whatever definitions were added

        match definition {
            cynic_parser::type_system::Definition::Schema(_) => todo!(),
            cynic_parser::type_system::Definition::SchemaExtension(_) => todo!(),
            cynic_parser::type_system::Definition::Type(_) => todo!(),
            cynic_parser::type_system::Definition::TypeExtension(_) => todo!(),
            cynic_parser::type_system::Definition::Directive(_) => todo!(),
        }
    }

    Ok(PatchedSchema { schema })
}

/// A schema patched with [patch()].
pub struct PatchedSchema {
    schema: String,
}

impl PatchedSchema {
    /// Turn into just the patched schema.
    pub fn into_schema(self) -> String {
        self.schema
    }

    /// The patched schema.
    pub fn schema(&self) -> &str {
        &self.schema
    }
}

fn split_path(path: &str) -> [&str; 3] {
    let mut segments = path.split('.');
    let path = std::array::from_fn(|_| segments.next().unwrap_or(""));
    debug_assert!(segments.next().is_none());
    path
}
