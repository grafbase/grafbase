use std::fmt;

use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    execution::ExecutionContext,
    plan::PlanOutput,
    response::{ExecutorOutput, ResponseBoundaryItem},
    sources::ExecutorError,
};

pub struct EntitiesDataSeed<'a> {
    pub ctx: ExecutionContext<'a>,
    pub response_boundary: &'a Vec<ResponseBoundaryItem>,
    pub output: &'a mut ExecutorOutput,
    pub plan_output: &'a PlanOutput,
}

impl<'de, 'a> DeserializeSeed<'de> for EntitiesDataSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntitiesDataSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("data with an entities list")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "_entities" => {
                    let seed = EntitiesSeed {
                        ctx: self.ctx,
                        response_boundary: self.response_boundary,
                        output: self.output,
                        plan_output: self.plan_output,
                    };
                    map.next_value_seed(seed)?;
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

struct EntitiesSeed<'a> {
    ctx: ExecutionContext<'a>,
    response_boundary: &'a Vec<ResponseBoundaryItem>,
    output: &'a mut ExecutorOutput,
    plan_output: &'a PlanOutput,
}

impl<'de, 'a> DeserializeSeed<'de> for EntitiesSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntitiesSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut index = 0;
        loop {
            if index >= self.response_boundary.len() {
                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    self.output.push_error(ExecutorError::Internal(
                        "Received more entities than expected".to_string(),
                    ));
                    while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                }
                break;
            }
            let writer = self
                .ctx
                .writer(self.output, &self.response_boundary[index], self.plan_output);
            match seq.next_element_seed(writer) {
                Ok(Some(_)) => {
                    index += 1;
                }
                Ok(None) => break,
                Err(err) => {
                    while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}
