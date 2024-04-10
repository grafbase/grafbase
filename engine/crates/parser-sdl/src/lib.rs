#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::collections::{HashMap, HashSet};

use common_types::UdfKind;
use engine::{
    registry::{
        enums::EngineEnums,
        scalars::{PossibleScalar, SDLDefinitionScalar},
    },
    Pos,
};
use engine_parser::{types::ServiceDocument, Error as ParserError};
use rules::{
    all_subgraphs_directive::{AllSubgraphsDirective, AllSubgraphsDirectiveVisitor},
    auth_directive::{v2::AuthV2DirectiveVisitor, AuthDirective},
    basic_type::BasicType,
    check_field_lowercase::CheckFieldCamelCase,
    check_known_directives::CheckAllDirectivesAreKnown,
    check_type_validity::CheckTypeValidity,
    check_types_underscore::CheckBeginsWithDoubleUnderscore,
    connector_transforms::run_transforms,
    default_directive::DefaultDirective,
    default_directive_types::DefaultDirectiveTypes,
    deprecated_directive::DeprecatedDirective,
    directive::Directives,
    enum_type::EnumType,
    extend_connector_types::ExtendConnectorTypes,
    extend_field::{ExtendFieldDirective, ExtendFieldVisitor},
    extend_query_and_mutation_types::ExtendQueryAndMutationTypes,
    federation::{
        ExternalDirective, FederationDirective, FederationDirectiveVisitor, InaccessibleDirective, KeyDirective,
        OverrideDirective, ProvidesDirective, ShareableDirective, TagDirective,
    },
    graph_directive::GraphVisitor,
    graphql_directive::GraphqlVisitor,
    input_object::InputObjectVisitor,
    interface::Interface,
    introspection::{IntrospectionDirective, IntrospectionDirectiveVisitor},
    join_directive::JoinDirective,
    length_directive::LengthDirective,
    map_directive::MapDirective,
    model_directive::ModelDirective,
    mongodb_directive::{MongoDBModelDirective, MongoDBTypeDirective},
    one_of_directive::OneOfDirective,
    openapi_directive::OpenApiVisitor,
    operation_limits_directive::{OperationLimitsDirective, OperationLimitsVisitor},
    postgres_directive::PostgresVisitor,
    requires_directive::RequiresDirective,
    resolver_directive::ResolverDirective,
    search_directive::SearchDirective,
    subgraph_directive::{SubgraphDirective, SubgraphDirectiveVisitor},
    trusted_documents_directive::{TrustedDocumentsDirective, TrustedDocumentsVisitor},
    unique_directive::UniqueDirective,
    unique_fields::UniqueObjectFields,
    visitor::{visit, RuleError, Visitor, VisitorContext},
};

pub mod federation;
mod parser_extensions;
mod schema_coord;
mod type_names;
mod validations;

pub use connector_parsers::ConnectorParsers;
pub use engine::registry::Registry;
pub use registry::names::*;
pub use rules::{
    auth_directive::v2::{AuthV2Directive, AuthV2Provider, Jwks, JwtTokenHeader},
    cache_directive::global::{GlobalCacheRules, GlobalCacheTarget},
    graph_directive::GraphDirective,
    graphql_directive::GraphqlDirective,
    mongodb_directive::MongoDBDirective,
    openapi_directive::{OpenApiDirective, OpenApiQueryNamingStrategy, OpenApiTransforms},
    postgres_directive::PostgresDirective,
};
use validations::post_parsing_validations;

use crate::rules::{
    cache_directive::{visitor::CacheVisitor, CacheDirective},
    experimental::{ExperimentalDirective, ExperimentalDirectiveVisitor},
    mongodb_directive::MongoDBVisitor,
    scalar_hydratation::ScalarHydratation,
};
pub use dynamic_string::DynamicString;

pub mod connector_parsers;
pub mod usage;

mod directive_de;
mod dynamic_string;
mod registry;
mod rules;
#[cfg(test)]
mod tests;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Parser(
        #[from]
        #[source]
        ParserError,
    ),
    #[error("{0:?}")]
    Validation(Vec<RuleError>),
    #[error("{} data source: {}", .0, .1.join("\n"))]
    ConnectorErrors(String, Vec<String>, Pos),
}

impl From<Vec<RuleError>> for Error {
    fn from(value: Vec<RuleError>) -> Self {
        Error::Validation(value)
    }
}

