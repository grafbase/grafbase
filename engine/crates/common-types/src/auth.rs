use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
};

use serde::Deserialize;
use serde_json::Value;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
    #[repr(transparent)]
    pub struct Operations: u8 {
        const CREATE        = 1 << 0;
        const GET           = 1 << 1;
        const LIST          = 1 << 2;
        const UPDATE        = 1 << 3;
        const DELETE        = 1 << 4;
        const INTROSPECTION = 1 << 5;
        // If both GET and LIST are set, READ is enabled transparently.
        // READ enables both GET and LIST.
        const READ   = Self::GET.bits() | Self::LIST.bits();
        // Similar to READ.
        const WRITE   = Self::CREATE.bits() | Self::UPDATE.bits() | Self::DELETE.bits();
    }
}

// Keep back-compatibility with the previous serialisation format.
impl serde::Serialize for Operations {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bits().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Operations {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Deserialize::deserialize(deserializer)?;
        Self::from_bits(value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid value for {type_name}: {value}",
                type_name = std::any::type_name::<Self>()
            ))
        })
    }
}

pub const API_KEY_OPS: Operations = Operations::all();

impl std::fmt::Display for Operations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        bitflags::parser::to_writer(self, &mut string)?;
        write!(f, "{}", string.to_lowercase())
    }
}

/// Created when authorizing the whole request,
/// represents global authorization with operations
/// defined on the global level.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Hash)]
pub enum ExecutionAuth {
    ApiKey,
    Token(ExecutionAuthToken),
    Public { global_ops: Operations },
}

impl ExecutionAuth {
    pub fn new_from_api_keys() -> Self {
        Self::ApiKey
    }

    pub fn new_from_token(
        private_public_and_group_ops: Operations,
        groups_from_token: BTreeSet<String>,
        subject_and_owner_ops: Option<(String, Operations)>,
        token_claims: BTreeMap<String, Value>,
    ) -> Self {
        Self::Token(ExecutionAuthToken {
            private_public_and_group_ops,
            groups_from_token,
            subject_and_owner_ops,
            token_claims,
        })
    }

    pub fn global_ops(&self) -> Operations {
        match self {
            Self::ApiKey => API_KEY_OPS,
            Self::Token(token) => token.global_ops(),
            Self::Public { global_ops } => *global_ops,
        }
    }

    pub fn is_introspection_allowed(&self) -> bool {
        self.global_ops().contains(Operations::INTROSPECTION)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ExecutionAuthToken {
    /// Private, public, group-based operations that are enabled on the global level.
    private_public_and_group_ops: Operations,
    groups_from_token: BTreeSet<String>,
    /// Owner's subject and enabled operations on the global level.
    subject_and_owner_ops: Option<(String, Operations)>,
    token_claims: BTreeMap<String, Value>,
}

impl Hash for ExecutionAuthToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.private_public_and_group_ops.hash(state);
        self.groups_from_token.hash(state);
        self.private_public_and_group_ops.hash(state);

        state.write_usize(self.token_claims.len());
        for (key, value) in &self.token_claims {
            key.hash(state);
            value.to_string().hash(state);
        }
    }
}

impl ExecutionAuthToken {
    fn global_ops(&self) -> Operations {
        self.private_public_and_group_ops.union(self.owner_ops())
    }

    pub fn groups_from_token(&self) -> &BTreeSet<String> {
        &self.groups_from_token
    }

    pub fn subject_and_owner_ops(&self) -> Option<&(String, Operations)> {
        self.subject_and_owner_ops.as_ref()
    }

    pub fn private_public_and_group_ops(&self) -> Operations {
        self.private_public_and_group_ops
    }

    pub fn owner_ops(&self) -> Operations {
        self.subject_and_owner_ops
            .as_ref()
            .map(|(_, ops)| *ops)
            .unwrap_or_default()
    }

    pub fn get_claim(&self, claim_name: &str) -> Option<String> {
        self.token_claims.get(claim_name).map(ToString::to_string)
    }
}
