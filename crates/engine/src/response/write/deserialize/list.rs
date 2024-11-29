use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};

use super::SeedContext;
use crate::response::{FieldShapeRecord, ResponseValue, ResponseValueId};

pub(super) struct ListSeed<'ctx, 'parent, Seed> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field: &'parent FieldShapeRecord,
    pub seed: &'parent Seed,
    pub element_is_nullable: bool,
}

impl<'de, Seed> DeserializeSeed<'de> for ListSeed<'_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, Seed> Visitor<'de> for ListSeed<'_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut index: u32 = 0;
        let list_id = self.ctx.subgraph_response.borrow_mut().data.reserve_list_id();
        let mut list = Vec::new();
        if let Some(size_hint) = seq.size_hint() {
            list.reserve(size_hint);
        }

        loop {
            self.ctx.path().push(ResponseValueId::Index {
                list_id,
                index,
                nullable: self.element_is_nullable,
            });
            let result = seq.next_element_seed(self.seed.clone());
            self.ctx.path().pop();
            match result {
                Ok(Some(value)) => {
                    list.push(value);
                    index += 1;
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    self.ctx
                        .push_field_deserialization_error_if_not_bubbling_up(self.field, true, &err);
                    // Try discarding the rest of the list, we might be able to use other parts of
                    // the response.
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    break;
                }
            }
        }

        self.ctx.subgraph_response.borrow_mut().data.put_list(list_id, list);
        Ok(list_id.into())
    }
}