impl Error {
    #[cfg(test)]
    fn validation_errors(self) -> Option<Vec<RuleError>> {
        if let Error::Validation(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub registry: Registry,
    pub required_udfs: HashSet<(UdfKind, String)>,
    pub global_cache_rules: GlobalCacheRules<'a>,
    pub federated_graph_config: Option<federation::FederatedGraphConfig>,
}

fn parse_schema(schema: &str) -> engine::parser::Result<ServiceDocument> {
    let directives = Directives::new()
        .with::<AllSubgraphsDirective>()
        .with::<AuthDirective>()
        .with::<AuthV2Directive>()
        .with::<CacheDirective>()
        .with::<DefaultDirective>()
        .with::<DeprecatedDirective>()
        .with::<ExperimentalDirective>()
        .with::<ExtendFieldDirective>()
        .with::<ExternalDirective>()
        .with::<FederationDirective>()
        .with::<GraphDirective>()
        .with::<GraphqlDirective>()
        .with::<InaccessibleDirective>()
        .with::<JoinDirective>()
        .with::<KeyDirective>()
        .with::<LengthDirective>()
        .with::<MapDirective>()
        .with::<ModelDirective>()
        .with::<MongoDBDirective>()
        .with::<OneOfDirective>()
        .with::<OpenApiDirective>()
        .with::<OperationLimitsDirective>()
        .with::<TrustedDocumentsDirective>()
        .with::<OverrideDirective>()
        .with::<PostgresDirective>()
        .with::<ProvidesDirective>()
        .with::<RequiresDirective>()
        .with::<ResolverDirective>()
        .with::<SearchDirective>()
        .with::<ShareableDirective>()
        .with::<SubgraphDirective>()
        .with::<TagDirective>()
        .with::<UniqueDirective>()
        .with::<IntrospectionDirective>();

    let schema = format!(
        "{}\n{}\n{}\n{}",
        schema,
        EngineEnums::sdl(),
        PossibleScalar::sdl(),
        directives.to_definition(),
    );

    engine::parser::parse_schema(schema)
}

/// Transform the input into a Registry
pub async fn parse<'a>(
    schema: &'a str,
    environment_variables: &HashMap<String, String>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<ParseResult<'a>, Error> {
    let schema = parse_schema(schema)?;
    let mut ctx = VisitorContext::new(&schema, environment_variables);

    // We parse out the basic things like `extend schema @graph`.
    parse_basic(&schema, &mut ctx).await?;

    // We parse out connectors (and run their sub-parsers) first so that our schema
    // can reference types generated by those connectors
    parse_connectors(&schema, &mut ctx, connector_parsers).await?;

    parse_types(&schema, &mut ctx);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    parse_post_types(&schema, &mut ctx);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    if !ctx.warnings.is_empty() {
        println!("{}", ctx.warnings);
    }

    let result = ctx.finish();
    let post_parsing_errors = post_parsing_validations(&result.registry);

    if !post_parsing_errors.is_empty() {
        return Err(post_parsing_errors.into());
    }

    Ok(result)
}

async fn parse_basic<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) -> Result<(), Error> {
    let mut connector_rules = rules::visitor::VisitorNil
        .with(GraphVisitor)
        .with(OperationLimitsVisitor)
        .with(TrustedDocumentsVisitor);

    visit(&mut connector_rules, ctx, schema);

    Ok(())
}

async fn parse_connectors<'a>(
    schema: &'a ServiceDocument,
    ctx: &mut VisitorContext<'a>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<(), Error> {
    let mut connector_rules = rules::visitor::VisitorNil
        .with(OpenApiVisitor)
        .with(GraphqlVisitor)
        .with(MongoDBVisitor)
        .with(PostgresVisitor);

    visit(&mut connector_rules, ctx, schema);

    validate_unique_names(ctx)?;

    // We could probably parallelise this, but the schemas and the associated
    // processing use a reasonable amount of memory so going to keep it sequential
    for (directive, position) in std::mem::take(&mut ctx.openapi_directives) {
        let directive_name = directive.name.clone();
        let transforms = directive.transforms.transforms.clone();
        match connector_parsers.fetch_and_parse_openapi(directive).await {
            Ok(mut registry) => {
                if let Some(transforms) = &transforms {
                    run_transforms(&mut registry, transforms);
                }
                connector_parsers::merge_registry(ctx, registry, position);
            }
            Err(errors) => return Err(Error::ConnectorErrors(directive_name, errors, position)),
        }
    }

    for (directive, position) in std::mem::take(&mut ctx.graphql_directives) {
        let directive_name = directive.name.clone();
        let transforms = directive.transforms.clone();
        match connector_parsers.fetch_and_parse_graphql(directive).await {
            Ok(mut registry) => {
                if let Some(transforms) = &transforms {
                    run_transforms(&mut registry, transforms);
                }
                connector_parsers::merge_registry(ctx, registry, position);
            }
            Err(errors) => return Err(Error::ConnectorErrors(directive_name, errors, position)),
        }
    }

    for (directive, position) in std::mem::take(&mut ctx.postgres_directives) {
        match connector_parsers.fetch_and_parse_postgres(&directive).await {
            Ok(registry) => {
                connector_parsers::merge_registry(ctx, registry, position);
            }
            Err(errors) => return Err(Error::ConnectorErrors(directive.name().to_string(), errors, position)),
        }
    }

    Ok(())
}

