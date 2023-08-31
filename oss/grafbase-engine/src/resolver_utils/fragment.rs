use grafbase_engine_parser::{
    types::{Directive, Selection, SelectionSet},
    Pos, Positioned,
};

use crate::{ContextBase, ServerError};

/// The details of a fragment spread/inline fragment.
///
/// Used to simplify handling each
pub(super) struct FragmentDetails<'a> {
    pub position: Pos,
    pub type_condition: Option<&'a str>,
    pub selection_set: &'a Positioned<SelectionSet>,
    pub defer: Option<DeferDirective>,
}

impl<'a> FragmentDetails<'a> {
    pub(super) fn from_fragment_selection(
        ctx: &ContextBase<'a, &Positioned<SelectionSet>>,
        selection: &'a Selection,
    ) -> Result<FragmentDetails<'a>, ServerError> {
        match selection {
            Selection::Field(_) => unreachable!("this should have been validated before calling this function"),
            Selection::FragmentSpread(spread) => {
                let defer = DeferDirective::parse(&spread.directives);
                let fragment = ctx.query_env.fragments.get(&spread.node.fragment_name.node);
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
                    position: spread.pos,
                    type_condition: Some(fragment.node.type_condition.node.on.node.as_str()),
                    selection_set: &fragment.node.selection_set,
                    defer,
                })
            }
            Selection::InlineFragment(fragment) => Ok(FragmentDetails {
                position: fragment.pos,
                type_condition: fragment
                    .node
                    .type_condition
                    .as_ref()
                    .map(|positioned| positioned.node.on.node.as_str()),
                selection_set: &fragment.node.selection_set,
                defer: DeferDirective::parse(&fragment.directives),
            }),
        }
    }
}

pub struct DeferDirective {
    #[allow(dead_code)]
    label: Option<String>,
}

impl DeferDirective {
    pub fn parse(directives: &[Positioned<Directive>]) -> Option<Self> {
        directives
            .iter()
            .find(|directive| directive.node.name.node == "defer")
            .map(|directive| &directive.node)
            .map(|_|
                // currently we're not bothering to parse attributes.  that will come later
                DeferDirective { label: None })
    }
}
