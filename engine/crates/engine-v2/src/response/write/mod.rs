mod de;
mod selection_set;

use de::AnyFieldsSeed;
use serde::de::DeserializeSeed;

use super::{ResponseData, ResponseObject, ResponseObjectId, ResponseValue};

impl ResponseData {
    // Temporary as it's simple. We still need to validate the data we're receiving in all cases.
    // Upstream might break the contract. This basically got me started.
    #[allow(clippy::panic, dead_code)]
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
        let response_object = self.get_mut(object_node_id);
        for (name, value) in fields {
            response_object.fields.insert(name, value);
        }
        Ok(())
    }

    #[allow(clippy::panic)]
    pub fn write_fields_json(&mut self, object_node_id: ResponseObjectId, value: serde_json::Value) {
        let mut response_fields = vec![];
        match value {
            serde_json::Value::Null => (),
            serde_json::Value::Object(fields) => {
                for (name, value) in fields {
                    let name = self.strings.get_or_intern(&name);
                    let value = self.push_json_value(value);
                    response_fields.push((name, value));
                }
            }
            _ => panic!("Expected object or null"),
        }
        let response_object = self.get_mut(object_node_id);
        for (name, value) in response_fields {
            response_object.fields.insert(name, value);
        }
    }

    fn push_json_value(&mut self, value: serde_json::Value) -> ResponseValue {
        match value {
            serde_json::Value::Null => ResponseValue::Null,
            serde_json::Value::Bool(b) => ResponseValue::Bool(b),
            serde_json::Value::Number(n) => ResponseValue::Number(n),
            serde_json::Value::String(s) => ResponseValue::String(s),
            serde_json::Value::Array(arr) => {
                ResponseValue::List(arr.into_iter().map(|v| self.push_json_value(v)).collect())
            }
            serde_json::Value::Object(obj) => {
                let response_object = ResponseObject {
                    object_id: None,
                    fields: obj
                        .into_iter()
                        .map(|(name, value)| (self.strings.get_or_intern(&name), self.push_json_value(value)))
                        .collect(),
                };
                ResponseValue::Object(self.push_object(response_object))
            }
        }
    }
}
