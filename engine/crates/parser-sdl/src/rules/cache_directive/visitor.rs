use engine::registry::CacheInvalidationPolicy;
use engine_parser::{
    types::{FieldDefinition, TypeDefinition, TypeKind},
    Positioned,
};

use crate::{
    rules::{
        cache_directive::{
            validation::{validate_directive, ValidationLevel},
            CacheDirectiveError,
        },
        visitor::{Visitor, VisitorContext},
    },
    utils::is_type_primitive,
};

pub struct CacheVisitor;

impl<'a> Visitor<'a> for CacheVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        doc: &'a Positioned<engine_parser::types::SchemaDefinition>,
    ) {
        if let Some(global_cache_directive) =
            validate_directive(ctx, doc.node.directives.iter(), doc.pos, ValidationLevel::Global)
        {
            ctx.global_cache_rules = global_cache_directive.into_global_rules(ctx);
            ctx.registry.get_mut().enable_caching = !ctx.global_cache_rules.is_empty();
        }
    }

    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        if let TypeKind::Object(object) = &type_definition.node.kind {
            if let Some(type_cache_directive) = validate_directive(
                ctx,
                type_definition.node.directives.iter(),
                type_definition.pos,
                ValidationLevel::Type,
            ) {
                ctx.registry.get_mut().enable_caching = true;

                if let Some(mutation_invalidation_policy) = type_cache_directive.mutation_invalidation_policy {
                    // ensure the referenced field exists in the type
                    match &mutation_invalidation_policy {
                        // we allow the _id_ to be missing because our @model types have it
                        CacheInvalidationPolicy::Entity { field: policy_field }
                            if policy_field != engine::names::OUTPUT_FIELD_ID =>
                        {
                            let referenced_field = object
                                .fields
                                .iter()
                                .find(|field| field.node.name.to_string() == *policy_field);

                            // doesn't exist return early with appropriate message
                            if referenced_field.is_none() {
                                let known_fields = object.fields.iter().map(|f| f.node.name.to_string()).collect();

                                ctx.report_error(
                                    vec![type_definition.pos],
                                    CacheDirectiveError::UnknownMutationInvalidationField(
                                        policy_field.to_string(),
                                        type_definition.node.name.to_string(),
                                        known_fields,
                                    )
                                    .to_string(),
                                );

                                return;
                            }

                            let referenced_field = referenced_field.unwrap();
                            // only primitives are allowed
                            // any complex type should have its own mutation invalidation
                            // lists are not supported to limit the number of distinct tags
                            if !is_type_primitive(&referenced_field.node) {
                                ctx.report_error(
                                    vec![type_definition.pos],
                                    CacheDirectiveError::UnknownMutationInvalidationFieldType(
                                        referenced_field.node.name.to_string(),
                                    )
                                    .to_string(),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            };
        }
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if validate_directive(ctx, field.node.directives.iter(), field.pos, ValidationLevel::Field).is_some() {
            ctx.registry.get_mut().enable_caching = true;
        };
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;

    use crate::rules::{
        cache_directive::visitor::CacheVisitor,
        visitor::{visit, VisitorContext},
    };

    #[rstest::rstest]
    /// Type
    // errors
    #[case::missing_max_age(r"
        type Test @cache {
            balance: Int!
        }
    ", & ["@cache error: missing mandatory argument(s) - [\"maxAge\"]"])]
    #[case::forbidden_usage_of_rules(r"
        type Test @cache(maxAge: 10, rules: []) {
            balance: Int!
        }
    ", & ["@cache error: forbidden argument(s) used - [\"rules\"]"])]
    #[case::single_directive(r"
        type Test @cache(maxAge: 10) @cache(maxAge: 10) {
            balance: Int!
        }
    ", & ["@cache error: only one directive is allowed"])]
    #[case::invalid_mutation_invalidation_variant(r"
        type Test @cache(maxAge: 10, mutationInvalidation: test) {
            balance: Int!
        }
    ", & ["@cache error: Unable to parse - [2:60] invalid value: string \"test\", expected one of entity, list, type"])]
    #[case::unknown_field_entity_mutation_invalidation(r#"
        type Test @cache(maxAge: 10, mutationInvalidation: {
            field: "random"
        }) {
            balance: Int! @cache(maxAge: 10)
            test2: Test2
        }
        type Test2 {
            id: ID!
        }
    "#, & ["@cache error: mutation invalidation uses an unknown field `random` for type `Test`. Known fields: [\"balance\", \"test2\"]"])]
    #[case::invalid_field_entity_mutation_validation(r#"
        type Test @cache(maxAge: 10, mutationInvalidation: {
            field: "test2"
        }) {
            balance: Int! @cache(maxAge: 10)
            test2: Test2
        }
        type Test2 {
            id: ID!
        }
    "#, & ["@cache error: mutation invalidation uses a field with an invalid type `test2`. Only primitives are allowed"])]
    // success
    #[case::successfull_max_age(r"
        type Test @cache(maxAge: 60) {
            balance: Int!
        }
    ", & [])]
    #[case::successfull_entity_mutation_invalidation(r#"
        type Test @cache(maxAge: 60, staleWhileRevalidate: 300, mutationInvalidation: {
            field: "balance"
        }) {
            balance: Int!
        }
    "#, & [])]
    #[case::successfull_swr(r"
        type Test @cache(maxAge: 60, staleWhileRevalidate: 300) {
            balance: Int!
        }
    ", & [])]
    fn test_type_parsing(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    #[rstest::rstest]
    /// Fields
    // errors
    #[case::missing_max_age(r"
        type Test {
            balance: Int! @cache
        }
    ", & ["@cache error: missing mandatory argument(s) - [\"maxAge\"]"])]
    #[case::forbidden_usage_of_rules(r"
        type Test {
            balance: Int! @cache(maxAge: 10, rules: [])
        }
    ", & ["@cache error: forbidden argument(s) used - [\"rules\", \"mutationInvalidation\"]"])]
    #[case::forbidden_usage_of_mutation_invalidation(r"
        type Test {
            balance: Int! @cache(maxAge: 10, mutationInvalidation: entity)
        }
    ", & ["@cache error: forbidden argument(s) used - [\"rules\", \"mutationInvalidation\"]"])]
    #[case::single_directive(r"
        type Test {
            balance: Int! @cache(maxAge: 10) @cache(maxAge: 10)
        }
    ", & ["@cache error: only one directive is allowed"])]
    // success
    #[case::successfull_max_age(r"
        type Test {
            balance: Int! @cache(maxAge: 60)
        }
    ", & [])]
    #[case::successfull_swr(r"
        type Test {
            balance: Int! @cache(maxAge: 60, staleWhileRevalidate: 300)
        }
    ", & [])]
    fn test_field_parsing(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    #[rstest::rstest]
    #[case::successfull_api_key_access_scope(r"
        type Test @cache(maxAge: 60, scopes: [apikey]) {
            balance: Int!
        }
    ", & [])]
    #[case::successfull_jwt_access_scope(r#"
        type Test @cache(maxAge: 60, scopes: [{ claim: "claim_name" }]) {
            balance: Int!
        }
    "#, & [])]
    #[case::successfull_header_access_scope(r#"
        type Test @cache(maxAge: 60, scopes: [{ header: "header_name" }]) {
            balance: Int!
        }
    "#, & [])]
    #[case::successfull_public_access_scope(r"
        type Test @cache(maxAge: 60, scopes: [public]) {
            balance: Int!
        }
    ", & [])]
    #[case::successfull_multiple_access_scopes(r#"
        type Test @cache(maxAge: 60, scopes: [apikey, { claim: "sub" }, { header: "header_name" }]) {
            balance: Int!
        }
    "#, & [])]
    fn test_access_scopes(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }
}
