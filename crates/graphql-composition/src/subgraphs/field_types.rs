use cynic_parser::type_system as ast;
use wrapping::{ListWrapping, Wrapping};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldType {
    pub definition_name_id: StringId,
    pub wrapping: Wrapping,
}

impl Subgraphs {
    pub(crate) fn intern_field_type(&mut self, field_type: ast::Type<'_>) -> FieldType {
        use cynic_parser::common::WrappingType;

        let wrappers = field_type.wrappers().collect::<Vec<_>>();
        let mut wrappers = wrappers.into_iter().rev().peekable();

        let mut wrapping = if wrappers.next_if(|w| matches!(w, WrappingType::NonNull)).is_some() {
            wrapping::Wrapping::default().non_null()
        } else {
            wrapping::Wrapping::default()
        };

        while let Some(next) = wrappers.next() {
            debug_assert_eq!(next, WrappingType::List, "double non-null wrapping type not possible");

            wrapping = if wrappers.next_if(|w| matches!(w, WrappingType::NonNull)).is_some() {
                wrapping.list_non_null()
            } else {
                wrapping.list()
            }
        }

        FieldType {
            definition_name_id: self.strings.intern(field_type.name()),
            wrapping,
        }
    }
}

pub(crate) type FieldTypeWalker<'a> = Walker<'a, FieldType>;

impl<'a> FieldTypeWalker<'a> {
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

        if self.id.definition_name_id != other.id.definition_name_id {
            return None;
        }

        let mut self_wrappers = self.id.wrapping.list_wrappings();
        let mut other_wrappers = other.id.wrapping.list_wrappings();
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
                return Some(matches!(other_wrapper, ListWrapping::ListNonNull));
            }

            // Inner list wrappers do not match in nullability.
            if self_wrapper != other_wrapper {
                return None;
            }
        }

        Some(other.is_required())
    }

    pub(crate) fn is_list(self) -> bool {
        self.id.wrapping.is_list()
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   nested: [Nested!]!
    ///            ^^^^^^
    /// }
    /// ```
    pub(crate) fn type_name(self) -> StringWalker<'a> {
        self.walk(self.id.definition_name_id)
    }

    pub(crate) fn inner_is_required(self) -> bool {
        self.id.wrapping.inner_is_required()
    }

    pub(crate) fn is_required(self) -> bool {
        self.id.wrapping.is_required()
    }
}

impl std::fmt::Display for FieldTypeWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id.wrapping.type_display(self.type_name().as_str()))
    }
}

impl PartialEq for FieldTypeWalker<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
