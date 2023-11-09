use super::*;
use async_graphql_parser::types as ast;
use indexmap::IndexSet;

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

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
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
                        kind: if inner.nullable {
                            WrapperTypeKind::List
                        } else {
                            WrapperTypeKind::NonNullList
                        },
                        outer: wrapper_type_id,
                    };

                    wrapper_type_id = Some(WrapperTypeId(
                        self.field_types.wrappers.insert_full(wrapper).0,
                    ));
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
        self.subgraphs
            .field_types
            .inner_types
            .get_index(self.id.0)
            .unwrap()
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