fn validate_unique_names(ctx: &VisitorContext<'_>) -> Result<(), Error> {
    let mut names = HashMap::new();

    for (directive, position) in &ctx.openapi_directives {
        let names = names.entry(directive.name.as_str()).or_insert(Vec::new());
        names.push(*position);
    }

    for (directive, position) in &ctx.graphql_directives {
        let names = names.entry(directive.name.as_str()).or_insert(Vec::new());
        names.push(*position);
    }

    for (directive, position) in &ctx.mongodb_directives {
        let names = names.entry(directive.name()).or_insert(Vec::new());
        names.push(*position);
    }

    for (directive, position) in &ctx.postgres_directives {
        let names = names.entry(directive.name()).or_insert(Vec::new());
        names.push(*position);
    }

    let mut errors = Vec::new();

    for (name, locations) in names {
        if locations.len() < 2 {
            continue;
        }

        errors.push(RuleError {
            locations,
            message: format!(r#"Name "{name}" is not unique. A connector must have a unique name."#),
        });
    }

    if !errors.is_empty() {
        return Err(Error::Validation(errors));
    }

    Ok(())
}

fn parse_types<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    let mut types_definitions_rules = rules::visitor::VisitorNil
        .with(ModelDirective)
        .with(AuthDirective)
        .with(AuthV2DirectiveVisitor)
        .with(ResolverDirective)
        .with(CacheVisitor)
        .with(InputObjectVisitor)
        .with(BasicType)
        .with(Interface)
        .with(ExtendQueryAndMutationTypes)
        .with(EnumType)
        .with(ScalarHydratation)
        .with(MongoDBTypeDirective)
        .with(MongoDBModelDirective)
        .with(LengthDirective)
        .with(CheckAllDirectivesAreKnown::default())
        .with(ExperimentalDirectiveVisitor)
        .with(FederationDirectiveVisitor) // This will likely need moved.  Here'll do for now though
        .with(ExtendFieldVisitor)
        .with(SubgraphDirectiveVisitor)
        .with(AllSubgraphsDirectiveVisitor)
        .with(IntrospectionDirectiveVisitor);

    visit(&mut types_definitions_rules, ctx, schema);

    let mut extend_types_rules = rules::visitor::VisitorNil.with(ExtendConnectorTypes);

    visit(&mut extend_types_rules, ctx, schema);
}

/// Visitors that require all user-defined types to be parsed already.
fn parse_post_types<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    let mut rules = rules::visitor::VisitorNil
        .with(UniqueObjectFields)
        .with(CheckBeginsWithDoubleUnderscore)
        .with(CheckFieldCamelCase)
        .with(CheckTypeValidity)
        .with(DefaultDirectiveTypes)
        .with(SearchDirective);

    visit(&mut rules, ctx, schema);
}

pub fn parse_registry<S: AsRef<str>>(input: S) -> Result<Registry, Error> {
    let input = input.as_ref();
    Ok(futures::executor::block_on(async move {
        let variables = HashMap::new();
        let connector_parsers = connector_parsers::MockConnectorParsers::default();
        parse(input, &variables, &connector_parsers).await
    })?
    .registry)
}

#[cfg(test)]
fn to_parse_result_with_variables<'a>(
    input: &'a str,
    variables: &HashMap<String, String>,
) -> Result<ParseResult<'a>, Error> {
    futures::executor::block_on(async move {
        let connector_parsers = connector_parsers::MockConnectorParsers::default();
        parse(input, variables, &connector_parsers).await
    })
}
