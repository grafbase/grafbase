use common_types::auth::ExecutionAuth;
use engine_value::{ConstValue, Variables};
use registry_for_cache::{CacheAccessScope, CacheControl};

use crate::query_subset::QuerySubsetDisplay;

use super::CachingPlan;

pub fn build_cache_keys(
    plan: &CachingPlan,
    auth: &ExecutionAuth,
    headers: &http::HeaderMap,
    variables: &Variables,
) -> Vec<Option<String>> {
    plan.cache_partitions
        .iter()
        .map(|(cache_control, query_subset)| {
            // If we can't figure out the scope for a cache_control entry we just skip it.
            // This will force it to always be fetched from the cache.
            let scopes = cache_scopes(cache_control, auth, headers)?;

            let serializer = CacheKeySerializer {
                query: query_subset.as_display(&plan.document),
                scopes,
                variables: variables_required(query_subset, &plan.document, variables),
            };

            match serializer.build_key_string() {
                Ok(key) => Some(key),
                Err(err) => {
                    tracing::error!("Could not build key string for {cache_control:?}: {err:?}");
                    None
                }
            }
        })
        .collect()
}

fn cache_scopes<'a>(
    cache_control: &CacheControl,
    auth: &'a ExecutionAuth,
    headers: &http::HeaderMap,
) -> Option<CacheAccess<'a>> {
    let Some(scopes) = &cache_control.access_scopes else {
        return Some(CacheAccess::Default(auth));
    };

    let actual_scopes = scopes
        .iter()
        .map(|scope| match scope {
            CacheAccessScope::Public | CacheAccessScope::ApiKey => Some(auth.global_ops().to_string()),
            CacheAccessScope::Jwt { claim } => auth.as_token().and_then(|token| token.get_claim(claim)),
            CacheAccessScope::Header { header: header_name } => headers
                .get(header_name)
                .and_then(|header| Some(header.to_str().ok()?.to_string())),
        })
        .collect::<Vec<_>>();

    if actual_scopes.is_empty() {
        tracing::warn!("Could not find any scopes for cache_control {cache_control:?}");
        return None;
    }

    Some(CacheAccess::Scoped(actual_scopes))
}

#[derive(Debug, Hash, serde::Serialize)]
enum CacheAccess<'a> {
    Scoped(Vec<Option<String>>),
    Default(&'a ExecutionAuth),
}

fn variables_required<'a>(
    query_subset: &'a crate::QuerySubset,
    document: &'a cynic_parser::ExecutableDocument,
    variables: &'a Variables,
) -> Vec<(&'a str, &'a ConstValue)> {
    query_subset
        .variables(document)
        .filter_map(|definition| Some((definition.name(), variables.get(definition.name())?)))
        .collect()
}

/// All the components of an individual cache key go in here
///
/// We use Serialize to create a JSON string, then hash that
#[derive(serde::Serialize)]
struct CacheKeySerializer<'a> {
    query: QuerySubsetDisplay<'a>,
    scopes: CacheAccess<'a>,
    variables: Vec<(&'a str, &'a ConstValue)>,
}

impl CacheKeySerializer<'_> {
    fn build_key_string(&self) -> anyhow::Result<String> {
        let mut hasher = blake3::Hasher::new();

        serde_json::to_writer(&mut hasher, self)?;

        Ok(hasher.finalize().to_hex().to_string())
    }
}

// Doing a manual impl here because then we can call collect_str which
// will _hopefully_ stream the query string instead of allocating a whole String
// for it
impl serde::Serialize for QuerySubsetDisplay<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
