//! The public API for traversing [Subgraphs].

use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Walker<'a, Id> {
    pub(crate) id: Id,
    pub(crate) subgraphs: &'a Subgraphs,
}

impl<'a, Id> Walker<'a, Id> {
    pub(crate) fn walk<T>(self, other: T) -> Walker<'a, T> {
        self.subgraphs.walk(other)
    }
}

pub(crate) type DefinitionWalker<'a> = Walker<'a, DefinitionId>;
pub(crate) type FieldWalker<'a> = Walker<'a, FieldId>;
pub(crate) type SubgraphWalker<'a> = Walker<'a, SubgraphId>;

impl<'a> SubgraphWalker<'a> {
    fn subgraph(self) -> &'a Subgraph {
        &self.subgraphs.subgraphs[self.id.0]
    }

    pub(crate) fn name_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.subgraph().name)
    }
}

impl<'a> DefinitionWalker<'a> {
    fn definition(self) -> &'a Definition {
        &self.subgraphs.definitions[self.id.0]
    }

    pub fn name_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.name())
    }

    pub fn name(self) -> StringId {
        self.definition().name
    }

    pub fn kind(self) -> DefinitionKind {
        self.definition().kind
    }

    pub fn subgraph(self) -> SubgraphWalker<'a> {
        self.walk(self.definition().subgraph_id)
    }

    // pub fn object_fields(self) -> impl Iterator<Item = ObjectFieldWalker<'a>> {
    //     binary_search_range_for_key(&self.subgraphs.object_fields, self.id, |field| {
    //         field.object_id
    //     })
    //     .map(move |idx| self.walk(ObjectFieldId(idx)))
    // }
}

impl<'a> FieldWalker<'a> {
    fn field(self) -> &'a Field {
        &self.subgraphs.fields[self.id.0]
    }

    pub fn parent_definition(self) -> DefinitionWalker<'a> {
        self.walk(self.field().parent_id)
    }

    pub fn name(self) -> StringId {
        self.field().name
    }

    pub fn name_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.name())
    }

    pub fn type_name(self) -> StringId {
        self.field().type_name
    }
}

// fn binary_search_range_for_key<'a, T, K>(
//     store: &'a [T],
//     key: K,
//     extract_key: impl Fn(&T) -> K + 'a,
// ) -> impl Iterator<Item = usize> + 'a
// where
//     K: Ord + Eq + 'static,
// {
//     let start = store.partition_point(|item| extract_key(item) < key);
//     store[start..]
//         .iter()
//         .take_while(move |item| extract_key(*item) == key)
//         .enumerate()
//         .map(move |(idx, _)| idx + start)
// }
