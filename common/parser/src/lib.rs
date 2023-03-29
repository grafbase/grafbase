#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::collections::{HashMap, HashSet};

use dynaql::registry::enums::DynaqlEnums;
use dynaql::registry::scalars::{PossibleScalar, SDLDefinitionScalar};
use dynaql_parser::{parse_schema, Error as ParserError};
use quick_error::quick_error;
use rules::auth_directive::AuthDirective;
use rules::basic_type::BasicType;
use rules::check_field_lowercase::CheckFieldCamelCase;
use rules::check_known_directives::CheckAllDirectivesAreKnown;
use rules::check_type_collision::CheckTypeCollision;
use rules::check_type_validity::CheckTypeValidity;
use rules::check_types_underscore::CheckBeginsWithDoubleUnderscore;
use rules::default_directive::DefaultDirective;
use rules::default_directive_types::DefaultDirectiveTypes;
use rules::directive::Directives;
use rules::enum_type::EnumType;
use rules::extend_query_and_mutation_types::ExtendQueryAndMutationTypes;
use rules::length_directive::LengthDirective;
use rules::model_directive::ModelDirective;
use rules::one_of_directive::OneOfDirective;
use rules::openapi_directive::OpenApiVisitor;
use rules::relations::{relations_rules, RelationEngine};
use rules::resolver_directive::ResolverDirective;
use rules::search_directive::SearchDirective;
use rules::unique_directive::UniqueDirective;
use rules::unique_fields::UniqueObjectFields;
use rules::visitor::{visit, RuleError, Visitor, VisitorContext};

mod models;

use crate::rules::cache_directive::{CacheDirective, CacheVisitor};
pub use dynaql::registry::Registry;
pub use migration_detection::{required_migrations, RequiredMigration};
pub use rules::openapi_directive::{OpenApiDirective, OpenApiQueryNamingStrategy, OpenApiTransforms};

use crate::rules::scalar_hydratation::ScalarHydratation;

mod directive_de;
mod dynamic_string;
mod migration_detection;
mod registry;
mod rules;
#[cfg(test)]
mod tests;
mod utils;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Parser(err: ParserError) {
            from()
            source(err)
            display("{}", err)
        }
        Validation(err: Vec<RuleError>) {
            from()
            display("{:?}", err)
        }
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

/// Transform the input schema into a Registry
pub fn to_registry<S: AsRef<str>>(input: S) -> Result<Registry, Error> {
    Ok(to_registry_with_variables(input, &HashMap::new())?.registry)
}

pub struct ParseResult {
    pub registry: Registry,
    pub required_resolvers: HashSet<String>,
    pub openapi_directives: Vec<OpenApiDirective>,
}

/// Transform the input schema into a Registry in the context of provided environment variables
pub fn to_registry_with_variables<S: AsRef<str>>(
    input: S,
    variables: &HashMap<String, String>,
) -> Result<ParseResult, Error> {
    let directives = Directives::new()
        .with::<AuthDirective>()
        .with::<DefaultDirective>()
        .with::<LengthDirective>()
        .with::<ModelDirective>()
        .with::<OneOfDirective>()
        .with::<RelationEngine>()
        .with::<ResolverDirective>()
        .with::<UniqueDirective>()
        .with::<SearchDirective>()
        .with::<OpenApiDirective>()
        .with::<CacheDirective>();

    let mut rules = rules::visitor::VisitorNil
        .with(CheckBeginsWithDoubleUnderscore)
        .with(CheckFieldCamelCase)
        .with(CheckTypeValidity)
        .with(SearchDirective)
        .with(ModelDirective)
        .with(AuthDirective)
        .with(ResolverDirective)
        .with(CacheVisitor)
        .with(BasicType)
        .with(ExtendQueryAndMutationTypes)
        .with(EnumType)
        .with(ScalarHydratation)
        .with(LengthDirective)
        .with(UniqueObjectFields)
        .with(OpenApiVisitor)
        .with(CheckAllDirectivesAreKnown::default());

    let schema = format!(
        "{}\n{}\n{}\n{}",
        input.as_ref(),
        DynaqlEnums::sdl(),
        PossibleScalar::sdl(),
        directives.to_definition(),
    );
    let schema = parse_schema(schema)?;

    let mut ctx = VisitorContext::new_with_variables(&schema, variables);
    // Building all relations first are it requires to parse the whole schema (for ManyToMany). This allows later
    // rules to rely on RelationEngine::get to have correct information on relations.
    visit(
        &mut relations_rules().with(CheckTypeCollision::default()),
        &mut ctx,
        &schema,
    );
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    visit(&mut rules, &mut ctx, &schema);

    // FIXME: Get rid of the ugly double pass.
    let mut second_pass_rules = rules::visitor::VisitorNil.with(DefaultDirectiveTypes);
    visit(&mut second_pass_rules, &mut ctx, &schema);

    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    let (registry, required_resolvers, openapi_directives) = ctx.finish();
    Ok(ParseResult {
        registry,
        required_resolvers,
        openapi_directives,
    })
}
