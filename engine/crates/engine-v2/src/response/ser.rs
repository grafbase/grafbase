use super::Response;

struct JsonResponseGraph<'resp> {
    response: &'resp Response,
}

impl<'resp> serde::Serialize for JsonResponseGraph<'resp> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
