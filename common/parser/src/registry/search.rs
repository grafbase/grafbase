use dynaql::indexmap::IndexMap;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::resolvers::query::{QueryResolver, SearchField, SearchSchema, MATCHING_RECORDS_ID_KEY};
use dynaql::registry::{
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::{AuthConfig, Operations};
use dynaql_parser::types::{ConstDirective, FieldDefinition, TypeDefinition};

use crate::registry::names::MetaNames;
use crate::rules::visitor::VisitorContext;

const SEARCH_INPUT_ARG_QUERY: &str = "query";
const SEARCH_INPUT_ARG_LIMIT: &str = "limit";

pub fn add_query_search(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
    search_fields: Vec<(&FieldDefinition, &ConstDirective)>,
) {
    assert!(!search_fields.is_empty());
    let type_name = MetaNames::model(model_type_definition);
    let field_name = MetaNames::search(model_type_definition);

    let search_schema = SearchSchema {
        fields: search_fields
            .into_iter()
            .map(|(field, _directive)| SearchField {
                name: field.name.node.to_string(),
                r#type: field.ty.node.to_string(),
            })
            .collect(),
    };

    ctx.queries.push(MetaField {
        name: field_name,
        description: Some(format!("Search `{type_name}`")),
        args: IndexMap::from([
            (
                SEARCH_INPUT_ARG_QUERY.to_string(),
                MetaInputValue {
                    name: SEARCH_INPUT_ARG_QUERY.to_string(),
                    description: Some("Raw search query".to_string()),
                    ty: "String!".to_string(),
                    default_value: None,
                    visible: None,
                    validators: None,
                    is_secret: false,
                },
            ),
            (
                SEARCH_INPUT_ARG_LIMIT.to_string(),
                MetaInputValue {
                    name: SEARCH_INPUT_ARG_LIMIT.to_string(),
                    description: Some("Maximum number of documents to retrieve (defaults to 10)".to_string()),
                    ty: "Int!".to_string(),
                    default_value: Some(10.into()),
                    visible: None,
                    validators: None,
                    is_secret: false,
                },
            ),
        ]),
        ty: format!("[{type_name}!]!"),
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: dynaql::CacheControl {
            public: true,
            max_age: 0usize,
        },
        external: false,
        provides: None,
        requires: None,
        visible: None,
        compute_complexity: None,
        edges: vec![],
        relation: None,
        plan: None,
        resolve: Some(
            Resolver {
                id: None,
                r#type: ResolverType::Query(QueryResolver::Search {
                    query: VariableResolveDefinition::InputTypeName(SEARCH_INPUT_ARG_QUERY.to_string()),
                    limit: VariableResolveDefinition::InputTypeName(SEARCH_INPUT_ARG_LIMIT.to_string()),
                    r#type: type_name,
                    schema: search_schema,
                }),
            }
            .and_then(ResolverType::DynamoResolver(DynamoResolver::QueryIds {
                ids: VariableResolveDefinition::LocalData(MATCHING_RECORDS_ID_KEY.to_string()),
            })),
        ),
        transformer: None,
        required_operation: Some(Operations::LIST),
        auth: model_auth.cloned(),
    });
}
