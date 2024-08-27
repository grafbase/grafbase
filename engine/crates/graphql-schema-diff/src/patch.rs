use crate::{Change, ChangeKind};

/// Apply a diff to a source schema. The spans in the diff from the original target schema must have been resolved by [resolve_spans()] and not sorted.
pub fn patch<S>(source: &str, diff: &[Change], resolved_spans: &[S]) -> Result<PatchedSchema, cynic_parser::Error>
where
    S: AsRef<str>,
{
    let parsed = cynic_parser::parse_type_system_document(source)?;
    let mut schema = String::with_capacity(source.len() / 2);
    let paths = Paths::new(diff, resolved_spans);

    for definition in parsed.definitions() {
        // TODO: add whatever definitions were added

        match definition {
            cynic_parser::type_system::Definition::Schema(def)
            | cynic_parser::type_system::Definition::SchemaExtension(def) => {
                if paths
                    .iter_exact([""; 3])
                    .any(|change| matches!(change.kind(), ChangeKind::RemoveSchemaDefinition))
                {
                    continue;
                }

                let span = def.span();
                schema.push_str(&source[span.start..span.end]);
            }
            cynic_parser::type_system::Definition::Type(ty)
            | cynic_parser::type_system::Definition::TypeExtension(ty) => {
                if let Some(description) = ty.description() {
                    let span = description.span();
                    schema.push_str(&source[span.start..span.end]);
                    schema.push('\n');
                }

                let span = ty.span();
                schema.push_str(&source[span.start..span.end]);
            }
            cynic_parser::type_system::Definition::Directive(directive_definition) => {
                if paths
                    .iter_exact([directive_definition.name(), "", ""])
                    .any(|change| matches!(change.kind(), ChangeKind::RemoveDirectiveDefinition))
                {
                    continue;
                }

                let span = directive_definition.span();

                schema.push_str(&source[span.start..span.end])
            }
        }

        schema.push_str("\n\n");
    }

    for change in paths.iter_exact([""; 3]) {
        match change.kind() {
            ChangeKind::AddSchemaDefinition => {
                schema.push_str(change.resolved_str());
            }

            // Already handled
            ChangeKind::RemoveSchemaDefinition => (),

            change => debug_assert!(false, "Unhandled change at `.`: {change:?}"),
        }
    }

    Ok(PatchedSchema { schema })
}

struct Paths<'a, T>
where
    T: AsRef<str>,
{
    diff: &'a [Change],
    resolved_spans: &'a [T],

    paths: Vec<([&'a str; 3], usize)>,
}

impl<'a, T> Paths<'a, T>
where
    T: AsRef<str>,
{
    fn new(diff: &'a [Change], resolved_spans: &'a [T]) -> Self {
        let mut paths = diff
            .iter()
            .enumerate()
            .map(|(idx, diff)| (split_path(&diff.path), idx))
            .collect::<Vec<_>>();

        paths.sort();

        Paths {
            diff,
            paths,
            resolved_spans,
        }
    }

    fn iter_exact<'b: 'a>(&'b self, path: [&'b str; 3]) -> impl Iterator<Item = ChangeView<'a, T>> + 'b {
        let first = self.paths.partition_point(|(diff_path, _)| diff_path < &path);
        self.paths[first..]
            .iter()
            .take_while(move |(diff_path, _)| diff_path == &path)
            .enumerate()
            .map(move |(idx, _)| ChangeView {
                paths: self,
                idx: first + idx,
            })
    }
}

struct ChangeView<'a, T>
where
    T: AsRef<str>,
{
    paths: &'a Paths<'a, T>,
    idx: usize,
}

impl<'a, T> ChangeView<'a, T>
where
    T: AsRef<str>,
{
    pub(crate) fn kind(&self) -> ChangeKind {
        self.paths.diff[self.idx].kind
    }

    pub(crate) fn resolved_str(&self) -> &'a str {
        self.paths.resolved_spans[self.idx].as_ref()
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

fn split_path(path: &str) -> [&str; 3] {
    let mut segments = path.split('.');
    let path = std::array::from_fn(|_| segments.next().unwrap_or(""));
    debug_assert!(segments.next().is_none());
    path
}
