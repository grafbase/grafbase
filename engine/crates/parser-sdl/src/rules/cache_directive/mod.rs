use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fmt::Formatter,
};

use engine::{
    registry::{CacheAccessScope, CacheInvalidationPolicy},
    CacheControl,
};
use engine_parser::{types::ConstDirective, Pos, Positioned};
use serde::{
    de::{value::MapAccessDeserializer, Error, MapAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    directive_de::parse_directive,
    rules::{
        cache_directive::global::{CacheRule, CacheRuleTargetType, GlobalCacheRules, GlobalCacheTarget},
        directive::Directive,
        visitor::{RuleError, VisitorContext},
    },
};

const CACHE_DIRECTIVE_NAME: &str = "cache";

pub const RULES_ARGUMENT: &str = "rules";
pub const MAX_AGE_ARGUMENT: &str = "maxAge";
pub const STALE_WHILE_REVALIDATE_ARGUMENT: &str = "staleWhileRevalidate";
pub const MUTATION_INVALIDATION_POLICY_ARGUMENT: &str = "mutationInvalidation";

pub mod global;
mod validation;
pub mod visitor;

#[derive(Debug, thiserror::Error)]
pub enum CacheDirectiveError<'a> {
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
    #[error("@cache error: mutation invalidation uses an unknown field `{0}` for type `{1}`. Known fields: {2:?}")]
    UnknownMutationInvalidationField(String, String, Vec<String>),
    #[error(
        "@cache error: mutation invalidation uses a field with an invalid type `{0}`. Only primitives are allowed"
    )]
    UnknownMutationInvalidationFieldType(String),
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
    #[serde(
        default,
        rename = "mutationInvalidation",
        deserialize_with = "de_mutation_invalidation"
    )]
    pub mutation_invalidation_policy: Option<CacheInvalidationPolicy>,
    #[serde(default, rename = "scopes")]
    pub access_scopes: Option<BTreeSet<CacheAccessScopeWrapper>>,

    #[serde(skip)]
    pos: Pos,
}

struct MutationInvalidationVisitor;
impl<'de> Visitor<'de> for MutationInvalidationVisitor {
    type Value = Option<CacheInvalidationPolicy>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an unquoted str, e.g: type")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "entity" => Ok(Some(CacheInvalidationPolicy::Entity {
                field: engine::names::OUTPUT_FIELD_ID.to_string(),
            })),
            "list" => Ok(Some(CacheInvalidationPolicy::List)),
            "type" => Ok(Some(CacheInvalidationPolicy::Type)),
            unknown => Err(Error::invalid_value(
                Unexpected::Str(unknown),
                &"one of entity, list, type",
            )),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    fn visit_map<D>(self, map: D) -> Result<Self::Value, D::Error>
    where
        D: MapAccess<'de>,
    {
        #[derive(Debug, serde::Deserialize)]
        struct CustomEntity {
            field: String,
        }
        let CustomEntity { field } = CustomEntity::deserialize(MapAccessDeserializer::new(map))?;
        Ok(Some(CacheInvalidationPolicy::Entity { field }))
    }
}

fn de_mutation_invalidation<'de, D>(deserializer: D) -> Result<Option<CacheInvalidationPolicy>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(MutationInvalidationVisitor)
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct CacheAccessScopeWrapper(CacheAccessScope);
impl<'de> Deserialize<'de> for CacheAccessScopeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CacheAccessScopeVisitor)
    }
}

struct CacheAccessScopeVisitor;
impl<'de> Visitor<'de> for CacheAccessScopeVisitor {
    type Value = CacheAccessScopeWrapper;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a valid access scope")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "apikey" => Ok(CacheAccessScopeWrapper(CacheAccessScope::ApiKey)),
            "public" => Ok(CacheAccessScopeWrapper(CacheAccessScope::Public)),
            unknown => Err(Error::invalid_value(Unexpected::Str(unknown), &"one of apikey, public")),
        }
    }

    fn visit_map<D>(self, mut map: D) -> Result<Self::Value, D::Error>
    where
        D: MapAccess<'de>,
    {
        if let Some((key, value)) = map.next_entry::<String, String>()? {
            match key.as_str() {
                "header" => Ok(CacheAccessScopeWrapper(CacheAccessScope::Header { header: value })),
                "claim" => Ok(CacheAccessScopeWrapper(CacheAccessScope::Jwt { claim: value })),
                x => Err(Error::unknown_field(x, &["header", "claim"])),
            }
        } else {
            Err(Error::invalid_value(Unexpected::Map, &self))
        }
    }
}

