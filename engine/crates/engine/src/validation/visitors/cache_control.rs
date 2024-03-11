use std::collections::HashSet;

use crate::{
    parser::types::{Field, SelectionSet},
    registry::MetaType,
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    CacheControl, CacheInvalidation, Positioned,
};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut CacheControl,
    pub invalidation_policies: &'a mut HashSet<CacheInvalidation>,
}

impl<'ctx, 'a> Visitor<'ctx> for CacheControlCalculate<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(&mut self, ctx: &mut VisitorContext<'_>, _selection_set: &Positioned<SelectionSet>) {
        if let Some(MetaType::Object(object)) = ctx.current_type() {
            self.cache_control.merge(object.cache_control.clone());

            if let Some(policy) = &object.cache_control.invalidation_policy {
                let ty = object.rust_typename.to_string();
                self.invalidation_policies.insert(CacheInvalidation {
                    ty,
                    policy: policy.clone(),
                });
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_>, field: &Positioned<Field>) {
        if let Some((registry_field, parent_type)) = ctx.parent_type().and_then(|parent| {
            parent
                .field_by_name(&field.node.name.node)
                .map(|field| (field, parent.name()))
        }) {
            self.cache_control.merge(registry_field.cache_control.clone());

            if let Some(policy) = &registry_field.cache_control.invalidation_policy {
                self.invalidation_policies.insert(CacheInvalidation {
                    ty: parent_type.to_string(),
                    policy: policy.clone(),
                });
            }
        }
    }
}
