use super::{arguments, directives};
use graphql_parser::query::{Selection, SelectionSet, TypeCondition};
use std::{cmp::Ordering, collections::HashMap};

pub(super) fn normalize<'a>(
    selection_set: &mut SelectionSet<'a, &'a str>,
    used_fragments: &mut HashMap<String, bool>,
    in_operation: bool,
) {
    for selection in &mut selection_set.items {
        normalize_selection(selection, used_fragments, in_operation);
    }

    selection_set.items.sort_by(sort_selection);
}

fn normalize_selection<'a>(
    selection: &mut Selection<'a, &'a str>,
    used_fragments: &mut HashMap<String, bool>,
    in_operation: bool,
) {
    match selection {
        Selection::Field(field) => {
            field.alias = None;

            arguments::normalize(&mut field.arguments);
            directives::normalize(&mut field.directives);

            normalize(&mut field.selection_set, used_fragments, in_operation);
        }
        Selection::FragmentSpread(fragment) => {
            let fragment_name = fragment.fragment_name.to_string();

            directives::normalize(&mut fragment.directives);
            used_fragments.entry(fragment_name).or_insert(in_operation);
        }
        Selection::InlineFragment(fragment) => {
            directives::normalize(&mut fragment.directives);
            normalize(&mut fragment.selection_set, used_fragments, in_operation);
        }
    }
}

fn sort_selection<'a>(a: &Selection<'a, &'a str>, b: &Selection<'a, &'a str>) -> Ordering {
    match (a, b) {
        (Selection::Field(a), Selection::Field(b)) => a.name.cmp(b.name),
        (Selection::Field(_), Selection::FragmentSpread(_)) => Ordering::Less,
        (Selection::Field(_), Selection::InlineFragment(_)) => Ordering::Less,
        (Selection::FragmentSpread(_), Selection::Field(_)) => Ordering::Greater,
        (Selection::FragmentSpread(a), Selection::FragmentSpread(b)) => a.fragment_name.cmp(b.fragment_name),
        (Selection::FragmentSpread(_), Selection::InlineFragment(_)) => Ordering::Less,
        (Selection::InlineFragment(_), Selection::Field(_)) => Ordering::Greater,
        (Selection::InlineFragment(_), Selection::FragmentSpread(_)) => Ordering::Greater,
        (Selection::InlineFragment(a), Selection::InlineFragment(b)) => match (&a.type_condition, &b.type_condition) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => match (a, b) {
                (TypeCondition::On(a), TypeCondition::On(b)) => a.cmp(b),
            },
        },
    }
}
