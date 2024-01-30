mod collected;
mod conditional;
mod runtime_concrete;

use std::borrow::Cow;

pub(crate) use collected::*;
use conditional::*;
use schema::ObjectId;
use serde::de::DeserializeSeed;

use self::runtime_concrete::RuntimeConcreteCollectionSetSeed;

use super::SeedContextInner;
use crate::{
    plan::CollectedSelectionSet,
    request::SelectionSetType,
    response::{ResponsePath, ResponseValue},
};

pub(super) struct SelectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub path: &'parent ResponsePath,
    pub collected: &'parent CollectedSelectionSet,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for SelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.collected {
            &CollectedSelectionSet::Concrete(id) => ConcreteCollectionSetSeed {
                ctx: self.ctx,
                path: self.path,
                id,
            }
            .deserialize(deserializer),
            &CollectedSelectionSet::Conditional(id) => ConditionalSelectionSetSeed {
                ctx: self.ctx,
                path: self.path,
                ty: self.ctx.plan[id].ty,
                selection_set_ids: Cow::Owned(vec![id]),
            }
            .deserialize(deserializer),
            CollectedSelectionSet::MergedConditionals { ty, selection_set_ids } => ConditionalSelectionSetSeed {
                ctx: self.ctx,
                path: self.path,
                ty: *ty,
                selection_set_ids: Cow::Borrowed(selection_set_ids),
            }
            .deserialize(deserializer),
            CollectedSelectionSet::RuntimeConcrete(selection_set) => RuntimeConcreteCollectionSetSeed {
                ctx: self.ctx,
                path: self.path,
                selection_set,
            }
            .deserialize(deserializer),
        }
    }
}

enum ObjectIdentifier<'ctx, 'parent> {
    Known(ObjectId),
    Unknown {
        discriminant_key: &'ctx str,
        ctx: &'parent SeedContextInner<'ctx>,
        root: SelectionSetType,
    },
    Failure {
        discriminant_key: &'ctx str,
        discriminant: String,
        ctx: &'parent SeedContextInner<'ctx>,
        root: SelectionSetType,
    },
}

impl<'ctx, 'parent> ObjectIdentifier<'ctx, 'parent> {
    fn new(ctx: &'parent SeedContextInner<'ctx>, root: SelectionSetType) -> Self {
        let schema = ctx.plan.schema();
        match root {
            SelectionSetType::Interface(interface_id) => Self::Unknown {
                discriminant_key: schema.names().interface_discriminant_key(schema.as_ref(), interface_id),
                root,
                ctx,
            },
            SelectionSetType::Union(union_id) => Self::Unknown {
                discriminant_key: schema.names().union_discriminant_key(schema.as_ref(), union_id),
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
            let schema = ctx.plan.schema();
            let maybe_object_id = match root {
                SelectionSetType::Interface(interface_id) => schema
                    .names()
                    .concrete_object_id_from_interface_discriminant(&schema, *interface_id, discriminant),
                SelectionSetType::Union(union_id) => {
                    schema
                        .names()
                        .concrete_object_id_from_union_discriminant(&schema, *union_id, discriminant)
                }
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
                "Upstream response error: Could not infer object. Discriminant key '{}' wasn't found for type named '{}'.",
                discriminant_key,
                ctx.plan.schema().walk(schema::Definition::from(root)).name()
            ))),
            ObjectIdentifier::Failure {
                discriminant_key,
                discriminant,
                ctx,
                root,
            } => Err(serde::de::Error::custom(format!(
                "Upstream response error: Could not infer object. Unknown discriminant '{}' (key: '{}') for type named '{}'.",
                discriminant,
                discriminant_key,
                ctx.plan.schema().walk(schema::Definition::from(root)).name()
            ))),
        }
    }
}
