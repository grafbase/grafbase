use std::{fmt, sync::atomic::Ordering};

use serde::{
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};

use super::SeedContextInner;
use crate::{
    request::BoundFieldId,
    response::{GraphqlError, ResponseValue},
};

pub(super) struct ListSeed<'ctx, 'parent, Seed> {
    pub bound_field_id: BoundFieldId,
    pub ctx: &'parent SeedContextInner<'ctx>,
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
        let mut values = if let Some(size_hint) = seq.size_hint() {
            Vec::<ResponseValue>::with_capacity(size_hint)
        } else {
            Vec::<ResponseValue>::new()
        };

        loop {
            self.ctx.push_edge(index.into());
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
                    if !self.ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                        let mut path = self.ctx.response_path();
                        path.push(index.into());
                        self.ctx.response_part.borrow_mut().push_error(GraphqlError {
                            message: err.to_string(),
                            locations: self
                                .ctx
                                .plan
                                .bound_walk_with(self.bound_field_id, ())
                                .name_location()
                                .into_iter()
                                .collect(),
                            path: Some(path),
                            ..Default::default()
                        });
                    }
                    // Discarding the rest of the sequence.
                    while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
                    return Err(err);
                }
            }
        }

        Ok(ResponseValue::List {
            id: self.ctx.response_part.borrow_mut().push_list(&values),
            nullable: false,
        })
    }
}
