use engine::{
    indexmap::IndexMap,
    registry::{
        resolvers::{custom::CustomResolver, transformer::Transformer, Resolver},
        MetaInputValue,
    },
    CacheControl,
};
use engine_parser::types::FieldDefinition;

use crate::rules::cache_directive::CacheDirective;

#[derive(Debug)]
pub(super) struct ResolverData {
    pub(super) resolver: Resolver,
    pub(super) args: IndexMap<String, MetaInputValue>,
    pub(super) field_type: String,
    pub(super) cache_control: CacheControl,
}

impl ResolverData {
    pub(super) fn resolver(resolver_name: &str, field: &FieldDefinition) -> Self {
        let resolver = Resolver::CustomResolver(CustomResolver {
            resolver_name: resolver_name.to_owned(),
        });

        let field_type = field.ty.node.to_string();
        let cache_control = CacheDirective::parse(&field.directives);

        let args = field
            .arguments
            .iter()
            .map(|argument| {
                let name = argument.node.name.to_string();
                let input = MetaInputValue::new(argument.node.name.to_string(), argument.node.ty.to_string());

                (name, input)
            })
            .collect();

        Self {
            resolver,
            args,
            field_type,
            cache_control,
        }
    }

    pub(super) fn projection(field: &FieldDefinition) -> Self {
        let key = field
            .mapped_name()
            .map(ToString::to_string)
            .unwrap_or_else(|| field.name().to_string());

        let mut resolver = Resolver::Transformer(Transformer::Select { key });

        if let "Timestamp" = field.ty.base.to_base_type_str() {
            resolver = resolver.and_then(Transformer::MongoTimestamp);
        }

        let field_type = field.ty.node.to_string();
        let cache_control = CacheDirective::parse(&field.directives);

        ResolverData {
            resolver,
            args: IndexMap::new(),
            field_type,
            cache_control,
        }
    }
}
