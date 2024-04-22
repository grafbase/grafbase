use case::CaseExt;
use engine::{registry::Constraint, AuthConfig, CacheControl, Positioned};
use engine_parser::types::{FieldDefinition, ObjectType, TypeDefinition};
use registry_v2::MongoDBConfiguration;

use super::COLLECTION_KEY;
use crate::rules::{cache_directive::CacheDirective, unique_directive::UniqueDirective, visitor::VisitorContext};

pub(crate) struct CreateTypeContext<'a> {
    pub(super) r#type: &'a Positioned<TypeDefinition>,
    pub(super) object: &'a ObjectType,
    model_name: &'a str,
    model_cache: Option<Box<CacheControl>>,
    model_auth: Option<Box<AuthConfig>>,
    collection: String,
    unique_directives: Vec<UniqueDirective>,
    config: MongoDBConfiguration,
    query_type_name: Option<String>,
    mutation_type_name: Option<String>,
}

impl<'a> CreateTypeContext<'a> {
    pub(crate) fn new(
        visitor_ctx: &mut VisitorContext<'_>,
        object: &'a ObjectType,
        model_auth: Option<AuthConfig>,
        r#type: &'a Positioned<TypeDefinition>,
        config: MongoDBConfiguration,
    ) -> Self {
        let model_cache = CacheDirective::parse(&r#type.node.directives);

        let collection = r#type
            .node
            .directives
            .iter()
            .filter(|directive| directive.is_model())
            .filter_map(|directive| directive.node.get_argument(COLLECTION_KEY))
            .find_map(|argument| argument.node.as_str())
            .unwrap_or_else(|| r#type.node.name.as_str())
            .to_string();

        let model_name = r#type.node.name.node.as_str();

        let unique_directives = object
            .fields
            .iter()
            .filter_map(|field| UniqueDirective::parse(visitor_ctx, object, model_name, field))
            .collect();

        let query_type_name = config.namespace.then(|| format!("{}Query", config.name).to_camel());
        let mutation_type_name = config.namespace.then(|| format!("{}Mutation", config.name).to_camel());

        Self {
            r#type,
            object,
            model_name,
            model_cache,
            model_auth: model_auth.map(Box::new),
            collection,
            unique_directives,
            config,
            query_type_name,
            mutation_type_name,
        }
    }

    pub(super) fn model_name(&self) -> &str {
        self.model_name
    }

    pub(super) fn type_description(&self) -> Option<&str> {
        self.r#type.description()
    }

    #[allow(clippy::borrowed_box)]
    pub(super) fn model_cache(&self) -> Option<&Box<CacheControl>> {
        self.model_cache.as_ref()
    }

    #[allow(clippy::borrowed_box)]
    pub(super) fn model_auth(&self) -> Option<&Box<AuthConfig>> {
        self.model_auth.as_ref()
    }

    pub(super) fn fields(&self) -> impl ExactSizeIterator<Item = &Positioned<FieldDefinition>> + '_ {
        self.object.fields.iter()
    }

    pub(super) fn unique_directives(&self) -> impl ExactSizeIterator<Item = &UniqueDirective> + '_ {
        self.unique_directives.iter()
    }

    pub(super) fn unique_constraints(&self) -> impl ExactSizeIterator<Item = Constraint> + '_ {
        self.unique_directives().map(UniqueDirective::to_constraint)
    }

    pub(super) fn config(&self) -> &MongoDBConfiguration {
        &self.config
    }

    pub(super) fn collection(&self) -> &str {
        &self.collection
    }

    pub(super) fn query_type_name(&self) -> Option<&str> {
        self.query_type_name.as_deref()
    }

    pub(super) fn mutation_type_name(&self) -> Option<&str> {
        self.mutation_type_name.as_deref()
    }
}
