mod directives;
mod paths;
mod schema_definitions;
mod type_definitions;

use self::paths::Paths;
use crate::{Change, ChangeKind};

const INDENTATION: &str = "  ";

/// Apply a diff to a source schema. The spans in the diff from the original target schema must have been resolved by [resolve_spans()] and not sorted.
pub fn patch<S>(source: &str, diff: &[Change], resolved_spans: &[S]) -> Result<PatchedSchema, cynic_parser::Error>
where
    S: AsRef<str>,
{
    let parsed = Some(source)
        .filter(|source| !source.trim().is_empty()) // FIXME: doesn't take comments into account
        .map(cynic_parser::parse_type_system_document)
        .transpose()?
        .unwrap_or_default();

    let mut schema = String::with_capacity(source.len() / 2);
    let paths = Paths::new(diff, resolved_spans, source);

    for change in paths.iter_top_level() {
        match change.kind() {
            ChangeKind::AddSchemaDefinition
            | ChangeKind::AddObjectType
            | ChangeKind::AddUnion
            | ChangeKind::AddEnum
            | ChangeKind::AddScalar
            | ChangeKind::AddInterface
            | ChangeKind::AddDirectiveDefinition
            | ChangeKind::AddInputObject => {
                schema.push_str(change.resolved_str());
                schema.push_str("\n\n");
            }
            ChangeKind::AddSchemaExtension => {
                schema.push_str("extend ");
                schema.push_str(change.resolved_str());
                schema.push_str("\n\n");
            }
            _ => (),
        }
    }

    for definition in parsed.definitions() {
        match definition {
            cynic_parser::type_system::Definition::Schema(def) => {
                schema_definitions::patch_schema_definition(
                    def,
                    DefinitionOrExtension::Definition,
                    &mut schema,
                    &paths,
                );
            }
            cynic_parser::type_system::Definition::SchemaExtension(def) => {
                schema_definitions::patch_schema_definition(def, DefinitionOrExtension::Extension, &mut schema, &paths);
            }
            cynic_parser::type_system::Definition::Type(ty) => {
                type_definitions::patch_type_definition(ty, DefinitionOrExtension::Definition, &mut schema, &paths);
            }
            cynic_parser::type_system::Definition::TypeExtension(ty) => {
                type_definitions::patch_type_definition(ty, DefinitionOrExtension::Extension, &mut schema, &paths);
            }
            cynic_parser::type_system::Definition::Directive(directive_definition) => {
                directives::patch_directive_definition(directive_definition, &mut schema, &paths);
            }
        }
    }

    Ok(PatchedSchema { schema })
}

enum DefinitionOrExtension {
    Extension,
    Definition,
}

impl DefinitionOrExtension {
    /// Returns `true` if the definition or extension is [`Extension`].
    ///
    /// [`Extension`]: DefinitionOrExtension::Extension
    #[must_use]
    fn is_extension(&self) -> bool {
        matches!(self, Self::Extension)
    }

    /// Returns `true` if the definition or extension is [`Definition`].
    ///
    /// [`Definition`]: DefinitionOrExtension::Definition
    #[must_use]
    fn is_definition(&self) -> bool {
        matches!(self, Self::Definition)
    }
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
