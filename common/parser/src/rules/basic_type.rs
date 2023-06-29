//! For basic types
//!
//! When a basic type is stubble uppon on the definition of the schema, if it
//! got no specialized behavior, we apply this behavior uppon it.
//!
use super::model_directive::MODEL_DIRECTIVE;
use super::visitor::{Visitor, VisitorContext};
use crate::registry::add_input_type_non_primitive;
use crate::rules::cache_directive::CacheDirective;

use dynaql::registry::plan::SchemaPlan;
use dynaql::registry::transformers::Transformer;
use dynaql::registry::{self, MetaField};
use dynaql_parser::types::TypeKind;
use if_chain::if_chain;

pub struct BasicType;

impl<'a> Visitor<'a> for BasicType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            if !["Query", "Mutation"].contains(&type_definition.node.name.node.as_str());
            if !directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
            if let TypeKind::Object(object) = &type_definition.node.kind;
            then {
                let type_name = type_definition.node.name.node.to_string();
                // If it's a modeled Type, we create the associated type into the registry.
                // Without more data, we infer it's from our modelization.
                ctx.registry.get_mut().create_type(|_| registry::ObjectType::new(
                    type_name.clone(),
                    object.fields.iter().map(|field| {
                        let name = field.node.name.node.to_string();
                        MetaField {
                            name: name.clone(),
                            description: field.node.description.clone().map(|x| x.node),
                            args: Default::default(),
                            ty: field.node.ty.clone().node.to_string().into(),
                            deprecation: Default::default(),
                            cache_control: CacheDirective::parse(&field.node.directives),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                            resolve: None,
                            edges: Vec::new(),
                            relation: None,
                            transformer: Some(Transformer::JSONSelect {
                                property: name.clone()
                            }),
                            plan: Some(SchemaPlan::projection(vec![name], true)),
                            required_operation: None,
                            auth: None,
                        }
                    })
                )
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .with_cache_control(CacheDirective::parse(&type_definition.node.directives))
                    .into()
                ,&type_name,
                &type_name
                );

                // If the type is a non primitive and also not modelized, it means we need to
                // create the Input version of it.
                // If the input is non used by other queries/mutation, it'll be removed from the
                // final schema.
                add_input_type_non_primitive(ctx, object, &type_name);
            }
        }
    }
}
