//! Registry structs for using the engine as an [apollo federation subgraph][1]
//!
//! [1]: https://www.apollographql.com/docs/federation/subgraph-spec

use serde_json::Value;

use super::{
    field_set::{FieldSet, Selection},
    resolvers::http::HttpResolver,
};

/// Federation details for a particular entity
///
/// There should be one instance of this for each MetaType that represents
/// a federation entity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct FederationEntity {
    pub keys: Vec<FederationKey>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FederationResolver {
    /// Fetches a dynamo entity by some unique key
    DynamoUnique,
    /// Makes an HTTP call to resolve
    Http(HttpResolver),
    /// This "resolver" doesn't actually resolve data in the same way the others do.
    ///
    /// This should be put on entities where the primary representation lives in
    /// another subgraph but we contribute fields to it - the result of resolution
    /// will be the representation we are passed from the router.
    ///
    /// This should only ever be applied to types where all the fields on that type
    /// are present in the representation or resolvable from the representation (e.g.
    /// fields with custom resolvers)
    BasicType,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct FederationKey {
    selections: FieldSet,
    resolver: Option<FederationResolver>,
}

impl FederationKey {
    pub fn single(field: impl Into<String>, resolver: FederationResolver) -> Self {
        FederationKey {
            selections: FieldSet::new([Selection {
                field: field.into(),
                selections: vec![],
            }]),
            resolver: Some(resolver),
        }
    }

    pub fn multiple(fields: Vec<String>, resolver: FederationResolver) -> Self {
        FederationKey {
            selections: FieldSet::new(fields.into_iter().map(|field| Selection {
                field,
                selections: vec![],
            })),
            resolver: Some(resolver),
        }
    }

    pub fn unresolvable(selections: Vec<Selection>) -> Self {
        FederationKey {
            selections: FieldSet::new(selections),
            resolver: None,
        }
    }

    pub fn basic_type(selections: Vec<Selection>) -> Self {
        FederationKey {
            selections: FieldSet::new(selections),
            resolver: Some(FederationResolver::BasicType),
        }
    }

    pub fn resolver(&self) -> Option<&FederationResolver> {
        self.resolver.as_ref()
    }

    pub fn is_resolvable(&self) -> bool {
        self.resolver.is_some()
    }
}

impl FederationEntity {
    /// The keys for this entity in the string format expected in federation SDL
    /// e.g. `fieldOne fieldTwo { someNestedField }`
    pub fn keys(&self) -> impl Iterator<Item = &FederationKey> + '_ {
        self.keys.iter()
    }

    /// Takes an `_Any` representation from the federation `_entities` field and determines
    /// which `FederationKey` the representation matches.
    pub(crate) fn find_key(&self, data: &Value) -> Option<&FederationKey> {
        let object = data.as_object()?;
        self.keys
            .iter()
            .find(|key| key.selections.all_fields_are_present(object))
    }
}

impl std::fmt::Display for FederationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.selections)?;
        Ok(())
    }
}

pub struct FederationEntityBuilder(FederationEntity);

impl FederationEntity {
    pub fn builder() -> FederationEntityBuilder {
        FederationEntityBuilder(FederationEntity::default())
    }
}

impl FederationEntityBuilder {
    pub fn with_keys(mut self, keys: Vec<FederationKey>) -> Self {
        self.0.keys.extend(keys);
        self
    }

    pub fn add_key(&mut self, key: FederationKey) {
        self.0.keys.push(key)
    }

    pub fn build(self) -> FederationEntity {
        self.0
    }
}
