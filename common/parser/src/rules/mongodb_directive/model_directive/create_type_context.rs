use crate::{
    rules::{
        cache_directive::CacheDirective, model_directive::MODEL_DIRECTIVE, unique_directive::UniqueDirective,
        visitor::VisitorContext,
    },
    MongoDBDirective,
};
use dynaql::{registry::Constraint, AuthConfig, CacheControl, Positioned};
use dynaql_parser::types::{FieldDefinition, ObjectType, TypeDefinition};

use super::COLLECTION_KEY;

pub(super) struct CreateTypeContext<'a> {
    pub(super) r#type: &'a Positioned<TypeDefinition>,
    pub(super) object: &'a ObjectType,
    type_name: &'a str,
    model_cache: CacheControl,
    model_auth: Option<AuthConfig>,
    directive: MongoDBDirective,
    collection: String,
    unique_directives: Vec<UniqueDirective>,
}

impl<'a> CreateTypeContext<'a> {
    pub(super) fn new(
        visitor_ctx: &mut VisitorContext<'_>,
        object: &'a ObjectType,
        model_auth: Option<AuthConfig>,
        r#type: &'a Positioned<TypeDefinition>,
        directive: MongoDBDirective,
    ) -> Self {
        let model_cache = CacheDirective::parse(&r#type.node.directives);

        let collection = r#type
            .node
            .directives
            .iter()
            .filter(|directive| directive.node.name.node == MODEL_DIRECTIVE)
            .filter_map(|directive| directive.node.get_argument(COLLECTION_KEY))
            .find_map(|argument| argument.node.as_str())
            .unwrap_or_else(|| r#type.node.name.as_str())
            .to_string();

        let type_name = r#type.node.name.node.as_str();

        let unique_directives = object
            .fields
            .iter()
            .filter_map(|field| UniqueDirective::parse(visitor_ctx, object, type_name, field))
            .collect();

        Self {
            r#type,
            object,
            type_name,
            model_cache,
            model_auth,
            directive,
            collection,
            unique_directives,
        }
    }

    pub(super) fn type_name(&self) -> &str {
        self.type_name
    }

    pub(super) fn type_description(&self) -> Option<&str> {
        self.r#type
            .node
            .description
            .as_ref()
            .map(|description| description.node.as_str())
    }

    pub(super) fn model_cache(&self) -> &CacheControl {
        &self.model_cache
    }

    pub(super) fn model_auth(&self) -> &Option<AuthConfig> {
        &self.model_auth
    }

    pub(super) fn fields(&self) -> impl ExactSizeIterator<Item = &FieldDefinition> + '_ {
        self.object.fields.iter().map(|field| &field.node)
    }

    pub(super) fn unique_directives(&self) -> impl ExactSizeIterator<Item = &UniqueDirective> + '_ {
        self.unique_directives.iter()
    }

    pub(super) fn unique_constraints(&self) -> impl ExactSizeIterator<Item = Constraint> + '_ {
        self.unique_directives().map(UniqueDirective::to_constraint)
    }

    pub(super) fn directive(&self) -> &MongoDBDirective {
        &self.directive
    }

    pub(super) fn collection(&self) -> &str {
        &self.collection
    }
}
