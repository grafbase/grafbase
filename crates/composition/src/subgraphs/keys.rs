use super::*;
use async_graphql_parser::types as ast;

/// All the keys (`@key(...)`) in all the subgraphs in one container.
#[derive(Default)]
pub(crate) struct Keys(Vec<(DefinitionId, Key)>);

impl Subgraphs {
    pub(crate) fn iter_object_keys(
        &self,
        object_id: DefinitionId,
    ) -> impl Iterator<Item = KeyWalker<'_>> + '_ {
        let start = self
            .keys
            .0
            .partition_point(|(parent, _)| *parent < object_id);
        self.keys.0[start..]
            .iter()
            .take_while(move |(parent, _)| *parent == object_id)
            .map(move |(_, key)| self.walk(key))
    }

    pub(crate) fn push_key(
        &mut self,
        object_id: DefinitionId,
        selection_set_str: &str,
        resolvable: bool,
    ) -> Result<(), String> {
        let selection_set = self.selection_set_from_str(selection_set_str)?;
        let key = Key {
            selection_set,
            resolvable,
        };
        self.keys.0.push((object_id, key));
        Ok(())
    }

    fn selection_set_from_str(&mut self, fields: &str) -> Result<Vec<Selection>, String> {
        // Cheating for now, we should port the parser from engines instead.
        let fields = format!("{{ {fields} }}");
        let parsed = async_graphql_parser::parse_query(fields).map_err(|err| err.to_string())?;

        let ast::ExecutableDocument {
            operations: ast::DocumentOperations::Single(operation),
            ..
        } = parsed
        else {
            return Err("The `fields` argument in `@keys` must be a selection set".to_owned());
        };

        let selection_set_ast = &operation.node.selection_set.node;

        fn build_selection_set(
            items: &[async_graphql_parser::Positioned<ast::Selection>],
            subgraphs: &mut Subgraphs,
        ) -> Result<Vec<Selection>, String> {
            items
                .iter()
                .map(|item| match &item.node {
                    ast::Selection::Field(item) => {
                        let field = subgraphs.strings.intern(item.node.name.node.as_str());
                        let subselection =
                            build_selection_set(&item.node.selection_set.node.items, subgraphs)?;
                        Ok(Selection {
                            field,
                            subselection,
                        })
                    }
                    ast::Selection::FragmentSpread(_) | ast::Selection::InlineFragment(_) => {
                        Err("Fragments not allowed in key definitions.".to_owned())
                    }
                })
                .collect()
        }

        build_selection_set(&selection_set_ast.items, self)
    }
}

/// Corresponds to an `@key` annotation.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub(crate) struct Key {
    selection_set: Vec<Selection>,
    resolvable: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct Selection {
    pub(crate) field: StringId,
    pub(crate) subselection: Vec<Selection>,
}

pub(crate) type KeyWalker<'a> = Walker<'a, &'a Key>;

impl<'a> KeyWalker<'a> {
    pub(crate) fn fields(&self) -> impl Iterator<Item = &'a Selection> {
        self.id.selection_set.iter()
    }

    pub(crate) fn is_resolvable(&self) -> bool {
        self.id.resolvable
    }
}
