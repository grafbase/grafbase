use std::collections::{HashMap, HashSet};

use grafbase_engine::Positioned;
use grafbase_engine_parser::types::ConstDirective;
use grafbase_engine_value::ConstValue;

use super::visitor::VisitorContext;

pub trait Directive {
    fn definition() -> String;
}

pub struct Directives(Vec<String>);

impl Directives {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn to_definition(&self) -> String {
        self.0.join("\n")
    }

    pub fn with<D: Directive>(self) -> Directives {
        let mut v = self.0;
        v.push(D::definition());
        Self(v)
    }
}

pub(crate) fn extract_arguments<'a>(
    ctx: &mut VisitorContext<'a>,
    directive: &'a Positioned<ConstDirective>,
    allowed_sorted_argument_combinations: &[&[&str]],
    argument_help_string: Option<&str>,
) -> Result<HashMap<&'a str, ConstValue>, ()> {
    use itertools::Itertools;

    let directive_name = directive.node.name.node.as_str();

    // Extract and group args
    let arguments: Vec<_> = directive
        .node
        .arguments
        .iter()
        .map(|(name, value)| (name.node.as_str(), value.node.clone()))
        .collect();

    let argument_keys: Vec<_> = arguments.iter().map(|(name, _)| *name).sorted().collect();

    if allowed_sorted_argument_combinations.contains(&argument_keys.as_slice()) {
        Ok(arguments.into_iter().collect())
    } else {
        let mut bail_out = false;
        for (duplicate_key, _) in arguments.iter().duplicates_by(|(key, _)| key) {
            ctx.report_error(
                vec![directive.pos],
                format!("The @{directive_name} directive expects the `{duplicate_key}` argument only once"),
            );
            bail_out = true;
        }
        if bail_out {
            return Err(());
        }

        if let &[&[single_argument]] = allowed_sorted_argument_combinations {
            ctx.report_error(
                vec![directive.pos],
                format!("The @{directive_name} directive takes a single `{single_argument}` argument"),
            );
        } else {
            let all_accepted_argument_keys: HashSet<_> = allowed_sorted_argument_combinations
                .iter()
                .flat_map(|combination| combination.iter())
                .copied()
                .collect();

            let argument_keys: HashSet<_> = argument_keys.into_iter().collect();

            for unknown_key in argument_keys.difference(&all_accepted_argument_keys) {
                let all_accepted_argument_keys_string: String = all_accepted_argument_keys
                    .iter()
                    .sorted()
                    .map(|argument| std::borrow::Cow::Owned(format!("`{argument}`")))
                    .interleave(
                        std::iter::repeat(", ")
                            .take(all_accepted_argument_keys.len().saturating_sub(2))
                            .chain(std::iter::once(" and "))
                            .map(std::borrow::Cow::Borrowed),
                    )
                    .collect();

                ctx.report_error(
                    vec![directive.pos],
                    format!("Unexpected argument {unknown_key}, @{directive_name} directive only supports the following arguments: {all_accepted_argument_keys_string}")
                );
                bail_out = true;
            }
            if bail_out {
                return Err(());
            }

            let combinations_string = argument_help_string.map(std::borrow::Cow::Borrowed).unwrap_or_else(|| {
                allowed_sorted_argument_combinations
                    .iter()
                    .map(|combination| combination.iter().map(|argument| format!("`{argument}`")).join(" and "))
                    .join(" or ")
                    .into()
            });
            ctx.report_error(
                vec![directive.pos],
                format!("The @{directive_name} directive expects at least one of the {combinations_string} arguments"),
            );
        }
        Err(())
    }
}
