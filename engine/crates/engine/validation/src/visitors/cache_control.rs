use engine_parser::Positioned;
use registry_v2::cache_control::CacheControl;
use std::collections::HashSet;

use crate::{
    registries::{AnyField, AnyMetaType, AnyRegistry},
    CacheInvalidation,
};

use {
    crate::visitor::{VisitMode, Visitor, VisitorContext},
    engine_parser::types::{Field, SelectionSet},
};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut CacheControl,
    pub invalidation_policies: &'a mut HashSet<CacheInvalidation>,
    pub cache_control_stack: Vec<CacheControl>,
    default_cache_control: CacheControl,
}

impl<'a> CacheControlCalculate<'a> {
    pub fn new(cache_control: &'a mut CacheControl, invalidation_policies: &'a mut HashSet<CacheInvalidation>) -> Self {
        CacheControlCalculate {
            cache_control,
            invalidation_policies,
            cache_control_stack: vec![],
            default_cache_control: CacheControl::default(),
        }
    }
}

impl<'ctx, 'a, Registry> Visitor<'ctx, Registry> for CacheControlCalculate<'a>
where
    Registry: AnyRegistry,
{
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'_, Registry>,
        _selection_set: &Positioned<SelectionSet>,
    ) {
        let cache_control = match ctx.current_type().and_then(|ty| ty.cache_control()) {
            Some(cache_control) => {
                self.cache_control_stack.push(cache_control.clone());
                cache_control
            }
            None if self.cache_control_stack.is_empty() => &self.default_cache_control,
            None => self.cache_control_stack.last().unwrap(),
        };

        self.cache_control.merge(cache_control.clone());
        if let Some(policy) = &cache_control.invalidation_policy {
            let Some(current_type) = ctx.current_type() else { return };

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
            };
        }
    }

    fn exit_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'ctx, Registry>,
        _selection_set: &'ctx Positioned<SelectionSet>,
    ) {
        if ctx.current_type().and_then(|ty| ty.cache_control()).is_some() {
            self.cache_control_stack.pop();
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_, Registry>, field: &Positioned<Field>) {
        let cache_control = match ctx
            .parent_type()
            .and_then(|parent| parent.field(&field.node.name.node)?.cache_control())
        {
            Some(cache_control) => {
                self.cache_control_stack.push(cache_control.clone());
                cache_control
            }
            None if self.cache_control_stack.is_empty() => &self.default_cache_control,
            None => self.cache_control_stack.last().unwrap(),
        };

        self.cache_control.merge(cache_control.clone());

        if let Some(policy) = &cache_control.invalidation_policy {
            let Some(parent_type) = ctx.parent_type() else { return };
            self.invalidation_policies.insert(CacheInvalidation {
                ty: parent_type.name().to_string(),
                policy: policy.clone(),
            });
        }
    }

    fn exit_field(&mut self, ctx: &mut VisitorContext<'ctx, Registry>, field: &'ctx Positioned<Field>) {
        if ctx
            .parent_type()
            .and_then(|parent| parent.field(&field.node.name.node)?.cache_control())
            .is_some()
        {
            self.cache_control_stack.pop();
        }
    }
}
