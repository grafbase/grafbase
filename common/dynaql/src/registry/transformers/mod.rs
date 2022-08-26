//! Transformers are a part of the Resolving Logic, Transformers are applied in
//! the resolving, after executing the async code to resolve a field, we apply
//! the transformers associated.
//!
//! Each Transformer function is defined to do:
//! serde_json::Value -> serde_json::Value
//!
//! The transform step is synchronous.
//!
//! At the end of the transformation, each transformed values are merged into one
//! serde_json::Value.

use super::utils::{attribute_to_value, merge};
use crate::Error;
use dynomite::AttributeValue;
use std::collections::HashMap;

/// Describe the Transformer step used to transform a Value from the Resolver.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Transformer {
    Functions {
        /// Functions to be applied to a Value.
        functions: Vec<JSONFunction>,
    },
    DynamoSelect {
        /// The key where this select
        property: String,
    },
    JSONSelect {
        /// The key where this select
        property: String,
        /// Functions to be applied to a Value.
        functions: Vec<JSONFunction>,
    },
}

/// JSONFunction to be applied to the Value
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum JSONFunction {
    ExtractCompositeID,
}

#[async_trait::async_trait]
pub trait TransformerTrait {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error>;
}

impl<'a, T: TransformerTrait + 'a> TransformerTrait for &'a T {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        (**self).transform(value)
    }
}

impl<'a, T: TransformerTrait + 'a> TransformerTrait for Box<T> {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        (**self).transform(value)
    }
}

/// Empty Value
pub struct TransformerNil;

#[derive(Debug)]
pub enum TransformationVisitor<T: TransformerTrait> {
    Nil,
    Cons(T, Box<TransformationVisitor<T>>),
}

impl<'a, T: TransformerTrait + 'a> TransformerTrait for TransformationVisitor<T> {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        match self {
            Self::Nil => Ok(value),
            Self::Cons(v, rest) => v.transform(rest.transform(value)?),
        }
    }
}

/// Concat rule
pub struct TransformerCons<A: TransformerTrait, B: TransformerTrait>(A, B);

impl TransformerTrait for TransformerNil {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        Ok(value)
    }
}

/// The monoid implementation for Visitor
impl<A, B> TransformerTrait for TransformerCons<A, B>
where
    A: TransformerTrait,
    B: TransformerTrait,
{
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        self.1.transform(self.0.transform(value)?)
    }
}

impl TransformerTrait for Transformer {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        match &self {
            Self::Functions { .. } => unimplemented!(),
            Self::JSONSelect { property, .. } => {
                let result = value
                    .get(property)
                    .map(std::borrow::ToOwned::to_owned)
                    .unwrap_or_else(|| serde_json::Value::Null);
                Ok(result)
            }
            Self::DynamoSelect { property } => {
                let cast: Option<HashMap<String, AttributeValue>> = serde_json::from_value(value)?;

                let result = cast
                    .map(|mut x| x.remove(property))
                    .flatten()
                    .map(attribute_to_value)
                    .unwrap_or_else(|| serde_json::Value::Null);

                Ok(result)
            }
        }
    }
}

impl TransformerTrait for Vec<Transformer> {
    fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        self.iter()
            .map(|x| x.transform(value.clone()))
            .collect::<Result<Vec<serde_json::Value>, Error>>()
            .map(|x| {
                x.into_iter().fold(serde_json::json!({}), |mut acc, cur| {
                    merge(&mut acc, cur);
                    acc
                })
            })
    }
}
