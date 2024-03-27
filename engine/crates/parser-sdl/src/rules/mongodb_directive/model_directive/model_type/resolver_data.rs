use engine::{
    indexmap::IndexMap,
    registry::{
        resolvers::{custom::CustomResolver, transformer::Transformer, Resolver},
        FieldSet, MetaInputValue,
    },
    CacheControl, Pos,
};
use engine_parser::types::FieldDefinition;

use crate::{
    parser_extensions::FieldExtension,
    rules::{
        cache_directive::CacheDirective,
        join_directive::{self, JoinDirective},
        requires_directive::RequiresDirective,
        resolver_directive::ResolverDirective,
        visitor::VisitorContext,
    },
};

#[derive(Debug)]
pub(super) struct ResolverData {
    pub(super) resolver: Resolver,
    pub(super) args: IndexMap<String, MetaInputValue>,
    pub(super) field_type: String,
    pub(super) cache_control: CacheControl,
    pub(super) requires: Option<FieldSet>,
}

impl ResolverData {
    pub(super) fn from_field(field: &FieldDefinition, visitor_ctx: &mut VisitorContext<'_>, position: Pos) -> Self {
        let resolver_name = ResolverDirective::resolver_name(field);
        let join_directive = JoinDirective::from_directives(&field.directives, visitor_ctx);

        match (resolver_name, join_directive) {
            (Some(resolver_name), None) => Self::resolver(resolver_name, field, visitor_ctx),
            (None, Some(join_directive)) => Self::join(field, join_directive),
            (Some(_), Some(_)) => {
                visitor_ctx.report_error(vec![position], "A field can't have a join and a custom resolver on it");

                Self::projection(field)
            }
            (None, None) => Self::projection(field),
        }
    }

    fn resolver(resolver_name: &str, field: &FieldDefinition, visitor_ctx: &mut VisitorContext<'_>) -> Self {
        let resolver = Resolver::CustomResolver(CustomResolver {
            resolver_name: resolver_name.to_owned(),
        });

        let field_type = field.ty.node.to_string();
        let cache_control = CacheDirective::parse(&field.directives);

        let args = field.converted_arguments();

        let requires =
            RequiresDirective::from_directives(&field.directives, visitor_ctx).map(RequiresDirective::into_fields);

        Self {
            resolver,
            args,
            field_type,
            cache_control,
            requires,
        }
    }

    fn join(field: &FieldDefinition, directive: JoinDirective) -> Self {
        ResolverData {
            resolver: Resolver::Join(directive.select.to_join_resolver()),
            args: field.converted_arguments(),
            cache_control: CacheDirective::parse(&field.directives),
            field_type: field.ty.node.to_string(),
            requires: directive.select.required_fieldset(&field.arguments),
        }
    }

    fn projection(field: &FieldDefinition) -> Self {
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
            requires: None,
        }
    }
}
