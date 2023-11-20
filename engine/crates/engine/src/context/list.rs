use std::borrow::Cow;

use engine_parser::{types::Type, Pos};
use ulid::Ulid;

use crate::{
    registry::type_kinds::OutputType, Context, ContextField, ContextSelectionSet, QueryEnv, QueryPath,
    QueryPathSegment, SchemaEnv,
};

/// Context when we're resolving a list field
///
/// This is likely to be less widely used than the other contexts - it's mostly an
/// intermediate type for the resolver_utils
#[derive(Clone, Debug)]
pub struct ContextList<'a> {
    /// The current type we are resolving.
    ///
    /// I'm using the parser::Type here because it's well suited for recursing through lists
    pub current_type: Cow<'a, Type>,

    /// The context of the field that contains the list
    pub field_context: &'a ContextField<'a>,

    /// The current path within query
    pub path: QueryPath,
}

/// The result of indexing into a ContextList
#[derive(Clone, Debug)]
pub enum ContextWithIndex<'a> {
    List(ContextList<'a>),
    Field(ContextField<'a>),
    SelectionSet(ContextSelectionSet<'a>),
}

impl<'a> ContextList<'a> {
    pub fn pos(&self) -> Pos {
        self.field_context.item.pos
    }

    pub fn list_is_nullable(&self) -> bool {
        self.current_type.nullable
    }

    pub fn contents_are_non_null(&self) -> Option<bool> {
        match &self.current_type.base {
            engine_parser::types::BaseType::Named(_) => None,
            engine_parser::types::BaseType::List(inner) => Some(!inner.nullable),
        }
    }

    #[must_use]
    pub fn with_index(&'a self, idx: usize) -> ContextWithIndex<'a> {
        let mut path = self.path.clone();
        path.push(QueryPathSegment::Index(idx));
        let engine_parser::types::BaseType::List(next) = &self.current_type.base else {
            unreachable!("We shouldn't have a ContextList if we're not a list");
        };
        let next = next.as_ref();
        if let engine_parser::types::BaseType::List(_) = &next.base {
            return ContextWithIndex::List(ContextList {
                current_type: Cow::Borrowed(next),
                path,
                field_context: self.field_context,
            });
        }

        // If we get here we've reached the end of the lists and need to return a field/selection set context
        match self.field_context.field_base_type() {
            OutputType::Scalar(_) | OutputType::Enum(_) => ContextWithIndex::Field(ContextField {
                path,
                execution_id: Ulid::from_datetime(self.field_context.query_env.current_datetime.clone().into()),
                field: self.field_context.field,
                item: self.field_context.item,
                parent_type: self.field_context.parent_type,
                schema_env: self.field_context.schema_env,
                query_env: self.field_context.query_env,
            }),
            ty @ (OutputType::Object(_) | OutputType::Interface(_) | OutputType::Union(_)) => {
                ContextWithIndex::SelectionSet(ContextSelectionSet {
                    ty: ty.try_into().expect("already verified it's a selection target"),
                    path,
                    item: &self.field_context.item.selection_set,
                    schema_env: self.field_context.schema_env,
                    query_env: self.field_context.query_env,
                })
            }
        }
    }
}

impl<'a> super::ContextField<'a> {
    /// Converts this into a ContextList
    pub fn to_list_context(&'a self) -> ContextList<'a> {
        ContextList {
            current_type: Cow::Owned(
                engine_parser::types::Type::new(self.field.ty.as_str()).expect("type names to be well formed"),
            ),
            path: self.path.clone(),
            field_context: self,
        }
    }
}

impl<'a> Context<'a> for ContextList<'a> {
    fn path(&self) -> &QueryPath {
        &self.path
    }

    fn query_env(&self) -> &'a QueryEnv {
        self.field_context.query_env()
    }

    fn schema_env(&self) -> &'a SchemaEnv {
        self.field_context.schema_env()
    }
}