impl CacheDirective {
    pub fn parse(directives: &[Positioned<ConstDirective>]) -> Option<Box<CacheControl>> {
        directives
            .iter()
            .find(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
            .and_then(|directive| parse_directive::<CacheDirective>(&directive.node, &HashMap::default()).ok())
            .map(Into::into)
            .map(Box::new)
    }

    fn into_global_rules(self, ctx: &mut VisitorContext<'_>) -> GlobalCacheRules<'static> {
        let mut visited_rules = GlobalCacheRules::default();

        let mut validate_insert =
            |key: GlobalCacheTarget<'static>,
             max_age: usize,
             stale_while_revalidate: usize,
             mutation_invalidation_policy: Option<CacheInvalidationPolicy>,
             access_scopes: Option<BTreeSet<CacheAccessScope>>| {
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
                        public: false,
                        max_age,
                        stale_while_revalidate,
                        invalidation_policy: mutation_invalidation_policy,
                        access_scopes,
                    },
                );
            };

        for rule in self.rules {
            match rule.types {
                CacheRuleTargetType::Simple(ty) => {
                    validate_insert(
                        GlobalCacheTarget::Type(Cow::Owned(ty)),
                        rule.max_age,
                        rule.stale_while_revalidate,
                        rule.mutation_invalidation_policy,
                        rule.access_scopes
                            .map(|scopes| scopes.into_iter().map(|scope| scope.0).collect()),
                    );
                }
                CacheRuleTargetType::List(ty_list) => {
                    for ty in ty_list {
                        validate_insert(
                            GlobalCacheTarget::Type(Cow::Owned(ty)),
                            rule.max_age,
                            rule.stale_while_revalidate,
                            rule.mutation_invalidation_policy.clone(),
                            rule.access_scopes
                                .clone()
                                .map(|scopes| scopes.into_iter().map(|scope| scope.0).collect()),
                        );
                    }
                }
                CacheRuleTargetType::Structured(structured_ty_list) => {
                    structured_ty_list
                        .into_iter()
                        .flat_map(|structured| {
                            if structured.fields.is_empty() {
                                return vec![GlobalCacheTarget::Type(Cow::Owned(structured.name))];
                            }

                            structured
                                .fields
                                .into_iter()
                                .map(|field| {
                                    GlobalCacheTarget::Field(Cow::Owned(structured.name.clone()), Cow::Owned(field))
                                })
                                .collect()
                        })
                        .for_each(|target| {
                            validate_insert(
                                target,
                                rule.max_age,
                                rule.stale_while_revalidate,
                                rule.mutation_invalidation_policy.clone(),
                                rule.access_scopes
                                    .clone()
                                    .map(|scopes| scopes.into_iter().map(|scope| scope.0).collect()),
                            );
                        });
                }
            }
        }

        visited_rules
    }
}

impl From<CacheDirective> for CacheControl {
    fn from(value: CacheDirective) -> Self {
        CacheControl {
            public: false,
            max_age: value.max_age,
            stale_while_revalidate: value.stale_while_revalidate,
            invalidation_policy: value.mutation_invalidation_policy,
            access_scopes: value
                .access_scopes
                .map(|scopes| scopes.into_iter().map(|scope| scope.0).collect()),
        }
    }
}

impl Directive for CacheDirective {
    fn definition() -> String {
        // check the tests to understand how to use it
        // slack thread: https://grafbase.slack.com/archives/C03FXRVCKGS/p1684841634755919
        // PR discussion: https://github.com/grafbase/api/pull/2227#discussion_r1200552979
        "directive @cache on SCHEMA | OBJECT | FIELD_DEFINITION".to_string()
    }
}
