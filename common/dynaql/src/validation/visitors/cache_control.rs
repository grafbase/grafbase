use crate::parser::types::{Field, SelectionSet};
use crate::registry::MetaType;
use crate::validation::visitor::{VisitMode, Visitor, VisitorContext};
use crate::{CacheControl, CacheInvalidation, Positioned};
use std::collections::HashSet;

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut CacheControl,
    pub invalidation_policies: &'a mut HashSet<CacheInvalidation>,
}

impl<'ctx, 'a> Visitor<'ctx> for CacheControlCalculate<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'_>,
        _selection_set: &Positioned<SelectionSet>,
    ) {
        if let Some(MetaType::Object {
            cache_control,
            rust_typename,
            ..
        }) = ctx.current_type()
        {
            self.cache_control.merge(cache_control.clone());

            if let Some(policy) = &cache_control.invalidation_policy {
                self.invalidation_policies.insert(CacheInvalidation {
                    ty: rust_typename.to_string(),
                    policy: policy.clone(),
                });
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_>, field: &Positioned<Field>) {
        if let Some(registry_field) = ctx
            .parent_type()
            .and_then(|parent| parent.field_by_name(&field.node.name.node))
        {
            self.cache_control
                .merge(registry_field.cache_control.clone());

            if let Some(policy) = &registry_field.cache_control.invalidation_policy {
                self.invalidation_policies.insert(CacheInvalidation {
                    ty: registry_field.ty.to_string(),
                    policy: policy.clone(),
                });
            }
        }
    }
}
