use std::collections::{hash_map::DefaultHasher, BTreeSet};

use crate::RequestContext;
use common_types::auth::ExecutionAuth;
use engine::registry::CacheAccessScope;

use super::{
    key::{CacheAccess, CacheKey},
    CacheConfig,
};

#[derive(thiserror::Error, Debug)]
pub enum BuildKeyError {
    #[error("Not a single cache scope matched")]
    MultipleScopesError,
    #[error("Could not determine cache control: {0}")]
    CouldNotDetermineCacheControl(String),
}

pub fn build_cache_key(
    config: &CacheConfig,
    ctx: &impl RequestContext,
    request: &engine::Request,
    auth: &ExecutionAuth,
) -> Result<String, BuildKeyError> {
    let request_cache_control = config
        .partial_registry
        .get_cache_control(request)
        .map_err(|err| BuildKeyError::CouldNotDetermineCacheControl(err.to_string()))?;

    let cache_access = request_cache_control
        .access_scopes
        .map(|scopes| {
            let cache_key_access = scopes.iter().fold(BTreeSet::new(), |mut current_scopes, scope| {
                match scope {
                    CacheAccessScope::Public | CacheAccessScope::ApiKey => {
                        current_scopes.insert(auth.global_ops().to_string());
                    }
                    CacheAccessScope::Jwt { claim } => {
                        if let ExecutionAuth::Token(token) = &auth {
                            if let Some(claim_value) = token.get_claim(claim) {
                                current_scopes.insert(claim_value);
                            }
                        }
                    }
                    CacheAccessScope::Header { header: name } => {
                        if let Some(header_value) = ctx.headers().get(name).and_then(|header| header.to_str().ok()) {
                            current_scopes.insert(header_value.to_string());
                        }
                    }
                };

                current_scopes
            });

            CacheAccess::Scoped(cache_key_access)
        })
        .unwrap_or(CacheAccess::Default(auth));

    match &cache_access {
        CacheAccess::Scoped(scopes) if scopes.is_empty() => return Err(BuildKeyError::MultipleScopesError),
        _ => {}
    }

    // cache key
    // note: I opted for using `DefaultHasher` as its using SipHash-2-4.
    // this hashing algorithm is *not* collision resistant but it provides a good mix of security and speed
    // using cryptographic hashes provide a more secure alternative as they are collision resistant BUT are slower
    // additionally, each combination of <project>-<branch> gets their own cache in order to reduce the number keys directed to a particular cache
    // note: I'm also using DefaultHasher and not SipHash24 because SipHash direct usage is deprecated.
    // But beware that the default hash implementation can change across rust releases so pay attention to that when bumping
    let subdomain = &config.subdomain;
    let cache_key = CacheKey::<DefaultHasher>::new(cache_access, request, subdomain);

    Ok(format!("https://{}/{}", subdomain, cache_key.to_hash_string()))
}
