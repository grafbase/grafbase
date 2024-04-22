#[cfg(test)]
#[macro_use]
mod test_harness;

pub mod dynamic_validators;
mod rules;
mod suggestion;
pub mod utils;
mod visitor;
mod visitors;

use std::collections::HashSet;

pub use visitor::VisitorContext;
use visitor::{visit, VisitorNil};

use crate::{
    parser::types::ExecutableDocument, registry::Registry, CacheControl, CacheInvalidation, ServerError, Variables,
};

/// Validation results.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Cache control
    pub cache_control: CacheControl,

    /// Cache mutation invalidation policies
    pub cache_invalidation_policies: HashSet<CacheInvalidation>,

    /// Query complexity
    pub complexity: usize,

    /// Query depth
    pub depth: usize,

    /// Query height
    ///
    /// Limits the number of unique fields included in an operation, including fields of fragments. If a particular field is included multiple times via aliases, it's counted only once.
    pub height: usize,

    /// Root fields in the query
    pub root_field_count: usize,

    /// Alias  count.
    pub alias_count: usize,
}

/// Validation mode
#[derive(Copy, Clone, Debug)]
pub enum ValidationMode {
    /// Execute all validation rules.
    Strict,

    /// The executor itself also has error handling, so it can improve performance, but it can lose some error messages.
    Fast,
}

pub fn check_rules(
    registry: &registry_v2::Registry,
    doc: &ExecutableDocument,
    variables: Option<&Variables>,
    mode: ValidationMode,
) -> Result<ValidationResult, Vec<ServerError>> {
    let mut ctx = VisitorContext::new(registry, doc, variables);
    let mut cache_control = registry_v2::cache_control::CacheControl::default();
    let mut cache_invalidation_policies = Default::default();
    let mut complexity = 0;
    let mut depth = 0;
    let mut root_field_count: usize = 0;
    let mut height: usize = 0;
    let mut alias_count: usize = 0;

    match mode {
        ValidationMode::Strict => {
            let mut visitor = VisitorNil
                .with(rules::ArgumentsOfCorrectType::default())
                .with(rules::DefaultValuesOfCorrectType)
                .with(rules::FieldsOnCorrectType)
                .with(rules::FragmentsOnCompositeTypes)
                .with(rules::KnownArgumentNames::default())
                .with(rules::NoFragmentCycles::default())
                .with(rules::KnownFragmentNames)
                .with(rules::KnownTypeNames)
                .with(rules::NoUndefinedVariables::default())
                .with(rules::NoUnusedFragments::default())
                .with(rules::NoUnusedVariables::default())
                .with(rules::UniqueArgumentNames::default())
                .with(rules::UniqueVariableNames::default())
                .with(rules::VariablesAreInputTypes)
                .with(rules::VariableInAllowedPosition::default())
                .with(rules::ScalarLeafs)
                .with(rules::PossibleFragmentSpreads::default())
                .with(rules::ProvidedNonNullArguments)
                .with(rules::KnownDirectives::default())
                .with(rules::DirectivesUnique)
                .with(rules::OverlappingFieldsCanBeMerged)
                .with(rules::UploadFile)
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                    invalidation_policies: &mut cache_invalidation_policies,
                })
                .with(visitors::DepthCalculate::new(&mut depth))
                .with(visitors::HeightCalculate::new(&mut height))
                .with(visitors::AliasCountCalculate::new(&mut alias_count))
                .with(visitors::RootFieldCountCalculate::new(&mut root_field_count));

            visit(&mut visitor, &mut ctx, doc);
        }
        ValidationMode::Fast => {
            let mut visitor = VisitorNil
                .with(rules::NoFragmentCycles::default())
                .with(rules::UploadFile)
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                    invalidation_policies: &mut cache_invalidation_policies,
                })
                .with(visitors::DepthCalculate::new(&mut depth))
                .with(visitors::HeightCalculate::new(&mut height))
                .with(visitors::AliasCountCalculate::new(&mut alias_count))
                .with(visitors::RootFieldCountCalculate::new(&mut root_field_count));

            visit(&mut visitor, &mut ctx, doc);
        }
    }

    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into_iter().map(Into::into).collect());
    }

    Ok(ValidationResult {
        cache_control,
        cache_invalidation_policies,
        complexity,
        depth,
        root_field_count,
        height,
        alias_count,
    })
}
