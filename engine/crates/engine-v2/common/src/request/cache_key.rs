use runtime::{
    auth::AccessToken,
    cache::{CacheScopeDefinition, OperationCacheControl},
};

use sha2::Digest;

use crate::{BorrowedValue, BorrowedVariables, GraphqlRequest, HasPersistedQueryExtension};

const OPERATION_CACHE_CONTROL_CACHE_KEY_VERSION: u8 = 0x0;
const RESPONSE_CACHE_KEY_VERSION: u8 = 0x0;
const JSON_CONTENT_TYPE: u8 = 0x0;

const QUERY: u8 = 0x0;
const PERSISTED_QUERY: u8 = 0x1;

#[derive(Clone)]
pub struct SchemaVersion(Vec<u8>);

impl<T: Into<Vec<u8>>> From<T> for SchemaVersion {
    fn from(version: T) -> Self {
        Self(version.into())
    }
}

#[derive(Clone)]
pub struct OperationCacheControlCacheKey(Vec<u8>);

impl OperationCacheControlCacheKey {
    pub fn build<E: HasPersistedQueryExtension>(
        schema_version: &SchemaVersion,
        request: &GraphqlRequest<'_, E>,
    ) -> Self {
        let mut hasher = sha2::Sha256::new();

        hasher.update([OPERATION_CACHE_CONTROL_CACHE_KEY_VERSION]);
        let engine_version = crate::built_info::git_version();
        hasher.update(engine_version.len().to_le_bytes());
        hasher.update(engine_version);
        hasher.update(schema_version.0.len().to_le_bytes());
        hasher.update(&schema_version.0);

        if let Some(operation_name) = request.operation_name.as_deref() {
            hasher.update(operation_name.as_bytes());
        }
        hasher.update([0x0]);

        if let Some(persisted_query) = request.extensions.persisted_query() {
            hasher.update([PERSISTED_QUERY]);
            hasher.update(persisted_query.version.to_le_bytes());
            hasher.update(&persisted_query.sha256_hash);
        } else {
            hasher.update([QUERY]);
            if let Some(query) = request.query.as_ref() {
                hasher.update(query.as_bytes());
            }
            hasher.update([0x0]);
        }

        Self(hasher.finalize().to_vec())
    }
}

impl ToString for OperationCacheControlCacheKey {
    fn to_string(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        URL_SAFE_NO_PAD.encode(&self.0)
    }
}

pub struct ResponseCacheKey(Vec<u8>);

impl ResponseCacheKey {
    pub fn build<E: HasPersistedQueryExtension>(
        headers: &http::HeaderMap,
        access_token: &AccessToken,
        request: &GraphqlRequest<'_, E>,
        operation_cache_control: &OperationCacheControl,
    ) -> Option<Self> {
        if operation_cache_control.max_age.is_zero() {
            return None;
        }

        // We must not cache anything private, only the browser should.
        if operation_cache_control.is_private() {
            return None;
        }

        // We could use something else like blake3 for x86_64 or eventually a WASM api
        let mut hasher = sha2::Sha256::new();

        // We have to generate a unique bytes sequence for the cache scope. So it should be treated
        // as serialization. If you change something, ask yourself the question: Could you deserialize it back?
        hasher.update([RESPONSE_CACHE_KEY_VERSION, JSON_CONTENT_TYPE]);
        hasher.update(operation_cache_control.scopes().len().to_le_bytes());
        for scope in operation_cache_control.scopes() {
            hasher.update([scope.stable_id()]);
            match scope {
                // The stable id is enough for public & authenticated
                CacheScopeDefinition::Public => {}
                CacheScopeDefinition::Authenticated => {
                    if access_token.is_anonymous() {
                        return None;
                    }
                }
                CacheScopeDefinition::JwtClaim { path } => {
                    let value = access_token.get_claim_with_path(path);
                    // Not ideal, but simple
                    serde_json::to_writer(&mut hasher, value).expect("Access token was deserialized");
                    // Cannot be included in a JWT claim and needed to ensure we could detect an
                    // empty value.
                    hasher.update([0x0]);
                }
                CacheScopeDefinition::HeaderValue { name } => {
                    if let Some(value) = headers.get(name) {
                        hasher.update(value.as_bytes());
                    }
                    // Cannot be included in a header value and needed to ensure we could detect an
                    // empty value.
                    hasher.update([0x0]);
                }
            }
        }

        if let Some(operation_name) = request.operation_name.as_deref() {
            hasher.update(operation_name.as_bytes());
        }
        hasher.update([0x0]);
        if let Some(persisted_query) = request.extensions.persisted_query() {
            hasher.update([PERSISTED_QUERY]);
            hasher.update(persisted_query.version.to_le_bytes());
            hasher.update(&persisted_query.sha256_hash);
        } else {
            hasher.update([QUERY]);
            if let Some(query) = request.query.as_ref() {
                hasher.update(query.as_bytes());
            }
            hasher.update([0x0]);
        }

        VariablesHashBuilder {
            variables: &request.variables,
            hasher: &mut hasher,
        }
        .update_with_value(&request.variables.root);

        Some(Self(hasher.finalize().to_vec()))
    }
}

impl ToString for ResponseCacheKey {
    fn to_string(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        URL_SAFE_NO_PAD.encode(&self.0)
    }
}

pub struct VariablesHashBuilder<'a> {
    variables: &'a BorrowedVariables<'a>,
    hasher: &'a mut sha2::Sha256,
}

impl VariablesHashBuilder<'_> {
    fn update_with_value(&mut self, value: &BorrowedValue<'_>) {
        self.update([value.stable_id()]);
        match value {
            // The stable id is enough
            BorrowedValue::Null => (),
            BorrowedValue::Bool(b) => self.update([*b as u8]),
            BorrowedValue::F64(value) => {
                self.update(value.to_le_bytes());
            }
            BorrowedValue::I64(value) => self.update(value.to_le_bytes()),
            BorrowedValue::U64(value) => self.update(value.to_le_bytes()),
            BorrowedValue::String(value) => {
                self.update(value.as_bytes());
                self.update([0x0]);
            }
            BorrowedValue::List(range) => {
                let values = &self.variables[*range];
                self.update(values.len().to_le_bytes());
                for value in values {
                    self.update_with_value(value);
                }
            }
            BorrowedValue::Map(range) => {
                let key_values = &self.variables[*range];
                self.update(key_values.len().to_le_bytes());
                // key values are sorted by the key.
                for (key, value) in key_values {
                    self.update(key.as_bytes());
                    self.update([0x0]);
                    self.update_with_value(value);
                }
            }
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.hasher.update(data)
    }
}
