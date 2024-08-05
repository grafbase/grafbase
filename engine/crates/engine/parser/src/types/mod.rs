//! GraphQL types.
//!
//! The two root types are [`ExecutableDocument`](struct.ExecutableDocument.html) and
//! [`ServiceDocument`](struct.ServiceDocument.html), representing an executable GraphQL query and a
//! GraphQL service respectively.
//!
//! This follows the [June 2018 edition of the GraphQL spec](https://spec.graphql.org/October2021/).

mod executable;
mod service;

use std::{
    collections::{hash_map, HashMap},
    fmt::{self, Display, Formatter, Write},
};

use engine_value::{ConstValue, Name, Value};
pub use executable::*;
use serde::{Deserialize, Serialize};
pub use service::*;

use crate::pos::Positioned;

/// The name of a directive that can be considered as a model.
pub const MODEL_DIRECTIVE: &str = "model";

/// The name of a directive representing authentication.
pub const AUTH_DIRECTIVE: &str = "auth";

/// The name of a search directive.
pub const SEARCH_DIRECTIVE: &str = "search";

/// The type of an operation; `query`, `mutation` or `subscription`.
///
/// [Reference](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// A query.
    Query,
    /// A mutation.
    Mutation,
    /// A subscription.
    Subscription,
}

impl From<OperationType> for grafbase_telemetry::metrics::OperationType {
    fn from(value: OperationType) -> Self {
        match value {
            OperationType::Query => Self::Query,
            OperationType::Mutation => Self::Mutation,
            OperationType::Subscription => Self::Subscription,
        }
    }
}

impl OperationType {
    /// Operation type as str
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }

    /// Returns `true` if the operation type is [`Mutation`].
    ///
    /// [`Mutation`]: OperationType::Mutation
    #[must_use]
    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation)
    }
}

impl AsRef<str> for OperationType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// A GraphQL type, for example `String` or `[String!]!`.
///
/// [Reference](https://spec.graphql.org/October2021/#Type).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    /// The base type.
    pub base: BaseType,
    /// Whether the type is nullable.
    pub nullable: bool,
}

impl Type {
    /// Create a type from the type string.
    #[must_use]
    pub fn new(ty: &str) -> Option<Self> {
        let (nullable, ty) = ty.strip_suffix('!').map_or((true, ty), |rest| (false, rest));

        Some(Self {
            base: if let Some(ty) = ty.strip_prefix('[') {
                BaseType::List(Box::new(Self::new(ty.strip_suffix(']')?)?))
            } else {
                BaseType::Named(Name::new(ty))
            },
            nullable,
        })
    }

    /// Create a required Type
    pub fn required(base: BaseType) -> Self {
        Type { base, nullable: false }
    }

    /// Create a nullable Type
    pub fn nullable(base: BaseType) -> Self {
        Type { base, nullable: true }
    }

    /// Create a new Type with its base overridden by the new base type.
    #[must_use]
    pub fn override_base(&self, base: BaseType) -> Self {
        Self {
            base: match &self.base {
                BaseType::Named(_) => base,
                BaseType::List(list) => BaseType::List(Box::new(list.override_base(base))),
            },
            nullable: self.nullable,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.base.fmt(f)?;
        if !self.nullable {
            f.write_char('!')?;
        }
        Ok(())
    }
}

impl From<Type> for String {
    fn from(val: Type) -> Self {
        format!("{val}")
    }
}

/// A GraphQL base type, for example `String` or `[String!]`. This does not include whether the
/// type is nullable; for that see [Type](struct.Type.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BaseType {
    /// A named type, such as `String`.
    Named(Name),
    /// A list type, such as `[String]`.
    List(Box<Type>),
}

impl BaseType {
    /// Create a new named BaseType
    pub fn named(name: &str) -> BaseType {
        BaseType::Named(Name::new(name))
    }

    /// Create a new list BaseType
    pub fn list(ty: Type) -> BaseType {
        BaseType::List(Box::new(ty))
    }

    /// Check the base type is a list
    pub fn is_list(&self) -> bool {
        matches!(self, BaseType::List(_))
    }
}

impl BaseType {
    /// Get the primitive type from a BaseType
    pub fn to_base_type_str(&self) -> &str {
        match self {
            BaseType::Named(name) => name,
            BaseType::List(ty_list) => ty_list.base.to_base_type_str(),
        }
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::List(ty) => write!(f, "[{ty}]"),
        }
    }
}

/// A const GraphQL directive, such as `@deprecated(reason: "Use the other field)`. This differs
/// from [`Directive`](struct.Directive.html) in that it uses [`ConstValue`](enum.ConstValue.html)
/// instead of [`Value`](enum.Value.html).
///
/// [Reference](https://spec.graphql.org/October2021/#Directive).
#[derive(Debug, Clone)]
pub struct ConstDirective {
    /// The name of the directive.
    pub name: Positioned<Name>,
    /// The arguments to the directive.
    pub arguments: Vec<(Positioned<Name>, Positioned<ConstValue>)>,
}

impl ConstDirective {
    /// Convert this `ConstDirective` into a `Directive`.
    #[must_use]
    pub fn into_directive(self) -> Directive {
        Directive {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| (name, value.map(ConstValue::into_value)))
                .collect(),
        }
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<ConstValue>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }

    /// Is the directive a model.
    pub fn is_model(&self) -> bool {
        self.name.as_str() == MODEL_DIRECTIVE
    }

    /// Is an auth directive.
    pub fn is_auth(&self) -> bool {
        self.name.as_str() == AUTH_DIRECTIVE
    }

    /// Is a search directive.
    pub fn is_search(&self) -> bool {
        self.name.as_str() == SEARCH_DIRECTIVE
    }
}

/// A GraphQL directive, such as `@deprecated(reason: "Use the other field")`.
///
/// [Reference](https://spec.graphql.org/October2021/#Directive).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Directive {
    /// The name of the directive.
    pub name: Positioned<Name>,
    /// The arguments to the directive.
    pub arguments: Vec<(Positioned<Name>, Positioned<Value>)>,
}

impl Directive {
    /// Attempt to convert this `Directive` into a `ConstDirective`.
    #[must_use]
    pub fn into_const(self) -> Option<ConstDirective> {
        Some(ConstDirective {
            name: self.name,
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, value)| Some((name, Positioned::new(value.node.into_const()?, value.pos))))
                .collect::<Option<_>>()?,
        })
    }

    /// Get the argument with the given name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}
