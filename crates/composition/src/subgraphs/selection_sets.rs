use super::Walker;
use crate::{strings::StringId, Subgraphs};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
pub(crate) struct SelectionSets(BTreeSet<Selection>);

impl SelectionSets {
    pub(crate) fn children(&self, parent: SelectionId) -> impl Iterator<Item = &Selection> {
        self.0.range(
            Selection {
                parent,
                id: SelectionId::MIN,
                field: StringId::MIN,
            }..Selection {
                parent,
                id: SelectionId::MAX,
                field: StringId::MAX,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SelectionId(usize);

impl SelectionId {
    pub const MIN: SelectionId = SelectionId(usize::MIN);
    pub const MAX: SelectionId = SelectionId(usize::MAX);
}

impl SelectionId {
    fn next() -> SelectionId {
        static ID_GENERATOR: AtomicUsize = AtomicUsize::new(0);
        SelectionId(ID_GENERATOR.fetch_add(1, Ordering::Relaxed))
    }
}

impl Subgraphs {
    pub(crate) fn selection_set_from_str(&mut self, fields: &str) -> Result<SelectionId, String> {
        // Cheating for now, we should port the parser from engines instead.
        let fields = format!("{{ {fields} }}");
        let parsed = async_graphql_parser::parse_query(fields).map_err(|err| err.to_string())?;

        let async_graphql_parser::types::ExecutableDocument {
            operations: async_graphql_parser::types::DocumentOperations::Single(operation),
            ..
        } = parsed
        else {
            return Err("The `fields` argument in `@keys` must be a selection set".to_owned());
        };

        let selection_set_ast = &operation.node.selection_set.node;
        let root_id = SelectionId::next();

        fn ingest_selection_set_rec(
            parent: SelectionId,
            item: &async_graphql_parser::types::Selection,
            subgraphs: &mut Subgraphs,
        ) -> Result<(), String> {
            match item {
                async_graphql_parser::types::Selection::Field(item) => {
                    let id = SelectionId::next();

                    subgraphs.selection_sets.0.insert(Selection {
                        parent,
                        id,
                        field: subgraphs.strings.intern(item.node.name.node.as_str()),
                    });

                    for item in &item.node.selection_set.node.items {
                        let id = SelectionId::next();
                        ingest_selection_set_rec(id, &item.node, subgraphs)?;
                    }

                    Ok(())
                }
                async_graphql_parser::types::Selection::FragmentSpread(_)
                | async_graphql_parser::types::Selection::InlineFragment(_) => {
                    Err("Fragments not allowed in key definitions.".to_owned())
                }
            }
        }

        for item in &selection_set_ast.items {
            ingest_selection_set_rec(root_id, &item.node, self)?
        }

        Ok(root_id)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Selection {
    parent: SelectionId,
    id: SelectionId,
    field: StringId,
}

pub(crate) type SelectionWalker<'a> = Walker<'a, &'a Selection>;

impl<'a> SelectionWalker<'a> {
    pub(crate) fn field(self) -> StringId {
        self.id.field
    }

    pub(crate) fn field_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.id.field)
    }

    pub(crate) fn children(self) -> impl Iterator<Item = SelectionWalker<'a>> {
        self.subgraphs
            .selection_sets
            .children(self.id.id)
            .map(move |selection| self.walk(selection))
    }
}
