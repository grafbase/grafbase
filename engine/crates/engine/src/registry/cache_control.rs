use std::{borrow::Borrow, collections::BTreeMap};

use engine_parser::types::OperationType::Query;
use engine_validation::ValidationMode;
use inflector::Inflector;

use crate::{
    registry::{MetaType, Registry},
    ServerError,
};

pub use registry_v2::cache_control::*;

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
        let document = engine_parser::parse_query(request.query()).map_err(CacheControlError::Parse)?;

        let registry_caching_view = Registry {
            enable_caching: self.enable_caching,
            types: self.types.clone(),
            ..Default::default()
        };

        engine_validation::check_rules(
            &build_caching_view(registry_caching_view),
            &document,
            Some(&request.variables),
            ValidationMode::Fast,
        )
        .map(|res| res.cache_control)
        .map_err(|errors| CacheControlError::Validate(errors.into_iter().map(ServerError::from).collect()))
    }
}

fn build_caching_view(_registry_caching_view: Registry) -> registry_v2::Registry {
    todo!("implement me")
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

#[cfg(todo)]
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
        let meta_cached_object = MetaType::Object(ObjectType::new("cached-object", []).with_cache_control(Some(
            Box::new(CacheControl {
                max_age: 60,
                ..Default::default()
            }),
        )));

        let meta_interface = MetaType::Interface(InterfaceType::new("non-cached-interface", []));
        let meta_cached_interface = MetaType::Interface(InterfaceType::new(
            "cached-interface",
            [
                MetaField::new("cached_field", "String!").with_cache_control(Some(Box::new(CacheControl {
                    max_age: 10,
                    ..Default::default()
                }))),
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
            .all(|k| k.starts_with("cached") || k.to_lowercase() == Query.to_string().to_lowercase()));
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
                    ObjectType::new(test.to_string(), [MetaField::new("id", "String!")]).with_cache_control(Some(
                        Box::new(CacheControl {
                            max_age: 10,
                            ..Default::default()
                        }),
                    )),
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
