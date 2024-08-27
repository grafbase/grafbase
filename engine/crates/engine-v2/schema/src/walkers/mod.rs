use crate::{
    sources::{
        graphql::{GraphqlEndpointWalker, RetryConfig},
        IntrospectionMetadata,
    },
    Schema, StringId,
};

mod definition;
mod directives;
mod entity;
mod r#enum;
mod field;
mod field_set;
mod header;
mod input_object;
mod input_value;
mod interface;
mod object;
mod requires;
mod resolver;
mod scalar;
mod r#type;
mod union;

pub use definition::*;
pub use directives::*;
pub use entity::*;
pub use field::*;
pub use field_set::*;
pub use header::*;
pub use input_object::*;
pub use input_value::*;
pub use interface::*;
pub use object::*;
pub use r#enum::*;
pub use r#type::*;
pub use requires::*;
pub use resolver::*;
pub use scalar::*;
pub use union::*;

#[derive(Clone, Copy)]
pub struct SchemaWalker<'a, I = ()> {
    // 'item' instead of 'inner' to avoid confusion with TypeWalker.inner()
    pub(crate) item: I,
    pub(crate) schema: &'a Schema,
}

impl<'a, I> SchemaWalker<'a, I> {
    pub fn new(item: I, schema: &'a Schema) -> Self {
        Self { item, schema }
    }

    pub fn walk<Other>(&self, item: Other) -> SchemaWalker<'a, Other> {
        SchemaWalker {
            item,
            schema: self.schema,
        }
    }
}

impl<'a, Id: Copy> SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    // Clippy complains because it's ambiguous with AsRef. But AsRef doesn't allow us to add the 'a
    // lifetime. I could rename to `to_ref()` or `ref()`, but doesn't feel better than `as_ref()`.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a <Schema as std::ops::Index<Id>>::Output {
        &self.schema[self.item]
    }

    pub fn id(&self) -> Id {
        self.item
    }
}

impl<'a> SchemaWalker<'a, ()> {
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = DefinitionWalker<'a>> + 'a {
        let walker = *self;
        self.schema
            .graph
            .type_definitions
            .iter()
            .map(move |definition| walker.walk(*definition))
    }

    pub fn description_id(&self) -> Option<StringId> {
        self.schema.graph.description
    }

    pub fn introspection_metadata(&self) -> &'a IntrospectionMetadata {
        &self.schema.data_sources.introspection
    }

    pub fn query(&self) -> ObjectDefinitionWalker<'a> {
        self.walk(self.schema.graph.root_operation_types.query)
    }

    pub fn mutation(&self) -> Option<ObjectDefinitionWalker<'a>> {
        self.schema.graph.root_operation_types.mutation.map(|id| self.walk(id))
    }

    pub fn subscription(&self) -> Option<ObjectDefinitionWalker<'a>> {
        self.schema
            .graph
            .root_operation_types
            .subscription
            .map(|id| self.walk(id))
    }

    // See further up
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a Schema {
        self.schema
    }

    pub fn default_header_rules(self) -> impl ExactSizeIterator<Item = HeaderRuleWalker<'a>> {
        self.as_ref()
            .settings
            .default_header_rules
            .iter()
            .map(move |id| self.walk(*id))
    }

    pub fn retry_config(self) -> Option<RetryConfig> {
        self.settings.retry.map(|retry| RetryConfig {
            min_per_second: retry.min_per_second,
            ttl: retry.ttl,
            retry_percent: retry.retry_percent,
            retry_mutations: retry.retry_mutations,
        })
    }

    pub fn graphql_endpoints(&self) -> impl ExactSizeIterator<Item = GraphqlEndpointWalker<'_>> {
        (0..self.data_sources.graphql.endpoints.len()).map(|i| GraphqlEndpointWalker::new(i.into(), self))
    }
}

impl<'a> std::ops::Deref for SchemaWalker<'a, ()> {
    type Target = Schema;

    fn deref(&self) -> &'a Self::Target {
        self.schema
    }
}
