use registry_v2::MetaType;
use std::collections::HashSet;

use crate::{
    parser::types::{Field, SelectionSet},
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    CacheControl, CacheInvalidation, Positioned,
};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut registry_v2::cache_control::CacheControl,
    pub invalidation_policies: &'a mut HashSet<CacheInvalidation>,
}

impl<'ctx, 'a> Visitor<'ctx> for CacheControlCalculate<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(&mut self, ctx: &mut VisitorContext<'_>, _selection_set: &Positioned<SelectionSet>) {
        match ctx.current_type() {
            Some(MetaType::Interface(interface)) if interface.cache_control().is_some() => {
                let cache_control = interface.cache_control().unwrap();
                self.cache_control.merge(cache_control.clone());
                if let Some(policy) = &cache_control.invalidation_policy {
                    self.invalidation_policies.insert(CacheInvalidation {
                        ty: interface.name().to_string(),
                        policy: policy.clone(),
                    });

                    interface.possible_types().for_each(|possible_type| {
                        self.invalidation_policies.insert(CacheInvalidation {
                            ty: possible_type.name().to_string(),
                            policy: policy.clone(),
                        });
                    });
                }
            }
            Some(MetaType::Object(object)) if object.cache_control().is_some() => {
                let cache_control = object.cache_control().unwrap();
                self.cache_control.merge(cache_control.clone());
                if let Some(policy) = &cache_control.invalidation_policy {
                    let ty = object.name().to_string();
                    self.invalidation_policies.insert(CacheInvalidation {
                        ty,
                        policy: policy.clone(),
                    });
                }
            }
            _ => {}
        };
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_>, field: &Positioned<Field>) {
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
