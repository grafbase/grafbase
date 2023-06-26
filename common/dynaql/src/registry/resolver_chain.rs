//! ResolvingChain is a main component which translate the Resolving logic into Rust
//!
//! In Graphql you can have a Query like:
//!
//! ```graphql
//! query {
//!   container {
//!     a
//!     b
//!     list {
//!       c
//!     }
//!   }
//! }
//! ```
//!
//! For the resolving to work, we we'll traverse the whole query like this:
//!
//! resolve(query)
//!   json -> resolve(container)
//!     -> resolve(a)
//!     -> resolve(b)
//!     -> resolve list
//!       -> resolve(c)
//!
//! Resolve function:
//! fn resolve ->
//!     resolve_parent
//!     resolve_current(resolve_parent);
//!     index;
//!     transform;
//!
//! A memoization is applied on the resolve function.

use crate::registry::{resolvers::Resolver, transformers::Transformer};
use crate::Result;
use crate::{Context, Error, QueryPathSegment};
use cached::Cached;
use dynaql_parser::{
    types::{Field, SelectionSet},
    Positioned,
};
use dynaql_value::{Name, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::ser::{SerializeSeq, Serializer};
use std::fmt::{self, Debug, Display, Formatter};
use ulid::Ulid;

use super::{
    resolvers::{ResolvedValue, ResolverContext, ResolverTrait},
    MetaField, MetaInputValue, MetaType,
};

/// A path to the current query with resolvers, transformers and associated type.
/// Reverse linked list used to help us construct the whole resolving flow.
#[derive(derivative::Derivative, Debug, Clone, PartialEq, Eq)]
#[derivative(Hash)]
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

    /// The current transformer applied to the current resolver, if it exists.
    pub transformer: Option<&'a Transformer>,

    /// The current variables on this node
    pub variables: Option<Vec<(&'a str, &'a MetaInputValue)>>,
}

impl<'a> ResolverChainNode<'a> {
    fn get_variable_by_name_internal(&self, name: &str) -> Option<&'a MetaInputValue> {
        self.variables.as_ref().and_then(|variables| {
            variables.iter().find_map(|(_, value)| {
                if name == value.name {
                    Some(*value)
                } else {
                    None
                }
            })
        })
    }

    /// Get the closest variable with this name
    pub fn get_variable_by_name(&self, name: &str) -> Option<&'_ MetaInputValue> {
        std::iter::once(self)
            .chain(self.parents())
            .find_map(|x| x.get_variable_by_name_internal(name))
    }

    fn get_arguments_internal(
        &'a self,
    ) -> Box<dyn Iterator<Item = (Positioned<Name>, Positioned<Value>)> + 'a> {
        match (self.field, self.executable_field) {
            (Some(field), Some(executable_field)) => {
                let arguments = field.args.iter().map(|(field_argument_name, _)| {
                    match executable_field
                        .node
                        .arguments
                        .iter()
                        .find(|(name, _)| name.node.as_str() == field_argument_name)
                    {
                        Some(executable_field_argument) => executable_field_argument.clone(),
                        None => (
                            Positioned::new(Name::new(field_argument_name), executable_field.pos),
                            Positioned::new(Value::Null, executable_field.pos),
                        ),
                    }
                });
                Box::new(arguments)
            }
            (None, Some(executable_field)) => {
                // TODO: Remove cloning when reworking the variable resolution.
                Box::new(executable_field.node.arguments.clone().into_iter())
            }
            _ => Box::new(std::iter::empty::<(Positioned<Name>, Positioned<Value>)>()),
        }
    }

    /// Get all arguments
    pub fn get_arguments(
        &'a self,
    ) -> Box<dyn Iterator<Item = (Positioned<Name>, Positioned<Value>)> + 'a> {
        Box::new(
            std::iter::once(self)
                .chain(self.parents())
                .flat_map(ResolverChainNode::get_arguments_internal),
        )
    }
}

#[async_trait::async_trait]
impl<'a> ResolverTrait for ResolverChainNode<'a> {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let mut hash_value = DefaultHasher::new();
        self.hash(&mut hash_value);
        let hash_value = hash_value.finish();
        {
            let mut guard = ctx.resolvers_cache.write().await;
            if let Some(value) = guard.cache_get(&hash_value) {
                let cached_value = value.clone();
                return cached_value;
            }
        }
        let mut final_result = ResolvedValue::new(Arc::new(serde_json::Value::Null));

