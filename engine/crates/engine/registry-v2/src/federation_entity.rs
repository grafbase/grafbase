use crate::{
    misc_types::FieldSet,
    resolvers::{http::HttpResolver, join::JoinResolver},
    Selection,
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
    /// Makes an HTTP call to resolve
    Http(Box<HttpResolver>),
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
    /// This entity resolves to a specific field in the schema using the same mechanism
    /// as a JoinResolver
    Join(Box<JoinResolver>),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct FederationKey {
    pub selections: FieldSet,
    pub resolver: Option<FederationResolver>,
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

    pub fn unresolvable(selections: FieldSet) -> Self {
        FederationKey {
            selections,
            resolver: None,
        }
    }

    pub fn basic_type(selections: FieldSet) -> Self {
        FederationKey {
            selections,
            resolver: Some(FederationResolver::BasicType),
        }
    }

    pub fn join(selections: FieldSet, resolver: JoinResolver) -> Self {
        FederationKey {
            selections,
            resolver: Some(FederationResolver::Join(Box::new(resolver))),
        }
    }

    pub fn resolver(&self) -> Option<&FederationResolver> {
        self.resolver.as_ref()
    }

    pub fn is_resolvable(&self) -> bool {
        self.resolver.is_some()
    }

    pub fn includes_field(&self, field: &str) -> bool {
        self.selections.0.iter().any(|selection| selection.field == field)
    }
}
