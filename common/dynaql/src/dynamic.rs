use std::ops::Deref;

use crate::registry::{MetaField, MetaType, MetaTypeName};

use crate::resolver_utils::resolve_input;
use crate::{Context, ContextSelectionSet, ServerError, ServerResult};
use dynaql_parser::types::{Field, SelectionSet};
use dynaql_parser::Positioned;
use dynaql_value::ConstValue;

#[derive(Debug, Clone)]
pub struct DynamicFieldContext<'ctx> {
    pub base: Context<'ctx>,
    pub maybe_parent_field: Option<&'ctx DynamicFieldContext<'ctx>>,
    pub meta: &'ctx MetaField,
    pub base_type: &'ctx MetaType,
}

impl<'ctx> Deref for DynamicFieldContext<'ctx> {
    type Target = Context<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub enum DynamicFieldKind {
    PRIMITIVE,
    ARRAY,
    OBJECT,
}

impl<'ctx> DynamicFieldContext<'ctx> {
    pub fn get_selection<'s>(&'s self) -> DynamicSelectionSetContext<'s>
    where
        'ctx: 's,
    {
        DynamicSelectionSetContext {
            maybe_parent_field: Some(self),
            base: self
                .base
                .with_selection_set(&self.base.item.node.selection_set),
            root_type: self.base_type,
        }
    }

    pub fn kind(&self) -> DynamicFieldKind {
        if self.meta.is_array() {
            DynamicFieldKind::ARRAY
        } else if self.base.item.node.selection_set.node.items.is_empty() {
            DynamicFieldKind::PRIMITIVE
        } else {
            DynamicFieldKind::OBJECT
        }
    }

    pub fn param_value_dynamic(&self, name: &str) -> ServerResult<serde_json::Value> {
        if let Some(meta_input_value) = self.meta.args.get(name) {
            let maybe_value = self
                .item
                .node
                .arguments
                .iter()
                .find(|(n, _)| n.node.as_str() == name)
                .map(|(_, value)| value)
                .cloned();

            let const_value = match maybe_value {
                Some(value) => self.resolve_input_value(value)?,
                None => ConstValue::Null,
            };
            resolve_input(self, meta_input_value, const_value)
        } else {
            Err(ServerError::new(
                "Internal Error: Unknown argument '{name}'",
                Some(self.item.pos),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicSelectionSetContext<'ctx> {
    pub maybe_parent_field: Option<&'ctx DynamicFieldContext<'ctx>>,
    pub base: ContextSelectionSet<'ctx>,
    pub root_type: &'ctx MetaType,
}

impl<'ctx> Deref for DynamicSelectionSetContext<'ctx> {
    type Target = ContextSelectionSet<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'ctx> DynamicSelectionSetContext<'ctx> {
    pub fn dynamic_with_field<'s>(
        &'s self,
        field: &'s Positioned<Field>,
        meta: &'s MetaField,
    ) -> DynamicFieldContext<'s>
    where
        'ctx: 's,
    {
        DynamicFieldContext {
            maybe_parent_field: self.maybe_parent_field,
            base: self.base.with_field(field),
            meta,
            base_type: self
                .schema_env
                .registry
                .types
                .get(MetaTypeName::concrete_typename(&meta.ty))
                .expect("Schema was already validated at this point"),
        }
    }

    pub fn dynamic_with_selection_set<'a>(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
    ) -> DynamicSelectionSetContext<'a>
    where
        'ctx: 'a,
    {
        DynamicSelectionSetContext {
            maybe_parent_field: self.maybe_parent_field,
            base: self.base.with_selection_set(selection_set),
            root_type: self.root_type,
        }
    }

    pub fn dynamic_with_index<'s>(&'s self, idx: usize) -> DynamicSelectionSetContext<'s>
    where
        'ctx: 's,
    {
        DynamicSelectionSetContext {
            maybe_parent_field: self.maybe_parent_field,
            base: self.base.with_index(idx),
            root_type: self.root_type,
        }
    }
}
