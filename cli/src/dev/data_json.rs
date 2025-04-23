//! The local app expects a `data.json` endpoint located next to its `index.html`. That data has to be populated on startup, and then reloaded anytime there is a change in schemas.

use std::sync::Arc;

use super::subgraphs::CachedSubgraph;
use chrono::{DateTime, Utc};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap as _, SerializeSeq as _},
};

/// The format of the data.json endpoint located next to the index.html.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) struct DataJson<'a> {
    #[serde(rename = "updatedAt")]
    pub(super) updated_at: DateTime<Utc>,
    pub(super) graphql_api_url: &'a str,
    pub(super) mcp_server_url: Option<&'a str>,
    pub(super) schemas: &'a DataJsonSchemas,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataJsonSchemas {
    pub(super) api_schema: Option<String>,
    pub(super) federated_schema: Option<String>,
    #[serde(serialize_with = "serialize_cached_subgraph")]
    pub(super) subgraphs: Vec<Arc<CachedSubgraph>>,
    pub(super) errors: Vec<DataJsonError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DataJsonError {
    pub(super) message: String,
    pub(super) severity: &'static str,
}

fn serialize_cached_subgraph<S>(subgraphs: &[Arc<CachedSubgraph>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    struct CachedSubgraphSerialize<'a>(&'a CachedSubgraph);

    impl Serialize for CachedSubgraphSerialize<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("name", &self.0.name)?;
            map.serialize_entry("schema", &self.0.sdl)?;
            map.end()
        }
    }

    let mut s = serializer.serialize_seq(Some(subgraphs.len()))?;

    for subgraph in subgraphs {
        s.serialize_element(&CachedSubgraphSerialize(subgraph))?;
    }

    s.end()
}
