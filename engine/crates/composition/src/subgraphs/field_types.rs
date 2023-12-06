use async_graphql_parser::types as ast;
use indexmap::IndexSet;

use super::*;

/// All the field types in the schema. Interned. Comparing two field's type has the same cost as
/// comparing two `usize`s.
#[derive(Default)]
pub(super) struct FieldTypes {
    inner_types: IndexSet<InnerFieldType>,
    wrappers: IndexSet<WrapperType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FieldTypeId(usize);

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct WrapperTypeId(usize);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum WrapperTypeKind {
    List,
    NonNullList,
}

#[derive(Hash, PartialEq, Eq)]
struct WrapperType {
    kind: WrapperTypeKind,
    outer: Option<WrapperTypeId>,
}

#[derive(Hash, PartialEq, Eq)]
struct InnerFieldType {
    name: StringId,
    wrapper: Option<WrapperTypeId>,
    is_required: bool,
}

impl Subgraphs {
    pub(crate) fn intern_field_type(&mut self, field_type: &ast::Type) -> FieldTypeId {
        let mut ty = field_type;
        let mut wrapper_type_id = None;

        loop {
            match &ty.base {
                ast::BaseType::List(inner) => {
                    let wrapper = WrapperType {
                        kind: if ty.nullable {
                            WrapperTypeKind::List
                        } else {
                            WrapperTypeKind::NonNullList
                        },
                        outer: wrapper_type_id,
                    };

                    wrapper_type_id = Some(WrapperTypeId(self.field_types.wrappers.insert_full(wrapper).0));
                    ty = inner.as_ref();
                }

                ast::BaseType::Named(name) => {
                    let ty = InnerFieldType {
                        is_required: !ty.nullable,
                        name: self.strings.intern(name.as_str()),
                        wrapper: wrapper_type_id,
                    };
                    return FieldTypeId(self.field_types.inner_types.insert_full(ty).0);
                }
            }
        }
    }
}

pub(crate) type FieldTypeWalker<'a> = Walker<'a, FieldTypeId>;

impl<'a> FieldTypeWalker<'a> {
    fn inner(self) -> &'a InnerFieldType {
        self.subgraphs.field_types.inner_types.get_index(self.id.0).unwrap()
    }

    /// The definition with the name returned by `type_name` in `subgraph`.
    pub(crate) fn definition(self, subgraph: SubgraphId) -> Option<DefinitionWalker<'a>> {
        self.subgraphs
            .definition_by_name_id(self.type_name().id, subgraph)
            .map(|id| self.walk(id))
    }

    /// Compose two field types for input. The most required of the two is picked.
    pub(crate) fn compose_for_input(self, other: Self) -> Option<Self> {
        Some(if self.compose(other)? { other } else { self })
    }

    /// Compose two field types for output. The less required of the two is picked.
    pub(crate) fn compose_for_output(self, other: Self) -> Option<Self> {
        Some(if self.compose(other)? { self } else { other })
    }

    /// Returns whether `other` is nonnullable. This is enough to make a decision about which to
    /// pick. The function returns `None` whenever the two types mismatch to such extend that they
    /// can't be composed.
    fn compose(self, other: Self) -> Option<bool> {
        // This should be the most frequent path: the two types are identical.
        if self.id == other.id {
            return Some(true); // true or false doesn't matter, they're identical
        }

        if self.inner().name != other.inner().name {
            return None;
        }

        let mut self_wrappers = self.iter_wrappers();
        let mut other_wrappers = other.iter_wrappers();
        let mut zipped_wrappers = (&mut self_wrappers).zip(&mut other_wrappers).peekable();

        // Check that the inner requiredness matches if there are wrappers.
        if zipped_wrappers.peek().is_some() && (self.inner_is_required() != other.inner_is_required()) {
            return None;
        }

        while let Some((self_wrapper, other_wrapper)) = zipped_wrappers.next() {
            if zipped_wrappers.peek().is_none() {
                // The wrappers should have the same level of nesting.
                if self_wrappers.next().is_some() || other_wrappers.next().is_some() {
                    return None;
                }

                // We reached the outermost list wrappers: return which is required.
                return Some(matches!(other_wrapper, WrapperTypeKind::NonNullList));
            }

            // Inner list wrappers do not match in nullability.
            if self_wrapper != other_wrapper {
                return None;
            }
        }

        Some(other.is_required())
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   nested: [Nested!]!
    ///            ^^^^^^
    /// }
    /// ```
    pub(crate) fn type_name(self) -> StringWalker<'a> {
        self.walk(self.inner().name)
    }

    /// Iterate wrapper types from the innermost to the outermost.
    pub(crate) fn iter_wrappers(self) -> impl Iterator<Item = WrapperTypeKind> + 'a {
        let inner = self.inner();
        let mut wrapper = inner.wrapper;
        std::iter::from_fn(move || {
            let next = self.subgraphs.field_types.wrappers.get_index(wrapper?.0)?;
            wrapper = next.outer;
            Some(next.kind)
        })
    }

    pub(crate) fn inner_is_required(self) -> bool {
        self.inner().is_required
    }

    pub(crate) fn is_required(self) -> bool {
        self.iter_wrappers()
            .last()
            .map(|wrapper| match wrapper {
                WrapperTypeKind::List => false,
                WrapperTypeKind::NonNullList => true,
            })
            .unwrap_or_else(|| self.inner().is_required)
    }
}

impl std::fmt::Display for FieldTypeWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = self.type_name().as_str().to_owned();

        if self.inner_is_required() {
            out = format!("{}!", out);
        }

        for wrapper in self.iter_wrappers() {
            out = match wrapper {
                WrapperTypeKind::List => format!("[{out}]"),
                WrapperTypeKind::NonNullList => format!("[{out}]!"),
            };
        }

        f.write_str(&out)
    }
}
