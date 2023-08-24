#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::collections::{HashMap, HashSet};

use grafbase::UdfKind;
use grafbase_engine::{
    registry::{
        enums::GrafbaseEngineEnums,
        scalars::{PossibleScalar, SDLDefinitionScalar},
    },
    Pos,
};
use grafbase_engine_parser::{parse_schema, types::ServiceDocument, Error as ParserError};
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
    directive::Directives,
    enum_type::EnumType,
    extend_connector_types::ExtendConnectorTypes,
    extend_query_and_mutation_types::ExtendQueryAndMutationTypes,
    graphql_directive::GraphqlVisitor,
    input_object::InputObjectVisitor,
    length_directive::LengthDirective,
    map_directive::MapDirective,
    model_directive::ModelDirective,
    mongodb_directive::{MongoDBModelDirective, MongoDBTypeDirective},
    one_of_directive::OneOfDirective,
    openapi_directive::OpenApiVisitor,
    relations::{relations_rules, RelationEngine},
    resolver_directive::ResolverDirective,
    search_directive::SearchDirective,
    unique_directive::UniqueDirective,
    unique_fields::UniqueObjectFields,
    visitor::{visit, RuleError, Visitor, VisitorContext},
};

mod type_names;

pub use connector_parsers::ConnectorParsers;
pub use grafbase_engine::registry::Registry;
pub use migration_detection::{required_migrations, RequiredMigration};
pub use rules::{
    cache_directive::global::{GlobalCacheRules, GlobalCacheTarget},
    graphql_directive::GraphqlDirective,
    mongodb_directive::MongoDBDirective,
    openapi_directive::{OpenApiDirective, OpenApiQueryNamingStrategy, OpenApiTransforms},
};

use crate::rules::{
    cache_directive::{visitor::CacheVisitor, CacheDirective},
    mongodb_directive::MongoDBVisitor,
    scalar_hydratation::ScalarHydratation,
};

pub mod connector_parsers;

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
    #[error("Errors parsing {} connector: \n\n{}", .0.as_deref().unwrap_or("unnamed"), .1.join("\n"))]
    ConnectorErrors(Option<String>, Vec<String>, Pos),
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

/// Transform the input into a Registry
pub async fn parse<'a>(
    schema: &'a str,
    variables: &HashMap<String, String>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<ParseResult<'a>, Error> {
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
        .with::<GraphqlDirective>()
        .with::<CacheDirective>()
        .with::<MongoDBDirective>();

    let schema = format!(
        "{}\n{}\n{}\n{}",
        schema,
        GrafbaseEngineEnums::sdl(),
        PossibleScalar::sdl(),
        directives.to_definition(),
    );

    let schema = parse_schema(schema)?;

    let mut ctx = VisitorContext::new_with_variables(&schema, variables);

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

    Ok(ctx.finish())
}

async fn parse_connectors<'a>(
    schema: &'a ServiceDocument,
    ctx: &mut VisitorContext<'a>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<(), Error> {
    let mut connector_rules = rules::visitor::VisitorNil
        .with(OpenApiVisitor)
        .with(GraphqlVisitor)
        .with(MongoDBVisitor);

    visit(&mut connector_rules, ctx, schema);

    // We could probably parallelise this, but the schemas and the associated
    // processing use a reasonable amount of memory so going to keep it sequential
    for (directive, position) in std::mem::take(&mut ctx.openapi_directives) {
        let directive_name = directive.namespace.clone();
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

    for (mut directive, position) in std::mem::take(&mut ctx.graphql_directives) {
        directive.id = Some(ctx.connector_id_generator.new_id());
        let directive_name = directive.namespace().map(ToOwned::to_owned);
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
        .with(CheckAllDirectivesAreKnown::default());

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
