use std::collections::HashSet;

use cynic_parser::executable::ids::SelectionId;

use crate::{parser_extensions::DeferExt, CachingPlan};

use super::visitor::Visitor;

impl crate::CachingPlan {
    pub fn defers(&self) -> impl ExactSizeIterator<Item = Defer<'_>> + '_ {
        self.defers.iter().enumerate().map(|(i, _)| Defer {
            id: DeferId(i.try_into().expect("there were more than 2^16 defers?  wtf")),
            plan: self,
        })
    }
}

pub(crate) struct DeferRecord {
    label: Option<String>,
    spread_id: SelectionId,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeferId(u16);

pub struct Defer<'a> {
    pub id: DeferId,
    plan: &'a CachingPlan,
}

impl<'a> Defer<'a> {
    pub fn spread_id(&self) -> SelectionId {
        self.record().spread_id
    }

    pub fn label(&self) -> Option<&'a str> {
        self.record().label.as_deref()
    }

    fn record(&self) -> &'a DeferRecord {
        &self.plan.defers[self.id.0 as usize]
    }
}

#[derive(Default)]
pub(super) struct DeferVisitor {
    pub defers: Vec<DeferRecord>,
    seen_selections: HashSet<SelectionId>,
}

impl DeferVisitor {
    pub fn new() -> Self {
        DeferVisitor::default()
    }
}

impl Visitor for DeferVisitor {
    fn enter_selection(&mut self, id: SelectionId, selection: cynic_parser::executable::Selection<'_>) {
        let directive = match selection {
            cynic_parser::executable::Selection::Field(_) => None,
            cynic_parser::executable::Selection::InlineFragment(fragment) => fragment.defer_directive(),
            cynic_parser::executable::Selection::FragmentSpread(spread) => spread.defer_directive(),
        };
        let Some(directive) = directive else { return };
        if !self.seen_selections.insert(id) {
            return;
        }

        self.defers.push(DeferRecord {
            label: directive.label.map(str::to_string),
            spread_id: id,
        })
    }
}
