use crate::directive_de::parse_directive;
use crate::rules::cache_directive::GlobalCacheRulesError::{
    ForbiddenRegistryType, UnknownRegistryType, UnknownRegistryTypeField,
};
use crate::rules::visitor::{RuleError, VisitorContext, MUTATION_TYPE};
use dynaql::registry::{MetaType, Registry};
use dynaql::CacheControl;
use dynaql_parser::types::{ConstDirective, FieldDefinition, TypeDefinition, TypeKind};
use dynaql_parser::{Pos, Positioned};
use dynaql_value::{ConstValue, Name};
use if_chain::if_chain;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::{directive::Directive, visitor::Visitor};

pub const CACHE_DIRECTIVE: &str = "@cache";
pub const RULES_ARGUMENT: &str = "rules";
pub const MAX_AGE_ARGUMENT: &str = "maxAge";
pub const STALE_WHILE_REVALIDATE_ARGUMENT: &str = "staleWhileRevalidate";

#[derive(Debug, thiserror::Error)]
enum CacheDirectiveError<'a> {
    #[error("@cache error: {0}")]
    GlobalRule(&'a str),
    #[error("@cache error: missing mandatory argument(s) - {0:?}")]
    MandatoryArguments(&'a [&'a str]),
    #[error("@cache error: forbidden argument(s) used - {0:?}")]
    ForbiddenArguments(&'a [&'a str]),
    #[error("@cache error: Unable to parse - {0}")]
    Parsing(RuleError),
    #[error("@cache error: only one directive is allowed")]
    Multiple,
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum GlobalCacheRulesError {
    #[error("@cache error: Global cache rule references an unknown type `{0}`.")]
    UnknownRegistryType(String),
    #[error("@cache error: Global cache rule references an unknown field `{0}` for type `{1}`. Known fields: {2:?}")]
    UnknownRegistryTypeField(String, String, Vec<String>),
    #[error("@cache error: Global cache rule references a forbidden type `{0}`.")]
    ForbiddenRegistryType(String),
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
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheDirective {
    #[serde(default, rename = "maxAge")]
    pub max_age: usize,
    #[serde(default, rename = "staleWhileRevalidate")]
    pub stale_while_revalidate: usize,
    #[serde(default)]
    pub rules: Vec<CacheRule>,

    #[serde(skip)]
    pos: Pos,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum GlobalCacheTarget<'a> {
    /// Type name
    Type(Cow<'a, str>),
    /// Type name + Field name
    Field(Cow<'a, str>, Cow<'a, str>),
}

#[derive(Debug, Default)]
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
    fn get_registry_type<'r>(ty: &str, registry: &'r mut Registry) -> Result<&'r mut MetaType, GlobalCacheRulesError> {
        if ty == MUTATION_TYPE {
            return Err(ForbiddenRegistryType(ty.to_string()));
        }

        let Some(registry_type) = registry.types.get_mut(ty) else {
            return Err(UnknownRegistryType(ty.to_string()));
        };

        Ok(registry_type)
    }

    pub fn apply(&self, registry: &mut Registry) -> Result<(), Vec<GlobalCacheRulesError>> {
        let mut errors = Vec::with_capacity(self.0.len());

        for (target_type, global_cache_control) in &self.0 {
            match target_type {
                GlobalCacheTarget::Type(ty) => {
                    let registry_type = Self::get_registry_type(ty.as_ref(), registry);
                    match registry_type {
                        Ok(registry_type) => {
                            if_chain! {
                                if let MetaType::Object { cache_control, .. } = registry_type;
                                // (!= 0) means caching was defined in a different level
                                // global works as default so we skip
                                if cache_control.max_age == 0;
                                then {
                                    *cache_control = *global_cache_control;
                                }
                            }
                        }
                        Err(err) => errors.push(err),
                    }
                }
                GlobalCacheTarget::Field(ty, field) => {
                    let registry_type = Self::get_registry_type(ty.as_ref(), registry);

                    match registry_type {
                        Ok(registry_type) => {
                            if let Some(registry_type_field) = registry_type.field_by_name_mut(field.as_ref()) {
                                // (!= 0) means caching was defined in a different level
                                // global works as default so we skip
                                if registry_type_field.cache_control.max_age == 0 {
                                    registry_type_field.cache_control = *global_cache_control;
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

impl CacheDirective {
    pub fn parse(directives: &[Positioned<ConstDirective>]) -> CacheControl {
        directives
            .iter()
            .find(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
            .and_then(|directive| parse_directive::<CacheDirective>(&directive.node, &HashMap::default()).ok())
            .unwrap_or_default()
            .into()
    }

    fn rules_checked(&self, ctx: &mut VisitorContext<'_>) -> GlobalCacheRules<'static> {
        let mut visited_rules = GlobalCacheRules::default();
        let mut cache_ty_checked = |key: GlobalCacheTarget<'static>, rule: &CacheRule| {
            if visited_rules.contains_key(&key) {
                ctx.report_error(
                    vec![self.pos],
                    CacheDirectiveError::GlobalRule(&format!("duplicate cache target: {key:?}")).to_string(),
                );

                return;
            }

            visited_rules.insert(
                key,
                CacheControl {
                    public: true,
                    max_age: rule.max_age,
                    stale_while_revalidate: rule.stale_while_revalidate,
                },
            );
        };

        for rule in &self.rules {
            match &rule.types {
                CacheRuleTargetType::Simple(ty) => {
                    cache_ty_checked(GlobalCacheTarget::Type(Cow::Owned(ty.clone())), rule);
                }
                CacheRuleTargetType::List(ty_list) => {
                    ty_list
                        .iter()
                        .for_each(|ty| cache_ty_checked(GlobalCacheTarget::Type(Cow::Owned(ty.clone())), rule));
                }
                CacheRuleTargetType::Structured(structured_ty_list) => {
                    structured_ty_list
                        .iter()
                        .flat_map(|structured| {
                            if structured.fields.is_empty() {
                                return vec![GlobalCacheTarget::Type(Cow::Owned(structured.name.clone()))];
                            }

                            structured
                                .fields
                                .iter()
                                .map(|field| {
                                    GlobalCacheTarget::Field(
                                        Cow::Owned(structured.name.clone()),
                                        Cow::Owned(field.clone()),
                                    )
                                })
                                .collect()
                        })
                        .for_each(|target| cache_ty_checked(target, rule));
                }
            }
        }

        visited_rules
    }
}

impl From<CacheDirective> for CacheControl {
    fn from(value: CacheDirective) -> Self {
        CacheControl {
            public: true,
            max_age: value.max_age,
            stale_while_revalidate: value.stale_while_revalidate,
        }
    }
}

const CACHE_DIRECTIVE_NAME: &str = "cache";

impl Directive for CacheDirective {
    fn definition() -> String {
        format!(
            r#"
        directive {CACHE_DIRECTIVE}(
          "How long query results should be cached, in seconds"
          {MAX_AGE_ARGUMENT}: Int!
          "How long, in seconds, stale cached results should be served while data is refreshed"
          {STALE_WHILE_REVALIDATE_ARGUMENT}: Int
          "Global caching rules"
          {RULES_ARGUMENT}: [CacheRule!]
        ) on SCHEMA | OBJECT | FIELD_DEFINITION

        input CacheRule {{
            {MAX_AGE_ARGUMENT}: Int!
            {STALE_WHILE_REVALIDATE_ARGUMENT}: Int
            "Targets where cache settings will apply"
            types: CacheRuleType!
        }}

        input CacheRuleType @oneOf {{
            type: String!
            types: [String!]!
            types_fields: [StructuredCacheRuleType!]!
        }}

        input StructuredCacheRuleType {{
          name: String!
          fields: [String!]
        }}
        "#
        )
    }
}

enum ArgumentValidation {
    Mandatory,
    Forbidden,
}

fn validate_directive_arguments(
    ctx: &mut VisitorContext<'_>,
    pos: &Pos,
    directive_arguments: &[(Positioned<Name>, Positioned<ConstValue>)],
    arguments: &[&str],
    validation: ArgumentValidation,
) {
    let has_arguments = directive_arguments
        .iter()
        .any(|(name, _)| arguments.contains(&name.node.as_str()));

    match validation {
        ArgumentValidation::Mandatory => {
            if !has_arguments {
                ctx.report_error(
                    vec![*pos],
                    CacheDirectiveError::MandatoryArguments(arguments).to_string(),
                );
            }
        }
        ArgumentValidation::Forbidden => {
            if has_arguments {
                ctx.report_error(
                    vec![*pos],
                    CacheDirectiveError::ForbiddenArguments(arguments).to_string(),
                );
            }
        }
    }
}

fn validate_global_cache_directive<'a>(ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
    // check that maxAge and staleWhileRevalidate are not used at the global level
    validate_directive_arguments(
        ctx,
        &directive.pos,
        &directive.node.arguments,
        &[MAX_AGE_ARGUMENT, STALE_WHILE_REVALIDATE_ARGUMENT],
        ArgumentValidation::Forbidden,
    );

    // check that rules is set at the global level
    validate_directive_arguments(
        ctx,
        &directive.pos,
        &directive.node.arguments,
        &[RULES_ARGUMENT],
        ArgumentValidation::Mandatory,
    );
}

fn validate_inline_cache_directive<'a>(ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
    // check that the rules argument is only used at the global level
    validate_directive_arguments(
        ctx,
        &directive.pos,
        &directive.node.arguments,
        &[RULES_ARGUMENT],
        ArgumentValidation::Forbidden,
    );

    // check that maxAge is defined
    validate_directive_arguments(
        ctx,
        &directive.pos,
        &directive.node.arguments,
        &[MAX_AGE_ARGUMENT],
        ArgumentValidation::Mandatory,
    );
}

fn validate_directive<'a>(
    ctx: &mut VisitorContext<'a>,
    directives: impl Iterator<Item = &'a Positioned<ConstDirective>>,
    pos: Pos,
    is_global: bool,
) -> Option<CacheDirective> {
    let directives: Vec<_> = directives
        .filter(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
        .collect();

    // only one @cache directive is allowed
    if directives.len() > 1 {
        ctx.report_error(vec![pos], CacheDirectiveError::Multiple.to_string());
    }

    directives.first().and_then(|pos_const_directive| {
        if is_global {
            validate_global_cache_directive(ctx, pos_const_directive);
        } else {
            validate_inline_cache_directive(ctx, pos_const_directive);
        }

        match parse_directive::<CacheDirective>(&pos_const_directive.node, &HashMap::default()) {
            Ok(mut cache_directive) => {
                cache_directive.pos = pos_const_directive.pos;
                Some(cache_directive)
            }
            Err(err) => {
                ctx.report_error(
                    vec![pos_const_directive.pos],
                    CacheDirectiveError::Parsing(err).to_string(),
                );
                None
            }
        }
    })
}

pub struct CacheVisitor;

impl<'a> Visitor<'a> for CacheVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        doc: &'a Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        if let Some(global_cache_directive) = validate_directive(ctx, doc.node.directives.iter(), doc.pos, true) {
            ctx.global_cache_rules = global_cache_directive.rules_checked(ctx);
            ctx.registry.get_mut().enable_caching = !ctx.global_cache_rules.is_empty();
        }
    }

    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        if let TypeKind::Object(_) = &type_definition.node.kind {
            if validate_directive(ctx, type_definition.node.directives.iter(), type_definition.pos, false).is_some() {
                ctx.registry.get_mut().enable_caching = true;
            };
        }
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if validate_directive(ctx, field.node.directives.iter(), field.pos, false).is_some() {
            ctx.registry.get_mut().enable_caching = true;
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::cache_directive::CacheVisitor;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    #[rstest::rstest]
    /// Model
    // errors
    #[case(r#"
        type Test @cache {
            balance: Int!
        }
    "#, & ["@cache error: missing mandatory argument(s) - [\"maxAge\"]"])]
    #[case(r#"
        type Test @cache(maxAge: 10, rules: []) {
            balance: Int!
        }
    "#, & ["@cache error: forbidden argument(s) used - [\"rules\"]"])]
    #[case(r#"
        type Test @cache(maxAge: 10) @cache(maxAge: 10) {
            balance: Int!
        }
    "#, & ["@cache error: only one directive is allowed"])]
    // success
    #[case(r#"
        type Test @cache(maxAge: 60) {
            balance: Int!
        }
    "#, & [])]
    #[case(r#"
        type Test @cache(maxAge: 60, staleWhileRevalidate: 300) {
            balance: Int!
        }
    "#, & [])]
    /// Fields
    // errors
    #[case(r#"
        type Test {
            balance: Int! @cache
        }
    "#, & ["@cache error: missing mandatory argument(s) - [\"maxAge\"]"])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 10, rules: [])
        }
    "#, & ["@cache error: forbidden argument(s) used - [\"rules\"]"])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 10) @cache(maxAge: 10)
        }
    "#, & ["@cache error: only one directive is allowed"])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 60)
        }
    "#, & [])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 60, staleWhileRevalidate: 300)
        }
    "#, & [])]
    fn test_inline_parsing(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    #[rstest::rstest]
    // errors
    #[case(r#"
        extend schema @cache(maxAge: 60, staleWhileRevalidate: 300, rules: [])
    "#, & ["@cache error: forbidden argument(s) used - [\"maxAge\", \"staleWhileRevalidate\"]"])]
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10
        }])
    "#, & ["@cache error: Unable to parse - [2:37] missing field `types`"])]
    // success
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: "Simple"
        }])
    "#, & [])]
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: ["List", "Of", "Strings"]
        }])
    "#, & [])]
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName"
            }]
        }])
    "#, & [])]
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: ["field1", "field2"]
            }]
        }])
    "#, & [])]
    #[case(r#"
        extend schema @cache(rules: [{
            maxAge: 10,
            types: [{
                name: "TypeName",
                fields: []
            }]
        }])
    "#, & [])]
    fn test_global_parsing(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }
}
