mod de;
mod selection_set;

use de::AnyFieldsSeed;
pub use selection_set::{WriteSelection, WriteSelectionSet};
use serde::de::DeserializeSeed;

use super::{Response, ResponseMutObject, ResponseObjectId};

impl Response {
    // Temporary as it's simple. We still need to validate the data we're receiving in all cases.
    // Upstream might break the contract. This basically got me started.
    #[allow(clippy::panic)]
    pub fn write_fields_any<'de, D>(
        &mut self,
        object_node_id: ResponseObjectId,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed = AnyFieldsSeed { response: self };
        let fields = seed.deserialize(deserializer)?;
        let response_object = match self.get_mut(object_node_id) {
            ResponseMutObject::Sparse(obj) => obj,
            ResponseMutObject::Dense(_) => panic!("Cannot add any fields in dense reponse object."),
        };
        for (name, value) in fields {
            response_object.fields.insert(name, value);
        }
        Ok(())
    }
}
