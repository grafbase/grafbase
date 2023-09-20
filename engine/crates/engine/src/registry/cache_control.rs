use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    hash::Hash,
};

use engine_parser::types::OperationType::Query;
use inflector::Inflector;

use crate::registry::{MetaType, Registry};

/// Cache control values
///
/// # Examples
///
/// ```rust, ignore
/// use engine::*;
///
/// struct Query;
///
/// #[Object(cache_control(max_age = 60))]
/// impl Query {
///     #[graphql(cache_control(max_age = 30))]
///     async fn value1(&self) -> i32 {
///         0
///     }
///
///     #[graphql(cache_control(private))]
///     async fn value2(&self) -> i32 {
///         0
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
/// assert_eq!(schema.execute("{ value1 }").await.into_result().unwrap().cache_control, CacheControl { public: true, max_age: 30 });
/// assert_eq!(schema.execute("{ value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 60 });
/// assert_eq!(schema.execute("{ value1 value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 30 });
/// # });
/// ```
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, bool, usize)]
#[derive(Default, Hash, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
pub struct CacheControl {
    /// Scope is public, default is false.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    pub stale_while_revalidate: usize,

    /// Invalidation policy for mutations, default is None.
    pub invalidation_policy: Option<CacheInvalidationPolicy>,

    /// Access scopes
    pub access_scopes: Option<BTreeSet<CacheAccessScope>>,
}

impl CacheControl {
    pub(crate) fn merge(&mut self, mut other: CacheControl) {
        *self = CacheControl {
            public: self.public && other.public,
            max_age: if self.max_age == 0 {
                other.max_age
            } else if other.max_age == 0 {
                self.max_age
            } else {
                self.max_age.min(other.max_age)
            },
            stale_while_revalidate: if self.stale_while_revalidate == 0 {
                other.stale_while_revalidate
            } else if other.stale_while_revalidate == 0 {
                self.stale_while_revalidate
            } else {
                self.stale_while_revalidate.min(other.stale_while_revalidate)
            },
            invalidation_policy: if self.invalidation_policy.is_none() {
                other.invalidation_policy
            } else if other.invalidation_policy.is_none() {
                self.invalidation_policy.take()
            } else {
                let self_policy = self.invalidation_policy.take().unwrap();
                let other_policy = other.invalidation_policy.take().unwrap();
                Some(self_policy.max(other_policy))
            },
            access_scopes: if self.access_scopes.is_none() {
                other.access_scopes
            } else if other.access_scopes.is_none() {
                self.access_scopes.take()
            } else {
                let mut self_scopes = self.access_scopes.take().unwrap();
                let other_scopes = other.access_scopes.unwrap();
                self_scopes.extend(other_scopes);
                Some(self_scopes)
            },
        };
    }
}

#[derive(Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub struct CacheInvalidation {
    pub ty: String,
    pub policy: CacheInvalidationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
