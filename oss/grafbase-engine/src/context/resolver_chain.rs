use std::fmt::{self, Debug, Display, Formatter};

use grafbase_engine_parser::{
    types::{Field, SelectionSet},
    Positioned,
};
use ulid::Ulid;

use crate::{
    registry::{resolvers::Resolver, MetaField, MetaInputValue, MetaType},
    QueryPathSegment, Result,
};

/// Holds some metadata about the current node in the query.
///
/// This is a hold-over from some old code and we should absolutely squash it into
/// ContextField or ContextSelectionSet at some point.
///
/// Part of a reverse linked list, so you can query up the chain of query nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverChainNode<'a> {
    /// The parent node to this, if there is one.
    pub parent: Option<&'a ResolverChainNode<'a>>,

    /// The current path segment being resolved.
    pub segment: QueryPathSegment<'a>,

    /// The current field being resolved if we know it.
    pub field: Option<&'a MetaField>,

    /// The current field being resolved if we know it.
    pub executable_field: Option<&'a Positioned<Field>>,

    /// The current Type being resolved if we know it.
    pub ty: Option<&'a MetaType>,

    /// The current SelectionSet.
    pub selections: Option<&'a SelectionSet>,

    /// The current execution_id for this node.
    /// A ResolverChainNode must have a execution_id to allow caching.
    pub execution_id: Ulid,

    /// The current resolver to apply, if it exists.
    /// There is no resolvers on QueryPathSegment::Index for instance.
    pub resolver: Option<&'a Resolver>,

    /// The current variables on this node
    pub variables: Option<Vec<(&'a str, &'a MetaInputValue)>>,
}

impl<'a> Display for ResolverChainNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        self.try_for_each(|segment| {
            if !first {
                write!(f, ".")?;
            }
            first = false;

            match segment {
                QueryPathSegment::Index(idx) => write!(f, "{}", *idx),
                QueryPathSegment::Name(name) => write!(f, "{name}"),
            }
        })
    }
}

#[derive(serde::Serialize)]
pub struct ResponsePath<'a> {
    key: &'a str,
    prev: Option<Box<ResponsePath<'a>>>,
    typename: Option<&'a str>,
}

impl<'a> ResolverChainNode<'a> {
    /// Get the current field name.
    ///
    /// This traverses all the parents of the node until it finds one that is a field name.
    pub fn field_name(&self) -> &str {
        std::iter::once(self)
            .chain(self.parents())
            .find_map(|node| match node.segment {
                QueryPathSegment::Name(name) => Some(name),
                QueryPathSegment::Index(_) => None,
            })
            .unwrap()
    }

    /// https://graphql-js.org/api/interface/ResponsePath
    pub fn to_response_path(&self) -> ResponsePath<'_> {
        ResponsePath {
            key: self.field_name(),
            prev: self.parent.as_ref().map(|parent| Box::new(parent.to_response_path())),
            typename: self.ty.map(crate::registry::MetaType::name),
        }
    }

    /// Iterate over the parents of the node.
    pub fn parents(&self) -> ResolversParents<'_> {
        ResolversParents(self)
    }

    pub(crate) fn try_for_each<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(&self, mut f: F) -> Result<(), E> {
        self.try_for_each_ref(&mut f)
    }

    fn try_for_each_ref<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(&self, f: &mut F) -> Result<(), E> {
        if let Some(parent) = &self.parent {
            parent.try_for_each_ref(f)?;
        }
        f(&self.segment)
    }
}

/// An iterator over the parents of a [`ResolverChainNode`](struct.ResolverChainNode.html).
#[derive(Debug, Clone)]
pub struct ResolversParents<'a>(&'a ResolverChainNode<'a>);

impl<'a> Iterator for ResolversParents<'a> {
    type Item = &'a ResolverChainNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.0.parent;
        if let Some(parent) = parent {
            self.0 = parent;
        }
        parent
    }
}
