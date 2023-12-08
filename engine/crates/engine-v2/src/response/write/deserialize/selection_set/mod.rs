mod arbitrary;
mod grouped;

use arbitrary::*;
pub(crate) use grouped::*;
use schema::ObjectId;
use serde::de::DeserializeSeed;

use super::SeedContext;
use crate::{
    plan::ExpectedSelectionSet,
    request::SelectionSetType,
    response::{ResponseBoundaryItem, ResponsePath, ResponseValue},
};

pub(super) struct SelectionSetSeed<'ctx, 'parent> {
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
        let (maybe_boundary_id, response_object) = match self.expected {
            ExpectedSelectionSet::Grouped(expected) => {
                let maybe_boundary_id = expected.maybe_boundary_id;
                let object = ObjectFieldsSeed {
                    ctx: self.ctx,
                    path: self.path,
                    expected,
                }
                .deserialize(deserializer)?;
                (maybe_boundary_id, object)
            }
            ExpectedSelectionSet::Arbitrary(expected) => {
                let maybe_boundary_id = expected.maybe_boundary_id;
                let object = ArbitraryFieldsSeed {
                    ctx: self.ctx,
                    path: self.path,
                    expected,
                }
                .deserialize(deserializer)?;
                (maybe_boundary_id, object)
            }
        };
        let mut data = self.ctx.data.borrow_mut();
        let object_id = response_object.object_id;
        let id = data.push_object(response_object);
        if let Some(boundary_id) = maybe_boundary_id {
            data[boundary_id].push(ResponseBoundaryItem {
                response_object_id: id,
                response_path: self.path.clone(),
                object_id,
            });
        }
        Ok(ResponseValue::Object { id, nullable: false })
    }
}

enum ObjectIdentifier<'ctx, 'parent> {
    Known(ObjectId),
    Unknown {
        discriminant_key: &'ctx str,
        ctx: &'parent SeedContext<'ctx>,
        root: SelectionSetType,
    },
    Failure {
        discriminant_key: &'ctx str,
        discriminant: String,
        ctx: &'parent SeedContext<'ctx>,
        root: SelectionSetType,
    },
}

impl<'ctx, 'parent> ObjectIdentifier<'ctx, 'parent> {
    fn new(ctx: &'parent SeedContext<'ctx>, root: SelectionSetType) -> Self {
        let schema = ctx.walker.schema().get();
        match root {
            SelectionSetType::Interface(interface_id) => Self::Unknown {
                discriminant_key: ctx.walker.names().interface_discriminant_key(schema, interface_id),
                root,
                ctx,
            },
            SelectionSetType::Union(union_id) => Self::Unknown {
                discriminant_key: ctx.walker.names().union_discriminant_key(schema, union_id),
                root,
                ctx,
            },
            SelectionSetType::Object(object_id) => Self::Known(object_id),
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
                SelectionSetType::Interface(interface_id) => ctx
                    .walker
                    .names()
                    .concrete_object_id_from_interface_discriminant(&ctx.walker.schema(), *interface_id, discriminant),
                SelectionSetType::Union(union_id) => ctx.walker.names().concrete_object_id_from_union_discriminant(
                    &ctx.walker.schema(),
                    *union_id,
                    discriminant,
                ),
                SelectionSetType::Object(_) => unreachable!("We wouldn't be trying to guess it otherwise."),
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
                ctx.walker.schema().walk(schema::Definition::from(root)).name()
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
                ctx.walker.schema().walk(schema::Definition::from(root)).name()
            ))),
        }
    }
}
