use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
    Deserializer,
};

use super::errors::UpstreamGraphqlErrorsSeed;
use crate::response::{GraphqlError, ResponsePath, SeedContext};

pub fn ingest_deserializer_into_response<'ctx, 'de, DataSeed, D>(
    ctx: &SeedContext<'ctx>,
    root_err_path: &'ctx ResponsePath,
    seed: DataSeed,
    deserializer: D,
) where
    D: Deserializer<'de>,
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    let result = GraphqlResponseSeed {
        ctx,
        root_err_path,
        seed: Some(seed),
    }
    .deserialize(deserializer);
    if let Err(err) = result {
        report_error_if_no_others(ctx, root_err_path, format!("Upstream response error: {err}"));
    }
}

struct GraphqlResponseSeed<'ctx, 'parent, DataSeed> {
    ctx: &'parent SeedContext<'ctx>,
    seed: Option<DataSeed>,
    root_err_path: &'ctx ResponsePath,
}

impl<'ctx, 'parent, 'de, DataSeed> DeserializeSeed<'de> for GraphqlResponseSeed<'ctx, 'parent, DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'ctx, 'parent, 'de, DataSeed> Visitor<'de> for GraphqlResponseSeed<'ctx, 'parent, DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid GraphQL response")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data_is_null: bool = true;
        let mut errors = vec![];
        while let Some(key) = map.next_key::<ResponseKey>()? {
            match key {
                ResponseKey::Data => match self.seed.take() {
                    Some(seed) => {
                        data_is_null = map
                            .next_value_seed(InfaillibleNullableSeed {
                                ctx: self.ctx,
                                root_err_path: self.root_err_path,
                                seed,
                            })
                            .expect("Infaillible by design.");
                    }
                    None => return Err(serde::de::Error::custom("data key present multiple times.")),
                },
                ResponseKey::Errors => map.next_value_seed(UpstreamGraphqlErrorsSeed {
                    path: self.root_err_path,
                    errors: &mut errors,
                })?,
                ResponseKey::Unknown => {
                    map.next_value::<IgnoredAny>()?;
                }
            };
        }
        if !errors.is_empty() && data_is_null {
            self.ctx.borrow_mut_response_part().replace_errors(errors);
        }
        Ok(())
    }
}

struct InfaillibleNullableSeed<'ctx, 'parent, Seed> {
    ctx: &'parent SeedContext<'ctx>,
    root_err_path: &'ctx ResponsePath,
    seed: Seed,
}

impl<'de, 'ctx, 'parent, Seed> DeserializeSeed<'de> for InfaillibleNullableSeed<'ctx, 'parent, Seed>
where
    Seed: DeserializeSeed<'de, Value = ()>,
{
    type Value = bool;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, 'ctx, 'parent, Seed> Visitor<'de> for InfaillibleNullableSeed<'ctx, 'parent, Seed>
where
    Seed: DeserializeSeed<'de, Value = ()>,
{
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a nullable value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_none()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Err(err) = self
            .seed
            .deserialize(serde_value::ValueDeserializer::<E>::new(serde_value::Value::Option(
                None,
            )))
        {
            report_error_if_no_others(self.ctx, self.root_err_path, format!("Upstream data error: {err}"));
        }
        Ok(true)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Err(err) = self.seed.deserialize(deserializer) {
            report_error_if_no_others(self.ctx, self.root_err_path, format!("Upstream data error: {err}"));
        }
        Ok(false)
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseKey {
    Data,
    Errors,
    #[serde(other)]
    Unknown,
}

fn report_error_if_no_others(ctx: &SeedContext<'_>, err_path: &ResponsePath, message: String) {
    let mut response_part = ctx.borrow_mut_response_part();
    // Only adding this if no other more precise errors were added.
    if !response_part.has_errors() {
        response_part.push_error(GraphqlError {
            message,
            path: Some(err_path.clone()),
            ..Default::default()
        });
    }
}
