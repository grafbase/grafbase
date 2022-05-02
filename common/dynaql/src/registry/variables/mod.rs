//! Variable Resolving definition strategy is explained here.
//!
//! When you need a Variable inside a Resolver, you can use a
//! `VariableResolveDefinition` struct to define how the graphql server should
//! resolve this variable.

use crate::{context::resolver_data_get_opt_ref, Context, Value};

/// Describe what should be done by the GraphQL Server to resolve this Variable.
#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum VariableResolveDefinition {
    /// A Debug VariableResolveDefinition where you can just put the Value you
    /// would like to have.
    /// This VariableResolveDefinition is not made to be inside the Registry.
    #[serde(skip)]
    DebugString(String),
    /// Check the last Resolver in the Query Graph and try to resolve the
    /// variable defined in this field.
    InputTypeName(String),
    /// Resolve a Value by querying the ResolverContextData with a key_id.
    /// What is store in the ResolverContextData is described on each Resolver
    /// implementation.
    ResolverData(String),
}

impl VariableResolveDefinition {
    /// Resolve the first variable with this definition
    pub fn param<'a>(&self, ctx: &'a Context<'a>) -> Option<Value> {
        match self {
            Self::InputTypeName(name) => {
                ctx.query_resolvers.iter().rev().find_map(|(_, _, _, x)| {
                    x.as_ref()
                        .map(|y| y.iter().find(|(var_name, _)| var_name == name))
                        .flatten()
                        .map(|(_, x)| x.clone())
                })
            }
            Self::ResolverData(key) => {
                resolver_data_get_opt_ref::<Value>(&ctx.resolvers_data.read().expect("handle"), key)
                    .map(std::clone::Clone::clone)
            }
            Self::DebugString(inner) => Some(Value::String(inner.clone())),
        }
    }
}
