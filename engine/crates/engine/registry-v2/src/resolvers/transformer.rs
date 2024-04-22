use super::Resolver;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Transformer {
    GraphqlField,
    /// Key based Resolver for ResolverContext
    Select {
        key: String,
    },
    /// This resolver get the PaginationData
    PaginationData,
    /// Resolves the correct values of a remote enum using the given enum name
    RemoteEnum,
    /// Resolves the __typename of a remote union type
    RemoteUnion,
    /// Convert MongoDB timestamp as number
    MongoTimestamp,
    /// A special transformer to fetch Postgres page info for the current results.
    PostgresPageInfo,
    /// Calculate cursor value for a Postgres row.
    PostgresCursor,
    /// Set Postgres selection data.
    PostgresSelectionData {
        directive_name: String,
        table_id: TableId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct TableId(pub u32);

impl From<TableId> for u32 {
    fn from(value: TableId) -> Self {
        value.0
    }
}

impl Transformer {
    pub fn and_then(self, resolver: impl Into<Resolver>) -> Resolver {
        Resolver::Transformer(self).and_then(resolver)
    }

    pub fn select(key: &str) -> Self {
        Self::Select { key: key.to_string() }
    }
}

impl From<Transformer> for Resolver {
    fn from(value: Transformer) -> Self {
        Resolver::Transformer(value)
    }
}
