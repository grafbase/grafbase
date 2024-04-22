use engine_parser::{
    types::{Directive, Selection, SelectionSet},
    Positioned,
};
use engine_value::Variables;
use serde::Deserialize;

use crate::{
    directive::DirectiveDeserializer, registry::type_kinds::SelectionSetTarget, Context, ContextExt,
    ContextSelectionSet, ServerError,
};

/// The details of a fragment spread/inline fragment.
///
/// Used to simplify handling each
pub(super) struct FragmentDetails<'a> {
    pub type_condition: Option<&'a str>,
    pub selection_set: &'a Positioned<SelectionSet>,
    pub defer: Option<DeferDirective>,
}

impl<'a> FragmentDetails<'a> {
    pub(super) fn should_defer(&self, ctx: &ContextSelectionSet<'a>) -> bool {
        self.defer
            .as_ref()
            .map(|directive| directive.should_defer)
            .unwrap_or_default()
            && ctx.deferred_workloads().is_some()
    }

    pub(super) fn from_fragment_selection(
        ctx: &dyn Context<'a>,
        selection: &'a Selection,
    ) -> Result<FragmentDetails<'a>, ServerError> {
        match selection {
            Selection::Field(_) => unreachable!("this should have been validated before calling this function"),
            Selection::FragmentSpread(spread) => {
                let defer = DeferDirective::parse(&spread.directives, &ctx.query_env().variables)?;
                let fragment = ctx.query_env().fragments.get(&spread.node.fragment_name.node);
                let fragment = match fragment {
                    Some(fragment) => fragment,
                    None => {
                        return Err(ServerError::new(
                            format!(r#"Unknown fragment "{}"."#, spread.node.fragment_name.node),
                            Some(spread.pos),
                        ));
                    }
                };
                Ok(FragmentDetails {
                    type_condition: Some(fragment.node.type_condition.node.on.node.as_str()),
                    selection_set: &fragment.node.selection_set,
                    defer,
                })
            }
            Selection::InlineFragment(fragment) => Ok(FragmentDetails {
                type_condition: fragment
                    .node
                    .type_condition
                    .as_ref()
                    .map(|positioned| positioned.node.on.node.as_str()),
                selection_set: &fragment.node.selection_set,
                defer: DeferDirective::parse(&fragment.directives, &ctx.query_env().variables)?,
            }),
        }
    }

    pub(super) fn type_condition_matches(&self, ctx: &ContextSelectionSet<'_>, typename: &str) -> bool {
        let Some(type_condition) = self.type_condition else {
            // If we've no type condition then we always match
            return true;
        };

        match ctx.ty {
            SelectionSetTarget::Union(union) => typename == type_condition || type_condition == union.name(),
            _ => {
                typename == type_condition
                    || ctx
                        .registry()
                        .interfaces_implemented(typename)
                        .any(|ty| ty.name() == type_condition)
            }
        }
    }
}

#[derive(serde::Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DeferDirective {
    pub label: Option<String>,
    #[serde(rename = "if", default = "default_true")]
    should_defer: bool,
}

fn default_true() -> bool {
    true
}

const DEFER: &str = "defer";

impl DeferDirective {
    pub fn parse(directives: &[Positioned<Directive>], variables: &Variables) -> Result<Option<Self>, ServerError> {
        directives
            .iter()
            .find(|directive| directive.node.name.node == DEFER)
            .map(|directive| {
                DeferDirective::deserialize(DirectiveDeserializer::new(&directive.node, variables))
                    .map_err(|error| error.into_server_error(DEFER, directive.pos))
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_query;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_defer_parsing() {
        assert_eq!(
            parse_directive("@defer").unwrap(),
            DeferDirective {
                label: None,
                should_defer: true
            }
        );

        assert_eq!(
            parse_directive("@defer(if: false)").unwrap(),
            DeferDirective {
                label: None,
                should_defer: false
            }
        );

        assert_eq!(
            parse_directive("@defer(if: $two)").unwrap(),
            DeferDirective {
                label: None,
                should_defer: true
            }
        );

        assert_eq!(
            parse_directive("@defer(if: $two, label: $one)").unwrap(),
            DeferDirective {
                label: Some("hello".into()),
                should_defer: true
            }
        );

        assert_eq!(
            parse_directive(r#"@defer(label: "one")"#).unwrap(),
            DeferDirective {
                label: Some("one".into()),
                should_defer: true
            }
        );
    }

    #[test]
    fn missing_variable_error() {
        insta::assert_snapshot!(parse_directive(r"@defer(label: $nope)").unwrap_err(), @"Error interpreting @defer: unknown variable nope");
    }

    #[test]
    fn additional_variable_error() {
        insta::assert_snapshot!(parse_directive(r"@defer(wrong: true)").unwrap_err(), @"Error interpreting @defer: unknown field `wrong`, expected `label` or `if`");
    }

    #[test]
    fn wrong_variable_type_error() {
        insta::assert_snapshot!(parse_directive(r"@defer(label: $two)").unwrap_err(), @"Error interpreting @defer: invalid type: boolean `true`, expected a string for the argument `label`");
    }

    #[test]
    fn wrong_literal_type_error() {
        insta::assert_snapshot!(parse_directive(r"@defer(label: true)").unwrap_err(), @"Error interpreting @defer: invalid type: boolean `true`, expected a string for the argument `label`");
    }

    fn parse_directive(directive_string: &str) -> Result<DeferDirective, ServerError> {
        let ast_directives = parse_query(format!("query @other(blah: true) {directive_string} {{ name }}"))
            .unwrap()
            .operations
            .iter()
            .next()
            .unwrap()
            .1
            .directives
            .clone();

        DeferDirective::parse(
            &ast_directives,
            &Variables::from_json(json!({
                "one": "hello",
                "two": true
            })),
        )
        .map(Option::unwrap)
    }
}
