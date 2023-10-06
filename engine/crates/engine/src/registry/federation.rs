//! Registry structs for using the engine as an [apollo federation subgraph][1]
//!
//! [1]: https://www.apollographql.com/docs/federation/subgraph-spec

use serde_json::{Map, Value};

use super::resolvers::http::HttpResolver;

/// Federation details for a particular entity
///
/// There should be one instance of this for each MetaType that represents
/// a federation entity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct FederationEntity {
    keys: Vec<FederationKey>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FederationResolver {
    DynamoUnique,
    Http(HttpResolver),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct FederationKey {
    selections: Vec<Selection>,
    resolver: FederationResolver,
}

impl FederationKey {
    pub fn single(field: impl Into<String>, resolver: FederationResolver) -> Self {
        FederationKey {
            selections: vec![Selection {
                field: field.into(),
                selections: vec![],
            }],
            resolver,
        }
    }

    pub fn multiple(fields: Vec<String>, resolver: FederationResolver) -> Self {
        FederationKey {
            selections: fields
                .into_iter()
                .map(|field| Selection {
                    field,
                    selections: vec![],
                })
                .collect(),
            resolver,
        }
    }

    pub fn resolver(&self) -> &FederationResolver {
        &self.resolver
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
struct Selection {
    field: String,
    selections: Vec<Selection>,
}

impl FederationEntity {
    /// The keys for this entity in the string format expected in federation SDL
    /// e.g. `fieldOne fieldTwo { someNestedField }`
    pub fn key_strings(&self) -> impl Iterator<Item = String> + '_ {
        self.keys.iter().map(ToString::to_string)
    }

    /// Takes an `_Any` representation from the federation `_entities` field and determines
    /// which `FederationKey` the representation matches.
    pub(crate) fn find_key(&self, data: &Value) -> Option<&FederationKey> {
        let object = data.as_object()?;
        self.keys
            .iter()
            .find(|key| selections_are_present(object, &key.selections))
    }
}

fn selections_are_present(object: &Map<String, Value>, selections: &[Selection]) -> bool {
    selections.iter().all(|selection| {
        if !object.contains_key(&selection.field) {
            return false;
        }
        if selection.selections.is_empty() {
            return true;
        }
        // Make sure any sub-selections are also present
        let Some(object) = object.get(&selection.field).and_then(Value::as_object) else {
            return false;
        };
        selections_are_present(object, &selection.selections)
    })
}

impl std::fmt::Display for FederationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, selection) in self.selections.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{selection}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Selection { field, selections } = self;
        write!(f, "{field}")?;
        if !selections.is_empty() {
            write!(f, " {{")?;
            for (i, selection) in selections.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?;
                }
                write!(f, "{selection}")?;
            }
            write!(f, "}}")?;
        }
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
