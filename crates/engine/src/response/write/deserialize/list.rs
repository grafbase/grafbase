use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};
use walker::Walk;

use super::SeedContext;
use crate::{
    operation::DataFieldId,
    response::{ErrorCode, GraphqlError, ResponseValue},
};

pub(super) struct ListSeed<'ctx, 'parent, Seed> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field_id: DataFieldId,
    pub seed: &'parent Seed,
}

impl<'de, 'ctx, 'parent, Seed> DeserializeSeed<'de> for ListSeed<'ctx, 'parent, Seed>
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

impl<'de, 'ctx, 'parent, Seed> Visitor<'de> for ListSeed<'ctx, 'parent, Seed>
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
        let mut index: usize = 0;
        let mut values = self.ctx.writer.new_list();
        if let Some(size_hint) = seq.size_hint() {
            values.reserve(size_hint);
        }

        loop {
            self.ctx.push_edge(index);
            let result = seq.next_element_seed(self.seed.clone());
            self.ctx.pop_edge();
            match result {
                Ok(Some(value)) => {
                    values.push(value);
                    index += 1;
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    if self.ctx.should_create_new_graphql_error() {
                        let mut path = self.ctx.response_path();
                        path.push(index);
                        self.ctx.writer.push_error(
                            GraphqlError::new(err.to_string(), ErrorCode::SubgraphInvalidResponseError)
                                .with_location(self.field_id.walk(self.ctx).location)
                                .with_path(path),
                        );
                    }
                    // Discarding the rest of the sequence.
                    while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
                    return Err(err);
                }
            }
        }

        Ok(self.ctx.writer.push_list(values).into())
    }
}
