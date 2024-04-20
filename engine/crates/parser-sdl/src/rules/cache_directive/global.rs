use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    ops::{Deref, DerefMut},
};

use engine::{
    indexmap::IndexMap,
    registry::{CacheInvalidationPolicy, MetaField, MetaType, Registry, TypeReference},
    CacheControl,
};
use if_chain::if_chain;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::rules::{
    cache_directive::{
        de_mutation_invalidation,
        global::GlobalCacheRulesError::{ForbiddenRegistryType, UnknownRegistryType, UnknownRegistryTypeField},
        CacheAccessScopeWrapper,
    },
    visitor::MUTATION_TYPE,
};

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum GlobalCacheRulesError {
    #[error("@cache error: Global cache rule references an unknown type `{0}`.")]
    UnknownRegistryType(String),
    #[error("@cache error: Global cache rule references an unknown field `{0}` for type `{1}`. Known fields: {2:?}")]
    UnknownRegistryTypeField(String, String, Vec<String>),
    #[error("@cache error: Global cache rule references a forbidden type `{0}`.")]
    ForbiddenRegistryType(String),
    #[error("@cache error: mutation invalidation uses an unknown field `{0}` for type `{1}`. Known fields: {2:?}")]
    UnknownMutationInvalidationField(String, String, Vec<String>),
    #[error(
        "@cache error: mutation invalidation uses a field with an invalid type `{0}`. Only primitives are allowed"
    )]
    UnknownMutationInvalidationFieldType(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct StructuredCacheRuleTargetType {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum CacheRuleTargetType {
    Simple(String),
    List(Vec<String>),
    Structured(Vec<StructuredCacheRuleTargetType>),
}

#[derive(Debug, serde::Deserialize)]
pub struct CacheRule {
    #[serde(rename = "maxAge")]
    pub max_age: usize,
    #[serde(default, rename = "staleWhileRevalidate")]
    pub stale_while_revalidate: usize,
    pub types: CacheRuleTargetType,
    #[serde(
        default,
        rename = "mutationInvalidation",
        deserialize_with = "de_mutation_invalidation"
    )]
    pub mutation_invalidation_policy: Option<CacheInvalidationPolicy>,
    #[serde(default, rename = "scopes")]
    pub access_scopes: Option<BTreeSet<CacheAccessScopeWrapper>>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum GlobalCacheTarget<'a> {
    /// Type name
    Type(Cow<'a, str>),
    /// Type name + Field name
    Field(Cow<'a, str>, Cow<'a, str>),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GlobalCacheRules<'a>(HashMap<GlobalCacheTarget<'a>, CacheControl>);

impl<'a> Deref for GlobalCacheRules<'a> {
    type Target = HashMap<GlobalCacheTarget<'a>, CacheControl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for GlobalCacheRules<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> GlobalCacheRules<'a> {
    pub fn get_registry_type<'r>(
        ty: &str,
        registry: &'r mut Registry,
    ) -> Result<&'r mut MetaType, GlobalCacheRulesError> {
        if ty == MUTATION_TYPE {
            return Err(ForbiddenRegistryType(ty.to_string()));
        }

        let Some(registry_type) = registry.types.get_mut(ty) else {
            return Err(UnknownRegistryType(ty.to_string()));
        };

        Ok(registry_type)
    }

    pub fn apply(self, registry: &mut Registry) -> Result<(), Vec<GlobalCacheRulesError>> {
        let mut errors = Vec::with_capacity(self.0.len());
        for (target_type, global_cache_control) in self.0 {
            match target_type {
                GlobalCacheTarget::Type(ty) => {
                    match Self::get_registry_type(ty.as_ref(), registry) {
                        Ok(registry_type) => {
                            let caching_interest = match registry_type {
                                MetaType::Object(object) => Some((&mut object.cache_control, &mut object.fields)),
                                MetaType::Interface(interface) => {
                                    Some((&mut interface.cache_control, &mut interface.fields))
                                }
                                _ => None,
                            };

                            if_chain! {
                                if let Some((cache_control, fields)) = caching_interest;
                                // (!= 0) means caching was defined in a different level
                                // global works as default so we skip
                                if cache_control.max_age == 0;
                                then {
                                    *cache_control = global_cache_control;

                                    // check the mutation invalidation
                                    if_chain! {
                                        if let Some(mutation_invalidation_policy) = &cache_control.invalidation_policy;
                                        if let Err(err) = validate_mutation_invalidation(
                                            ty.as_ref(),
                                            fields,
                                            mutation_invalidation_policy,
                                        );
                                        then {
                                            errors.push(err)
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => errors.push(err),
                    }
                }
                GlobalCacheTarget::Field(ty, field) => {
                    match Self::get_registry_type(ty.as_ref(), registry) {
                        Ok(registry_type) => {
                            let type_fields = registry_type.fields().cloned();
                            let registry_type_field = registry_type.field_by_name_mut(field.as_ref());

                            if let Some(registry_type_field) = registry_type_field {
                                // (!= 0) means caching was defined in a different level
                                // global works as default so we skip
                                if registry_type_field.cache_control.max_age == 0 {
                                    registry_type_field.cache_control = global_cache_control;

                                    // check the mutation invalidation
                                    if_chain! {
                                        if let Some(mutation_invalidation_policy) = &registry_type_field.cache_control.invalidation_policy;
                                        if let Err(err) = validate_mutation_invalidation(
                                            ty.as_ref(),
                                            // safe, field found means there are fields
                                            &type_fields.unwrap(),
                                            mutation_invalidation_policy,
                                        );
                                        then {
                                            errors.push(err)
                                        }
                                    }
                                }
                            } else {
                                let known_fields = registry_type
                                    .fields()
                                    .map(|fields| fields.keys().map(|k| k.to_string()).collect_vec())
                                    .unwrap_or_default();

                                errors.push(UnknownRegistryTypeField(
                                    field.to_string(),
                                    ty.to_string(),
                                    known_fields,
                                ));
                            }
                        }
                        Err(err) => errors.push(err),
                    }
                }
            }
        }

        if errors.is_empty() {
            return Ok(());
        }

        Err(errors)
    }
}

fn validate_mutation_invalidation(
    ty: &str,
    fields: &IndexMap<String, MetaField>,
    mutation_invalidation_policy: &CacheInvalidationPolicy,
) -> Result<(), GlobalCacheRulesError> {
    match mutation_invalidation_policy {
        // ensure the referenced field exists in the type
        // we allow the _id_ to be missing because our @model types have it
        CacheInvalidationPolicy::Entity { field: policy_field } if policy_field != engine::names::OUTPUT_FIELD_ID => {
            let referenced_field = fields.iter().find(|(field_name, _)| *field_name == policy_field);

            // doesn't exist return early with appropriate message
            if referenced_field.is_none() {
                let known_fields = fields.iter().map(|(name, _)| name.to_string()).collect();
                return Err(GlobalCacheRulesError::UnknownMutationInvalidationField(
                    policy_field.to_string(),
                    ty.to_string(),
                    known_fields,
                ));
            }

            let (field_name, meta_field) = referenced_field.unwrap();

            // only primitives are allowed
            // any complex type should have its own mutation invalidation
            // lists are not supported to limit the number of distinct tags
            if !&meta_field.ty.named_type().is_primitive_type() {
                return Err(GlobalCacheRulesError::UnknownMutationInvalidationFieldType(
                    field_name.to_string(),
                ));
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::HashMap};

    use engine::{
        registry::{CacheInvalidationPolicy, MetaField},
        CacheControl,
    };
    use engine_parser::parse_schema;

    use crate::{
        rules::{
            cache_directive::{
                global::{
                    GlobalCacheRulesError,
                    GlobalCacheRulesError::{ForbiddenRegistryType, UnknownRegistryType, UnknownRegistryTypeField},
                    GlobalCacheTarget,
                },
                visitor::CacheVisitor,
            },
            visitor::{visit, VisitorContext, MUTATION_TYPE},
        },
        to_parse_result_with_variables, ParseResult,
    };

    #[rstest::rstest]
    // errors
    #[case::forbidden_usage_max_age(r"
        extend schema @cache(maxAge: 60, staleWhileRevalidate: 300, rules: [])
    ", & ["@cache error: forbidden argument(s) used - [\"maxAge\", \"staleWhileRevalidate\", \"mutationInvalidation\"]"])]
    #[case::forbidden_usage_mutation_invalidation(r"
        extend schema @cache(maxAge: 60, mutationInvalidation: type, rules: [])
    ", & ["@cache error: forbidden argument(s) used - [\"maxAge\", \"staleWhileRevalidate\", \"mutationInvalidation\"]"])]
    #[case::missing_types_field(r"
        extend schema @cache(rules: [{
            maxAge: 10
        }])
    ", & ["@cache error: Unable to parse - [2:37] missing field `types`"])]
    #[case::forbidden_and_invalid_mutation_invalidation(r"
        extend schema @cache(maxAge: 60, mutationInvalidation: test, rules: [])
    ", & [
    "@cache error: forbidden argument(s) used - [\"maxAge\", \"staleWhileRevalidate\", \"mutationInvalidation\"]",
    "@cache error: Unable to parse - [2:64] invalid value: string \"test\", expected one of entity, list, type",
    ])]
    // success
    #[case::successfull_simple_types_rule(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: "Simple"
        }])
    "#, & [])]
    #[case::successfull_list_types_rule(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["List", "Of", "Strings"]
        }])
    "#, & [])]
    #[case::successfull_simple_structured_type_rule(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName"
            }]
        }])
    "#, & [])]
    #[case::successfull_structured_type_rule_with_fields(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: ["field1", "field2"]
            }]
        }])
    "#, & [])]
    #[case::successfull_structured_type_rule_with_empty_fields(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: []
            }]
        }])
    "#, & [])]
    #[case::successfull_mutation_invalidation(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: []
            }],
            mutationInvalidation: type
        }])
    "#, & [])]
    #[case::successfull_mutation_invalidation_custom_entity(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: []
            }],
            mutationInvalidation: {
                field: "id"
            }
        }])
    "#, & [])]
    #[case::successful_api_key_access_scope(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["TypeName"],
            scopes: [apikey]
        }])
    "#, &[])]
    #[case::successful_jwt_access_scope(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["TypeName"],
            scopes: [{
                claim: "sub"
            }]
        }])
    "#, &[])]
    #[case::successful_header_access_scope(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["TypeName"],
            scopes: [{
                header: "hey"
            }]
        }])
    "#, &[])]
    #[case::successful_public_access_scope(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["TypeName"],
            scopes: [public]
        }])
    "#, &[])]
    #[case::successful_multiple_access_scopes(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["TypeName"],
            scopes: [apikey, { claim: "sub" }, { header: "header_name" }]
        }])
    "#, &[])]
    fn test_global_parsing(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    #[test]
    #[ignore] // FIXME: Update to not use `@model`.
    #[allow(clippy::panic)]
    fn should_apply_global_cache_rules_and_check_inline_precedence() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        extend schema @cache(rules: [
            {maxAge: 10, types: "User", mutationInvalidation: type},
            {maxAge: 5, types: [{ name: "Post", fields: ["contents"]}]}
        ])

        type User @model @cache(maxAge: 60, mutationInvalidation: entity) {
            name: String!
            email: String!
        }

        type Post @model @cache(maxAge: 20) {
            author: User!
            contents: String! @cache(maxAge: 10)
        }
    "#;

        let mut result = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

        // apply caching controls
        if let Err(global_cache_rules_result) = result.global_cache_rules.apply(&mut result.registry) {
            panic!("global cache rules apply must succeed - {global_cache_rules_result:?}");
        };

        let user = result
            .registry
            .types
            .get("User")
            .unwrap()
            .object()
            .expect("should be an object");

        let post_type = result
            .registry
            .types
            .get("Post")
            .unwrap()
            .object()
            .expect("should be an object");

        let MetaField {
            cache_control: post_contents_cache_control,
            ..
        } = post_type.fields.get("contents").unwrap();

        assert_eq!(
            user.cache_control,
            CacheControl {
                public: false,
                max_age: 60,
                stale_while_revalidate: 0,
                invalidation_policy: Some(CacheInvalidationPolicy::Entity {
                    field: "id".to_string()
                }),
                access_scopes: None,
            }
        );

        assert_eq!(
            post_type.cache_control,
            CacheControl {
                public: false,
                max_age: 20,
                stale_while_revalidate: 0,
                invalidation_policy: None,
                access_scopes: None,
            }
        );

        assert_eq!(
            post_contents_cache_control,
            &CacheControl {
                public: false,
                max_age: 10,
                stale_while_revalidate: 0,
                invalidation_policy: None,
                access_scopes: None,
            }
        );
    }

    #[test]
    #[ignore] // FIXME: Update to not use `@model`.
    fn should_fail_global_cache_rules_apply_due_to_unknown_type_and_field() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        extend schema @cache(rules: [
            {maxAge: 10, types: "User"},
            {maxAge: 5, types: [{ name: "Post", fields: ["contents"]}]}
        ])

        type User @model @cache(maxAge: 60) {
            name: String!
            email: String!
        }

        type Post @model @cache(maxAge: 20) {
            author: User!
            contents: String! @cache(maxAge: 10)
        }
    "#;

        let ParseResult {
            mut registry,
            mut global_cache_rules,
            ..
        } = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");
        global_cache_rules.insert(
            GlobalCacheTarget::Type(Cow::Owned("UnknownType".to_string())),
            CacheControl::default(),
        );
        global_cache_rules.insert(
            GlobalCacheTarget::Field(Cow::Owned("Post".to_string()), Cow::Owned("unknownField".to_string())),
            CacheControl::default(),
        );
        let known_post_fields = registry
            .types
            .get("Post")
            .unwrap()
            .fields()
            .unwrap()
            .keys()
            .map(|s| s.to_string())
            .collect();

        let apply_result = global_cache_rules.apply(&mut registry);
        assert!(apply_result.is_err());

        let err = apply_result.err().unwrap();
        assert_eq!(err.len(), 2);
        assert!(err.contains(&UnknownRegistryType("UnknownType".to_string())));
        assert!(err.contains(&UnknownRegistryTypeField(
            "unknownField".to_string(),
            "Post".to_string(),
            known_post_fields,
        )));
    }

    #[test]
    #[ignore] // FIXME: Update to not use `@model`.
    fn should_fail_global_cache_rules_apply_due_to_mutation_rule() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        extend schema @cache(rules: [
            {maxAge: 10, types: "User"},
            {maxAge: 5, types: [{ name: "Post", fields: ["contents"]}]}
        ])

        type User @model @cache(maxAge: 60) {
            name: String!
            email: String!
        }

        type Post @model @cache(maxAge: 20) {
            author: User!
            contents: String! @cache(maxAge: 10)
        }
    "#;

        let ParseResult {
            mut registry,
            mut global_cache_rules,
            ..
        } = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");
        global_cache_rules.insert(
            GlobalCacheTarget::Type(Cow::Owned(MUTATION_TYPE.to_string())),
            CacheControl::default(),
        );

        let apply_result = global_cache_rules.apply(&mut registry);
        assert!(apply_result.is_err());

        let err = apply_result.err().unwrap();
        assert_eq!(err.len(), 1);
        assert!(err.contains(&ForbiddenRegistryType(MUTATION_TYPE.to_string())));
    }

    #[test]
    #[ignore] // FIXME: Update to not use `@model`.
    fn should_fail_global_cache_rules_apply_due_to_invalid_mutation_invalidation() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        extend schema @cache(rules: [
            {maxAge: 10, types: "User", mutationInvalidation: {
                field: "random"
            }},
            {maxAge: 5, types: [{ name: "Post", fields: ["contents"]}]}
        ])

        type User @model {
            name: String!
            email: String!
        }

        type Post @model {
            author: User!
            contents: String!
        }
    "#;

        let ParseResult {
            mut registry,
            global_cache_rules,
            ..
        } = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

        let apply_result = global_cache_rules.apply(&mut registry);
        assert!(apply_result.is_err());

        let err = apply_result.err().unwrap();
        let known_user_fields = registry
            .types
            .get("User")
            .unwrap()
            .fields()
            .unwrap()
            .keys()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(err.len(), 1);
        assert!(err.contains(&GlobalCacheRulesError::UnknownMutationInvalidationField(
            "random".to_string(),
            "User".to_string(),
            known_user_fields,
        )));
    }

    #[test]
    #[ignore] // FIXME: Update to not use `@model`.
    fn should_fail_global_cache_rules_apply_due_to_invalid_mutation_invalidation_2() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        extend schema @cache(rules: [
            {maxAge: 10, types: "User"},
            {maxAge: 5, types: [{ name: "Post", fields: ["contents"]}], mutationInvalidation: {
                field: "author"
            }}
        ])

        type User @model {
            name: String!
            email: String!
        }

        type Post @model {
            author: User!
            contents: String!
        }
    "#;

        let ParseResult {
            mut registry,
            global_cache_rules,
            ..
        } = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

        let apply_result = global_cache_rules.apply(&mut registry);
        assert!(apply_result.is_err());

        let err = apply_result.err().unwrap();
        assert_eq!(err.len(), 1);
        assert!(
            err.contains(&GlobalCacheRulesError::UnknownMutationInvalidationFieldType(
                "author".to_string(),
            ))
        );
    }

    #[test]
    #[allow(clippy::panic)]
    fn should_apply_global_cache_rules_on_resolver_types() {
        let variables = HashMap::new();
        const SCHEMA: &str = r#"
        type Query {
            slow(seconds: String!): Post! @resolver(name: "slow")
        }

        extend schema @cache(rules: [{ maxAge: 60, staleWhileRevalidate: 10, types: "Post" }])

        type Post {
            seconds: String!
            hello: String!
        }
    "#;

        let mut result = to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

        // apply caching controls
        if let Err(global_cache_rules_result) = result.global_cache_rules.apply(&mut result.registry) {
            panic!("global cache rules apply must succeed - {global_cache_rules_result:?}");
        };

        let post_type = result
            .registry
            .types
            .get("Post")
            .unwrap()
            .object()
            .expect("should be an object");

        assert_eq!(
            post_type.cache_control,
            CacheControl {
                public: false,
                max_age: 60,
                stale_while_revalidate: 10,
                invalidation_policy: None,
                access_scopes: None,
            }
        );
    }
}
