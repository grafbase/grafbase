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
        match ctx.current_type() {
            Some(MetaType::Interface(crate::registry::InterfaceType {
                cache_control: Some(cache_control),
                possible_types,
                name,
                ..
            })) => {
                self.cache_control.merge(*cache_control.clone());
                if let Some(policy) = &cache_control.invalidation_policy {
                    self.invalidation_policies.insert(CacheInvalidation {
                        ty: name.to_string(),
                        policy: policy.clone(),
                    });

                    possible_types.iter().for_each(|possible_type| {
                        self.invalidation_policies.insert(CacheInvalidation {
                            ty: possible_type.to_string(),
                            policy: policy.clone(),
                        });
                    });
                }
            }
            Some(MetaType::Object(crate::registry::ObjectType {
                cache_control: Some(cache_control),
                rust_typename,
                ..
            })) => {
                self.cache_control.merge(*cache_control.clone());
                if let Some(policy) = &cache_control.invalidation_policy {
                    let ty = rust_typename.to_string();
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
        if let Some((registry_field, parent_type)) = ctx.parent_type().and_then(|parent| {
            parent
                .field_by_name(&field.node.name.node)
                .map(|field| (field, parent.name()))
        }) {
            if let Some(cache_control) = &registry_field.cache_control {
                self.cache_control.merge(*cache_control.clone());

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
