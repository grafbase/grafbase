use std::str::FromStr;

use dynaql::indexmap::IndexMap;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::resolvers::query::{search, QueryResolver, MATCHING_RECORDS_ID_KEY};
use dynaql::registry::MetaTypeName;
use dynaql::registry::{
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::{AuthConfig, Operations, Positioned};
use dynaql_parser::types::{ConstDirective, FieldDefinition, TypeDefinition};
use itertools::Itertools;

use crate::registry::names::MetaNames;
use crate::rules::search_directive::SEARCH_DIRECTIVE;
use crate::rules::visitor::VisitorContext;

const SEARCH_INPUT_ARG_QUERY: &str = "query";
const SEARCH_INPUT_ARG_LIMIT: &str = "limit";

fn convert_to_search_scalar(ty: &str) -> Result<search::Scalar, String> {
    match MetaTypeName::create(ty) {
        MetaTypeName::List(type_name) | MetaTypeName::NonNull(type_name) => convert_to_search_scalar(type_name),
        MetaTypeName::Named(type_name) => search::Scalar::from_str(type_name).map_err(|_| type_name.to_string()),
    }
}

pub fn add_query_search(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
    search_fields: Vec<(&FieldDefinition, &Positioned<ConstDirective>)>,
) {
    assert!(!search_fields.is_empty());
    let type_name = MetaNames::model(model_type_definition);
    let field_name = MetaNames::search(model_type_definition);

    let (fields, errors): (Vec<_>, Vec<_>) = search_fields
        .into_iter()
        .map(|(field, directive)| {
            convert_to_search_scalar(&field.ty.node.to_string())
                .map(|scalar| search::Field {
                    name: field.name.node.to_string(),
                    scalar,
                })
                .map_err(|unsupported_type_name| {
                    ctx.report_error(
                        vec![directive.pos],
                        format!(
                            "The @{SEARCH_DIRECTIVE} directive cannot be used with the {unsupported_type_name} type."
                        ),
                    );
                })
        })
        .partition_result();
    let search_schema = if errors.is_empty() {
        search::Schema { fields }
    } else {
        return;
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
