use serde::{de::DeserializeSeed, Deserializer};

use crate::{
    execution::ExecutionContext,
    response::{ErrorCode, GraphqlError, ResponseKeys, ResponsePath, SubgraphResponseRefMut, UnpackedResponseEdge},
    Runtime,
};

pub(super) trait GraphqlErrorsSeed<'resp> {
    fn response(&self) -> &SubgraphResponseRefMut<'resp>;
    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath>;
}

pub(in crate::sources::graphql) struct RootGraphqlErrors<'resp> {
    response: SubgraphResponseRefMut<'resp>,
    response_keys: &'resp ResponseKeys,
}

impl<'resp> RootGraphqlErrors<'resp> {
    pub fn new<R: Runtime>(ctx: &ExecutionContext<'resp, R>, response: SubgraphResponseRefMut<'resp>) -> Self {
        Self {
            response,
            response_keys: &ctx.operation.response_keys,
        }
    }
}

impl<'resp> GraphqlErrorsSeed<'resp> for RootGraphqlErrors<'resp> {
    fn response(&self) -> &SubgraphResponseRefMut<'resp> {
        &self.response
    }

    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath> {
        let mut out = ResponsePath::default();
        for edge in path.as_array()? {
            if let Some(index) = edge.as_u64() {
                out.push(index as usize);
            } else {
                let key = edge.as_str()?;
                let response_key = self.response_keys.get(key)?;
                // We need this path for two reasons only:
                // - To report nicely in the error message
                // - To know whether an error exist if we're missing the appropriate data for a
                //   response object.
                // For the latter we only check whether there is an error at all, not if it's one
                // that could actually propagate up to the root response object. That's a lot more
                // work and very likely useless.
                // So, currently, we'll never read those fields and treat them as extra field
                // to cram them into an ResponseEdge.
                out.push(UnpackedResponseEdge::ExtraFieldResponseKey(response_key.into()))
            }
        }
        Some(out)
    }
}

#[derive(serde::Deserialize)]
pub(super) struct SubgraphGraphqlError {
    pub message: String,
    #[serde(default)]
    pub path: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

pub(super) struct ConcreteGraphqlErrorsSeed<T>(pub(super) T);

impl<'resp, 'de, T> DeserializeSeed<'de> for ConcreteGraphqlErrorsSeed<T>
where
    T: GraphqlErrorsSeed<'resp>,
    'resp: 'de,
{
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let errors = <Option<Vec<SubgraphGraphqlError>> as serde::Deserialize>::deserialize(deserializer)?;

        let Some(errors) = errors else {
            return Ok(0);
        };

        let errors_count = errors.len();
        let errors = errors
            .into_iter()
            .map(|subgraph_error| {
                let mut error = GraphqlError::new(subgraph_error.message, ErrorCode::SubgraphError);
                if let Some(path) = self.0.convert_path(&subgraph_error.path) {
                    error = error.with_path(path);
                } else if !subgraph_error.path.is_null() {
                    error = error.with_extension("upstream_path", subgraph_error.path);
                }
                if !subgraph_error.extensions.is_null() {
                    error = error.with_extension("upstream_extensions", subgraph_error.extensions);
                }
                error
            })
            .collect();
        self.0.response().push_errors(errors);
        Ok(errors_count)
    }
}
