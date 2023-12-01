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
    auth_directive::AuthDirective,
    basic_type::BasicType,
    check_field_lowercase::CheckFieldCamelCase,
    check_known_directives::CheckAllDirectivesAreKnown,
    check_type_collision::CheckTypeCollision,
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
    join_directive::JoinDirective,
    length_directive::LengthDirective,
    map_directive::MapDirective,
    model_directive::ModelDirective,
    mongodb_directive::{MongoDBModelDirective, MongoDBTypeDirective},
    one_of_directive::OneOfDirective,
    openapi_directive::OpenApiVisitor,
    postgres_directive::PostgresVisitor,
    relations::{relations_rules, RelationEngine},
    requires_directive::RequiresDirective,
    resolver_directive::ResolverDirective,
    search_directive::SearchDirective,
    unique_directive::UniqueDirective,
    unique_fields::UniqueObjectFields,
    visitor::{visit, RuleError, Visitor, VisitorContext},
};

mod type_names;
mod validations;

pub use connector_parsers::ConnectorParsers;
pub use engine::registry::Registry;
pub use migration_detection::{required_migrations, RequiredMigration};
pub use registry::names::*;
pub use rules::{
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

pub mod connector_parsers;
pub mod usage;

mod directive_de;
mod dynamic_string;
mod migration_detection;
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
}

fn parse_schema(schema: &str) -> engine::parser::Result<ServiceDocument> {
    let directives = Directives::new()
        .with::<AuthDirective>()
        .with::<DefaultDirective>()
        .with::<MapDirective>()
        .with::<LengthDirective>()
        .with::<ModelDirective>()
        .with::<OneOfDirective>()
        .with::<RelationEngine>()
        .with::<ResolverDirective>()
        .with::<UniqueDirective>()
        .with::<SearchDirective>()
        .with::<OpenApiDirective>()
        .with::<GraphDirective>()
        .with::<GraphqlDirective>()
        .with::<CacheDirective>()
        .with::<MongoDBDirective>()
        .with::<PostgresDirective>()
        .with::<ExperimentalDirective>()
        .with::<FederationDirective>()
        .with::<RequiresDirective>()
        .with::<KeyDirective>()
        .with::<JoinDirective>()
        .with::<ExternalDirective>()
        .with::<ShareableDirective>()
        .with::<OverrideDirective>()
        .with::<ProvidesDirective>()
        .with::<DeprecatedDirective>()
        .with::<InaccessibleDirective>()
        .with::<TagDirective>()
        .with::<ExtendFieldDirective>();

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
    database_models_enabled: bool,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<ParseResult<'a>, Error> {
    let schema = parse_schema(schema)?;
    let mut ctx = VisitorContext::new(&schema, database_models_enabled, environment_variables);

    // We parse out the basic things like `extend schema @graph`.
    parse_basic(&schema, &mut ctx).await?;

    // We parse out connectors (and run their sub-parsers) first so that our schema
    // can reference types generated by those connectors
    parse_connectors(&schema, &mut ctx, connector_parsers).await?;

    // Building all relations first are it requires to parse the whole schema (for ManyToMany). This allows later
    // rules to rely on RelationEngine::get to have correct information on relations.
    parse_relations(&schema, &mut ctx);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

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
    let mut connector_rules = rules::visitor::VisitorNil.with(GraphVisitor);

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

fn parse_relations<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    visit(&mut relations_rules().with(CheckTypeCollision::default()), ctx, schema);
}

fn parse_types<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    let mut rules = rules::visitor::VisitorNil
        .with(CheckBeginsWithDoubleUnderscore)
        .with(CheckFieldCamelCase)
        .with(CheckTypeValidity)
        .with(ModelDirective)
        .with(AuthDirective)
        .with(ResolverDirective)
        .with(CacheVisitor)
        .with(InputObjectVisitor)
        .with(BasicType)
        .with(ExtendQueryAndMutationTypes)
        .with(ExtendConnectorTypes)
        .with(EnumType)
        .with(ScalarHydratation)
        .with(MongoDBTypeDirective)
        .with(MongoDBModelDirective)
        .with(LengthDirective)
        .with(UniqueObjectFields)
        .with(CheckAllDirectivesAreKnown::default())
        .with(ExperimentalDirectiveVisitor)
        .with(FederationDirectiveVisitor) // This will likely need moved.  Here'll do for now though
        .with(ExtendFieldVisitor);

    visit(&mut rules, ctx, schema);
}

/// Visitors that require all user-defined types to be parsed already.
fn parse_post_types<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    let mut rules = rules::visitor::VisitorNil
        .with(DefaultDirectiveTypes)
        .with(SearchDirective);

    visit(&mut rules, ctx, schema);
}

pub fn parse_registry<S: AsRef<str>>(input: S) -> Result<Registry, Error> {
    let input = input.as_ref();
    Ok(futures::executor::block_on(async move {
        let variables = HashMap::new();
        let connector_parsers = connector_parsers::MockConnectorParsers::default();
        parse(input, &variables, true, &connector_parsers).await
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
        parse(input, variables, true, &connector_parsers).await
    })
}