        if let Some(parent) = self.parent {
            let parent_ctx = ResolverContext::new(&parent.execution_id)
                .with_ty(parent.ty)
                .with_resolver_id(parent.resolver.and_then(|resolver| resolver.id.as_deref()))
                .with_selection_set(parent.selections)
                .with_field(parent.field);
            final_result = parent
                .resolve(ctx, &parent_ctx, last_resolver_value)
                .await?;
        }

        if let QueryPathSegment::Index(idx) = self.segment {
            // If we are in a segment, it means we do not have a current resolver (YET).
            final_result = ResolvedValue::new(Arc::new(
                final_result
                    .data_resolved
                    .as_ref()
                    .get(idx)
                    .map(Clone::clone)
                    .unwrap_or(serde_json::Value::Null),
            ));
        }

        if let Some(actual) = self.resolver {
            let current_ctx = ResolverContext::new(&self.execution_id)
                .with_resolver_id(actual.id.as_deref())
                .with_ty(self.ty)
                .with_selection_set(self.selections)
                .with_field(self.field);

            final_result = actual
                .resolve(ctx, &current_ctx, Some(&final_result))
                .await?;
        }

        if let Some(transformers) = self.transformer {
            // TODO: Ensure it doesn't fail by changing the way the data is modelized withing
            // resolver, we shouldn't have dynamic typing here.
            //
            // It can be done by transforming the Resolver Return type to a struct with the result
            // and with the extra data, where each resolver can add extra data.
            final_result.data_resolved =
                Arc::new(transformers.transform(final_result.data_resolved.as_ref().clone())?);

            if *final_result.data_resolved == serde_json::Value::Null {
                final_result = final_result.with_early_return();
            }
        }

        {
            let mut guard = ctx.resolvers_cache.write().await;
            guard.cache_set(hash_value, Ok(final_result.clone()));
        }

        Ok(final_result)
    }
}

impl<'a> serde::Serialize for ResolverChainNode<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        self.try_for_each(|segment| seq.serialize_element(segment))?;
        seq.end()
    }
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

    /// Get the path represented by `Vec<String>`; numbers will be stringified.
    #[must_use]
    pub fn to_string_vec(self) -> Vec<String> {
        let mut res = Vec::new();
        self.for_each(|s| {
            res.push(match s {
                QueryPathSegment::Name(name) => (*name).to_string(),
                QueryPathSegment::Index(idx) => idx.to_string(),
            });
        });
        res
    }

    /// https://graphql-js.org/api/interface/ResponsePath
    pub fn to_response_path(&self) -> ResponsePath<'_> {
        ResponsePath {
            key: self.field_name(),
            prev: self
                .parent
                .as_ref()
                .map(|parent| Box::new(parent.to_response_path())),
            typename: self.ty.map(crate::registry::MetaType::name),
        }
    }

    /// Iterate over the parents of the node.
    pub fn parents(&self) -> ResolversParents<'_> {
        ResolversParents(self)
    }

    pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
        let _ = self.try_for_each::<std::convert::Infallible, _>(|segment| {
            f(segment);
            Ok(())
        });
    }

    pub(crate) fn try_for_each<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        self.try_for_each_ref(&mut f)
    }

    fn try_for_each_ref<E, F: FnMut(&QueryPathSegment<'a>) -> Result<(), E>>(
        &self,
        f: &mut F,
    ) -> Result<(), E> {
        if let Some(parent) = &self.parent {
            parent.try_for_each_ref(f)?;
        }
        f(&self.segment)
    }
}

/// An iterator over the parents of a [`ResolverChainNode`](struct.ResolverChainNode.html).
#[derive(Debug, Clone)]
pub struct ResolversParents<'a>(&'a ResolverChainNode<'a>);

impl<'a> ResolversParents<'a> {
    /// Get the current query path node, which the next call to `next` will get the parents of.
    #[must_use]
    pub fn current(&self) -> &'a ResolverChainNode<'a> {
        self.0
    }
}

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
