use engine_parser::Positioned;
use registry_v2::MetaType;
use std::{collections::HashSet, thread::current};

use crate::{
    registries::{ValidationField, ValidationMetaType, ValidationRegistry},
    CacheInvalidation,
};

use {
    crate::visitor::{VisitMode, Visitor, VisitorContext},
    engine_parser::types::{Field, SelectionSet},
};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut registry_v2::cache_control::CacheControl,
    pub invalidation_policies: &'a mut HashSet<CacheInvalidation>,
}

impl<'ctx, 'a, Registry> Visitor<'ctx, Registry> for CacheControlCalculate<'a>
where
    Registry: ValidationRegistry,
{
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'_, Registry>,
        _selection_set: &Positioned<SelectionSet>,
    ) {
        let Some(current_type) = ctx.current_type() else { return };
        let Some(cache_control) = current_type.cache_control() else {
            return;
        };

        self.cache_control.merge(cache_control.clone());
        if let Some(policy) = &cache_control.invalidation_policy {
            self.invalidation_policies.insert(CacheInvalidation {
                ty: current_type.name().to_string(),
                policy: policy.clone(),
            });

            if let Some(possible_types) = current_type.possible_types() {
                possible_types.for_each(|possible_type| {
                    self.invalidation_policies.insert(CacheInvalidation {
                        ty: possible_type.name().to_string(),
                        policy: policy.clone(),
                    });
                });
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_, Registry>, field: &Positioned<Field>) {
        if let Some((registry_field, parent_type)) = ctx
            .parent_type()
            .and_then(|parent| parent.field(&field.node.name.node).map(|field| (field, parent.name())))
        {
            if let Some(cache_control) = registry_field.cache_control() {
                self.cache_control.merge(cache_control.clone());

                if let Some(policy) = &cache_control.invalidation_policy {
                    self.invalidation_policies.insert(CacheInvalidation {
                        ty: parent_type.to_string(),
                        policy: policy.clone(),
                    });
                }
            }
        }
    }
}