/// Represents cache purge behaviour for mutations
/// The order of variants is significant, from highest to lowest specificity
pub enum CacheInvalidationPolicy {
    /// Mutations for the target type will invalidate all cache values that have the chosen identifier
    /// E.g:
    /// with a mutation policy { policy: Entity, field: id }
    /// a mutation for a Post returns a Post { id: "1234" }, all cache values that have a Post#id:1234 will be invalidated
    Entity { field: String },
    /// Mutations for the target type will invalidate all cache values that have lists of the type in them
    /// Post#List
    List,
    /// Mutations for the target type will invalidate all cache values that have Type in them
    /// Post
    Type,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum CacheAccessScope {
    ApiKey,
    Jwt { claim: String },
    Header { header: String },
    Public,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheControlError {
    #[error(transparent)]
    Parse(#[from] crate::parser::Error),
    #[error("Validation Error: {0:?}")]
    Validate(Vec<crate::ServerError>),
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CachePartialRegistry {
    pub enable_caching: bool,
    pub types: BTreeMap<String, MetaType>,
}

impl CachePartialRegistry {
    pub fn get_cache_control(&self, request: &crate::Request) -> Result<CacheControl, CacheControlError> {
        let document = engine_parser::parse_query(&request.query).map_err(CacheControlError::Parse)?;

        let registry_caching_view = Registry {
            enable_caching: self.enable_caching,
            types: self.types.clone(),
            ..Default::default()
        };

        crate::validation::check_rules(
            &registry_caching_view,
            &document,
            Some(&request.variables),
            crate::ValidationMode::Fast,
        )
        .map(|res| res.cache_control)
        .map_err(CacheControlError::Validate)
    }
}

impl<T: Borrow<Registry>> From<T> for CachePartialRegistry {
    fn from(registry: T) -> Self {
        let registry: &Registry = registry.borrow();

        let types_with_cache = registry
            .types
            .iter()
            .filter_map(|(type_name, type_value)| {
                // it is expected that the Query node is always present as it is the starting point
                // for validation visiting. check rules/visitor.rs:588
                if *type_name == Query.to_string().to_pascal_case() {
                    return Some((type_name.to_string(), type_value.clone()));
                }

                match type_value {
                    MetaType::Object(o) => {
                        if o.cache_control != Default::default() {
                            return Some((type_name.clone(), MetaType::Object(o.clone())));
                        }
                        None
                    }
                    MetaType::Interface(i) => {
                        let has_relevant_cache_control = i
                            .fields
                            .values()
                            .find(|value| value.cache_control != Default::default());

                        if has_relevant_cache_control.is_some() {
                            return Some((type_name.clone(), MetaType::Interface(i.clone())));
                        }
                        None
                    }
                    _ => None,
                }
            })
            .collect();

        Self {
            enable_caching: registry.enable_caching,
            types: types_with_cache,
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::types::OperationType::Query;

    use crate::{
        registry::{
            CachePartialRegistry, EnumType, InterfaceType, MetaField, MetaFieldType, MetaType, ObjectType, Registry,
            ScalarType,
        },
        validation::check_rules,
        CacheControl, ValidationMode,
    };

    #[test]
    fn test_caching_view() {
        // prepare
        let meta_object = MetaType::Object(ObjectType::new("non-cached-object", []));
        let meta_cached_object =
            MetaType::Object(ObjectType::new("cached-object", []).with_cache_control(CacheControl {
                max_age: 60,
                ..Default::default()
            }));

        let meta_interface = MetaType::Interface(InterfaceType::new("non-cached-interface", []));
        let meta_cached_interface = MetaType::Interface(InterfaceType::new(
            "cached-interface",
            [
                MetaField::new("cached_field", "String!").with_cache_control(CacheControl {
                    max_age: 10,
                    ..Default::default()
                }),
            ],
        ));
        let meta_scalar = MetaType::Scalar(ScalarType {
            name: "scalar".to_string(),
            description: None,
            is_valid: None,
            visible: None,
            specified_by_url: None,
            parser: Default::default(),
        });
        let meta_enum = MetaType::Enum(EnumType {
            name: "enum".to_string(),
            description: None,
            enum_values: Default::default(),
            visible: None,
            rust_typename: String::new(),
        });

        let mut registry = Registry::new();
        registry.enable_caching = true;
        registry.insert_type(meta_object);
        registry.insert_type(meta_cached_object);
        registry.insert_type(meta_interface);
        registry.insert_type(meta_cached_interface);
        registry.insert_type(meta_scalar);
        registry.insert_type(meta_enum);

        // act
        let caching_config: CachePartialRegistry = registry.into();

        // assert
        assert!(caching_config.enable_caching);
        assert_eq!(3, caching_config.types.keys().len());
        assert!(caching_config
            .types
            .keys()
            .all(|k| k.starts_with("cached") || k.to_lowercase() == Query.to_string().to_lowercase()))
    }

    #[test]
    // the goal for this test is to make sure that the partial registry given by as_caching_view()
    // works as intended with queries that uses types not in it
    // 1) query without any cached type -> cache_control is default
    // 2) query with cached types -> cache_control is set as per as_caching_view()
    fn should_validate_with_cache_view() {
        // prepare
        let gql_query_non_cached = "query { test2 { id } }";
        let gql_query_cached = "query { test { id } }";
        let test = "test";

        //// registry
        let mut registry = Registry::new();
        registry.create_type(
            |_| {
                MetaType::Object(
                    ObjectType::new(test.to_string(), [MetaField::new("id", "String!")]).with_cache_control(
                        CacheControl {
                            max_age: 10,
                            ..Default::default()
                        },
                    ),
                )
            },
            test,
            test,
        );

        registry.query_root_mut().fields_mut().unwrap().insert(
            test.to_string(),
            MetaField::new(test.to_string(), MetaFieldType::from(test)),
        );

        // act
        let document = engine_parser::parse_query(gql_query_cached).expect("should properly parse");
        let cached_validation_result =
            check_rules(&registry, &document, None, ValidationMode::Fast).expect("should validate successfully");

        let document = engine_parser::parse_query(gql_query_non_cached).expect("should properly parse");
        let non_cached_validation_result =
            check_rules(&registry, &document, None, ValidationMode::Fast).expect("should validate successfully");

        // assert
        assert_eq!(
            cached_validation_result.cache_control,
            CacheControl {
                max_age: 10,
                ..Default::default()
            }
        );

        assert_eq!(non_cached_validation_result.cache_control, Default::default());
    }
}
