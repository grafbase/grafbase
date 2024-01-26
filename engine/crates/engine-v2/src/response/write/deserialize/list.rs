use std::{fmt, sync::atomic::Ordering};

use serde::{
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};

use super::SeedContextInner;
use crate::{
    request::BoundFieldId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct ListSeed<'ctx, 'parent, F> {
    pub path: &'parent ResponsePath,
    pub bound_field_id: Option<BoundFieldId>,
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub seed_builder: F,
}

impl<'de, 'ctx, 'parent, F, Seed> DeserializeSeed<'de> for ListSeed<'ctx, 'parent, F>
where
    F: Fn(ResponsePath) -> Seed,
    Seed: DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx, 'parent, F, Seed> Visitor<'de> for ListSeed<'ctx, 'parent, F>
where
    F: Fn(ResponsePath) -> Seed,
    Seed: DeserializeSeed<'de, Value = ResponseValue>,
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
            match seq.next_element_seed((self.seed_builder)(self.path.child(index))) {
                Ok(Some(value)) => {
                    values.push(value);
                    index += 1;
                }
                Ok(None) => break,
                Err(err) => {
                    if !self.ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                        self.ctx.data.borrow_mut().push_error(GraphqlError {
                            message: err.to_string(),
                            locations: self
                                .bound_field_id
                                .map(|id| self.ctx.walker.walk(id).name_location())
                                .into_iter()
                                .collect(),
                            path: Some(self.path.clone()),
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
            id: self.ctx.data.borrow_mut().push_list(&values),
            nullable: false,
        })
    }
}
