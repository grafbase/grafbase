use std::collections::HashMap;

use engine_parser::{types::ConstDirective, Pos, Positioned};
use engine_value::{ConstValue, Name};

use crate::{
    directive_de::parse_directive,
    rules::{
        cache_directive::{
            CacheDirective, CacheDirectiveError, CACHE_DIRECTIVE_NAME, MAX_AGE_ARGUMENT,
            MUTATION_INVALIDATION_POLICY_ARGUMENT, RULES_ARGUMENT, STALE_WHILE_REVALIDATE_ARGUMENT,
        },
        visitor::VisitorContext,
    },
};

enum ArgumentValidation {
    Mandatory,
    Forbidden,
}

pub enum ValidationLevel {
    Global,
    Type,
    Field,
}

fn validate_directive_arguments(
    ctx: &mut VisitorContext<'_>,
    pos: &Pos,
    directive_arguments: &[(Positioned<Name>, Positioned<ConstValue>)],
    arguments: &[&str],
    validation: ArgumentValidation,
) {
    let has_arguments = directive_arguments
        .iter()
        .any(|(name, _)| arguments.contains(&name.node.as_str()));

    match validation {
        ArgumentValidation::Mandatory => {
            if !has_arguments {
                ctx.report_error(
                    vec![*pos],
                    CacheDirectiveError::MandatoryArguments(arguments).to_string(),
                );
            }
        }
        ArgumentValidation::Forbidden => {
            if has_arguments {
                ctx.report_error(
                    vec![*pos],
                    CacheDirectiveError::ForbiddenArguments(arguments).to_string(),
                );
            }
        }
    }
}

pub fn validate_directive<'a>(
    ctx: &mut VisitorContext<'a>,
    directives: impl Iterator<Item = &'a Positioned<ConstDirective>>,
    pos: Pos,
    validation_level: ValidationLevel,
) -> Option<CacheDirective> {
    let directives: Vec<_> = directives
        .filter(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
        .collect();

    // only one @cache directive is allowed
    if directives.len() > 1 {
        ctx.report_error(vec![pos], CacheDirectiveError::Multiple.to_string());
    }

    directives.first().and_then(|pos_const_directive| {
        match validation_level {
            ValidationLevel::Global => {
                // check that maxAge and staleWhileRevalidate are not used at the global level
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[
                        MAX_AGE_ARGUMENT,
                        STALE_WHILE_REVALIDATE_ARGUMENT,
                        MUTATION_INVALIDATION_POLICY_ARGUMENT,
                    ],
                    ArgumentValidation::Forbidden,
                );

                // check that rules is set at the global level
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[RULES_ARGUMENT],
                    ArgumentValidation::Mandatory,
                );
            }
            ValidationLevel::Type => {
                // check that the rules argument is only used at the global level
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[RULES_ARGUMENT],
                    ArgumentValidation::Forbidden,
                );

                // check that maxAge is defined
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[MAX_AGE_ARGUMENT],
                    ArgumentValidation::Mandatory,
                );
            }
            ValidationLevel::Field => {
                // check that rules are only used at the global level and no mutation policy is present
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[RULES_ARGUMENT, MUTATION_INVALIDATION_POLICY_ARGUMENT],
                    ArgumentValidation::Forbidden,
                );

                // check that maxAge is defined
                validate_directive_arguments(
                    ctx,
                    &pos_const_directive.pos,
                    &pos_const_directive.node.arguments,
                    &[MAX_AGE_ARGUMENT],
                    ArgumentValidation::Mandatory,
                );
            }
        }

        match parse_directive::<CacheDirective>(&pos_const_directive.node, &HashMap::default()) {
            Ok(mut cache_directive) => {
                cache_directive.pos = pos_const_directive.pos;
                Some(cache_directive)
            }
            Err(err) => {
                ctx.report_error(
                    vec![pos_const_directive.pos],
                    CacheDirectiveError::Parsing(err).to_string(),
                );
                None
            }
        }
    })
}
