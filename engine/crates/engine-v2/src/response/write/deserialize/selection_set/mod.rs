mod collected;
mod conditional;

use std::borrow::Cow;

pub(crate) use collected::*;
use conditional::*;
use schema::{ObjectId, SchemaWalker};
use serde::de::DeserializeSeed;

use super::SeedContextInner;
use crate::{
    plan::{AnyCollectedSelectionSet, RuntimeMergedConditionals},
    request::SelectionSetType,
    response::ResponseValue,
};

pub(super) struct SelectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub collected: &'parent AnyCollectedSelectionSet,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for SelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.collected {
            &AnyCollectedSelectionSet::Collected(id) => {
                CollectedSelectionSetSeed::new_from_id(self.ctx, id).deserialize(deserializer)
            }
            &AnyCollectedSelectionSet::Conditional(id) => ConditionalSelectionSetSeed {
                ctx: self.ctx,
                selection_set_ty: self.ctx.plan[id].ty,
                selection_set_ids: Cow::Owned(vec![id]),
            }
            .deserialize(deserializer),
            AnyCollectedSelectionSet::RuntimeMergedConditionals(RuntimeMergedConditionals {
                ty,
                selection_set_ids,
            }) => ConditionalSelectionSetSeed {
                ctx: self.ctx,
                selection_set_ty: *ty,
                selection_set_ids: Cow::Borrowed(selection_set_ids),
            }
            .deserialize(deserializer),
            AnyCollectedSelectionSet::RuntimeCollected(selection_set) => {
                CollectedSelectionSetSeed::new(self.ctx, selection_set).deserialize(deserializer)
            }
        }
    }
}

struct ObjectIdentifier<'ctx> {
    discriminant_key: &'ctx str,
    schema: SchemaWalker<'ctx, ()>,
    root: SelectionSetType,
}

impl<'ctx> ObjectIdentifier<'ctx> {
    fn new(ctx: &SeedContextInner<'ctx>, root: SelectionSetType) -> Self {
        let schema = ctx.plan.schema();
        match root {
            SelectionSetType::Interface(interface_id) => Self {
                discriminant_key: schema.names().interface_discriminant_key(schema.as_ref(), interface_id),
                schema,
                root,
            },
            SelectionSetType::Union(union_id) => Self {
                discriminant_key: schema.names().union_discriminant_key(schema.as_ref(), union_id),
                schema,
                root,
            },
            _ => unreachable!("Wouldn't be necessary"),
        }
    }

    fn discriminant_key_matches(&self, key: &str) -> bool {
        key == self.discriminant_key
    }

    fn determine_object_id_from_discriminant(&mut self, discriminant: &str) -> Option<ObjectId> {
        match self.root {
            SelectionSetType::Interface(interface_id) => self
                .schema
                .names()
                .concrete_object_id_from_interface_discriminant(self.schema.as_ref(), interface_id, discriminant),
            SelectionSetType::Union(union_id) => self.schema.names().concrete_object_id_from_union_discriminant(
                self.schema.as_ref(),
                union_id,
                discriminant,
            ),
            SelectionSetType::Object(_) => unreachable!("We wouldn't be trying to guess it otherwise."),
        }
    }
}
