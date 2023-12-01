mod arbitrary;
mod grouped;

pub use arbitrary::*;
pub use grouped::*;
use schema::ObjectId;
use serde::de::DeserializeSeed;

use super::SeedContext;
use crate::{
    plan::ExpectedSelectionSet,
    request::SelectionSetRoot,
    response::{ResponsePath, ResponseValue},
};

pub struct SelectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub path: &'parent ResponsePath,
    pub expected: &'parent ExpectedSelectionSet,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for SelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.expected {
            ExpectedSelectionSet::Grouped(expected) => ObjectFieldsSeed {
                ctx: self.ctx,
                path: self.path,
                expected,
            }
            .deserialize(deserializer),
            ExpectedSelectionSet::Arbitrary(expected) => ArbitraryFieldsSeed {
                ctx: self.ctx,
                path: self.path,
                expected,
            }
            .deserialize(deserializer),
        }
        .map(|object| ResponseValue::Object {
            id: self.ctx.data.borrow_mut().push_object(object),
            nullable: false,
        })
    }
}

enum ObjectIdentifier<'ctx, 'parent> {
    Known(ObjectId),
    Unknown {
        discriminant_key: &'ctx str,
        ctx: &'parent SeedContext<'ctx>,
        root: SelectionSetRoot,
    },
    Failure {
        discriminant_key: &'ctx str,
        discriminant: String,
        ctx: &'parent SeedContext<'ctx>,
        root: SelectionSetRoot,
    },
}

impl<'ctx, 'parent> ObjectIdentifier<'ctx, 'parent> {
    fn new(ctx: &'parent SeedContext<'ctx>, root: SelectionSetRoot) -> Self {
        match root {
            SelectionSetRoot::Interface(interface_id) => Self::Unknown {
                discriminant_key: ctx.schema_walker.names().interface_discriminant_key(interface_id),
                root,
                ctx,
            },
            SelectionSetRoot::Union(union_id) => Self::Unknown {
                discriminant_key: ctx.schema_walker.names().union_discriminant_key(union_id),
                root,
                ctx,
            },
            SelectionSetRoot::Object(object_id) => Self::Known(object_id),
        }
    }

    fn discriminant_key_matches(&self, key: &str) -> bool {
        match self {
            ObjectIdentifier::Unknown { discriminant_key, .. } => key == *discriminant_key,
            _ => false,
        }
    }

    fn determine_object_id_from_discriminant(&mut self, discriminant: &str) {
        if let ObjectIdentifier::Unknown {
            discriminant_key,
            ctx,
            root,
        } = self
        {
            let maybe_object_id = match root {
                SelectionSetRoot::Interface(interface_id) => ctx
                    .schema_walker
                    .names()
                    .conrete_object_id_from_interface_discriminant(*interface_id, discriminant),
                SelectionSetRoot::Union(union_id) => ctx
                    .schema_walker
                    .names()
                    .conrete_object_id_from_union_discriminant(*union_id, discriminant),
                SelectionSetRoot::Object(_) => unreachable!("We wouldn't be trying to guess it otherwise."),
            };
            if let Some(object_id) = maybe_object_id {
                *self = ObjectIdentifier::Known(object_id);
            } else {
                *self = Self::Failure {
                    discriminant_key,
                    discriminant: discriminant.to_string(),
                    root: *root,
                    ctx,
                }
            }
        };
    }

    fn try_into_object_id<E>(self) -> Result<ObjectId, E>
    where
        E: serde::de::Error,
    {
        match self {
            ObjectIdentifier::Known(object_id) => Ok(object_id),
            ObjectIdentifier::Unknown {
                discriminant_key,
                ctx,
                root,
            } => Err(serde::de::Error::custom(format!(
                "Could not infer object: discriminant key: '{}' wasn't found for type named '{}'",
                discriminant_key,
                ctx.schema_walker.walk(schema::Definition::from(root)).name()
            ))),
            ObjectIdentifier::Failure {
                discriminant_key,
                discriminant,
                ctx,
                root,
            } => Err(serde::de::Error::custom(format!(
                "Could not infer object: unknown discriminant '{}' (key: '{}') for type named '{}'",
                discriminant,
                discriminant_key,
                ctx.schema_walker.walk(schema::Definition::from(root)).name()
            ))),
        }
    }
}
