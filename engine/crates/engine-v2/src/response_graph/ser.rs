use super::ResponseGraph;

struct JsonResponseGraph<'resp> {
    response_graph: &'resp ResponseGraph,
}

impl<'resp> serde::Serialize for JsonResponseGraph<'resp> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
