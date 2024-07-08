use std::collections::HashMap;

use cynic_parser::executable::{ids::SelectionId, FieldSelection, Selection};
use indexmap::{IndexMap, IndexSet};

use crate::{parser_extensions::FieldExt, planning::defers::DeferId, CachingPlan, TypeRelationships};

use super::{
    fragment_iter::FragmentIter, shape_builder::OutputShapesBuilder, ConcreteShapeId, FieldRecord, ObjectShapeId,
    OutputShapes,
};

type DeferMap = HashMap<SelectionId, DeferId>;

pub fn build_output_shapes(plan: &CachingPlan, type_relationships: &dyn TypeRelationships) -> OutputShapes {
    let defer_map = build_defer_map(plan);

    let mut builder = OutputShapesBuilder::default();

    let selections = plan
        .operation()
        .selection_set()
        .with_ids()
        .map(DeferrableSelection::without_defer)
        .collect();

    let root = build_output_shape(&mut builder, selections, &defer_map, type_relationships);
    let root = ConcreteShapeId(root.0);

    let mut defer_roots = builder.defer_roots;
    defer_roots.sort_by_key(|(shape_id, _)| *shape_id);

    OutputShapes {
        objects: builder.objects,
        type_conditions: builder.type_conditions,
        root,
        defer_roots,
    }
}

fn build_output_shape(
    builder: &mut OutputShapesBuilder,
    selections: Vec<DeferrableSelection<'_>>,
    defer_map: &DeferMap,
    type_relationships: &dyn TypeRelationships,
) -> super::ObjectShapeId {
    let type_conditions = FragmentIter::new(&selections)
        .filter_map(|fragment| fragment.type_condition())
        .collect::<IndexSet<_>>();

    if type_conditions.is_empty() {
        let field_shapes = field_shapes_for_type_condition(builder, &selections, None, defer_map, type_relationships);
        let defers = defers_for_type_condition(&selections, None, defer_map, type_relationships);

        ObjectShapeId(builder.insert_concrete_object(field_shapes, defers).0)
    } else {
        let unknown_typename_fields =
            field_shapes_for_type_condition(builder, &selections, None, defer_map, type_relationships);

        let unknown_defers = defers_for_type_condition(&selections, None, defer_map, type_relationships);

        let known_typename_fields = type_conditions
            .into_iter()
            .map(|type_condition| {
                (
                    type_condition.to_string(),
                    field_shapes_for_type_condition(
                        builder,
                        &selections,
                        Some(type_condition),
                        defer_map,
                        type_relationships,
                    ),
                    defers_for_type_condition(&selections, Some(type_condition), defer_map, type_relationships),
                )
            })
            .collect::<Vec<_>>();

        builder.insert_polymorphic_object(
            unknown_typename_fields,
            unknown_defers,
            known_typename_fields,
            type_relationships,
        )
    }
}

fn field_shapes_for_type_condition(
    builder: &mut OutputShapesBuilder,
    selections: &[DeferrableSelection<'_>],
    type_condition: Option<&str>,
    defer_map: &DeferMap,
    type_relationships: &dyn TypeRelationships,
) -> Vec<FieldRecord> {
    let mut grouped_fields = IndexMap::new();

    collect_fields(
        &mut grouped_fields,
        &mut vec![],
        selections,
        type_condition,
        defer_map,
        type_relationships,
    );

    let merged_fields = merge_selection_sets(grouped_fields);

    let mut field_shapes = vec![];

    for field in merged_fields {
        let mut subselection_shape = None;
        if !field.merged_selections.is_empty() {
            subselection_shape = Some(build_output_shape(
                builder,
                field.merged_selections,
                defer_map,
                type_relationships,
            ));
        }
        field_shapes.push(FieldRecord {
            response_key: field.response_key.to_string(),
            defer: field.defer,
            subselection_shape,
        });
    }

    field_shapes
}

struct CollectedField<'a> {
    field: FieldSelection<'a>,

    /// If this field is anywhere inside a defer, this will be set
    defer: Option<DeferId>,
}

/// A defer aware implementation of CollectFields from the GraphQL spec:
///
/// Note that this doesn't process include or skip currently.  I think this
/// is fine and we can leave that up to the actual GraphQL server, but may
/// need revisited if that's wrong.
///
/// http://spec.graphql.org/October2021/#CollectFields()
fn collect_fields<'a>(
    grouped_fields: &mut IndexMap<&'a str, Vec<CollectedField<'a>>>,
    defer_stack: &mut Vec<DeferId>,
    selections: &[DeferrableSelection<'a>],
    type_condition: Option<&'a str>,
    defer_map: &DeferMap,
    type_relationships: &dyn TypeRelationships,
) {
    for selection in selections {
        match selection.selection {
            Selection::Field(field) => {
                // If the current selection has applied a defer we take that, otherwise we take the propagated
                // defer label (if present)
                let defer = defer_stack.last().copied().or(selection.parent_defer);

                grouped_fields
                    .entry(field.response_key())
                    .or_default()
                    .push(CollectedField { field, defer });
            }
            Selection::InlineFragment(fragment) => {
                if let Some((required_condition, current_condition)) = fragment.type_condition().zip(type_condition) {
                    if !type_relationships.type_condition_matches(required_condition, current_condition) {
                        continue;
                    }
                }

                let defer = defer_map.get(&selection.id).copied();
                if let Some(defer) = defer {
                    defer_stack.push(defer);
                }

                collect_fields(
                    grouped_fields,
                    defer_stack,
                    &fragment
                        .selection_set()
                        .with_ids()
                        .map(|nested_selection| {
                            DeferrableSelection::with_defer(nested_selection, selection.parent_defer)
                        })
                        .collect::<Vec<_>>(),
                    type_condition,
                    defer_map,
                    type_relationships,
                );

                if defer.is_some() {
                    defer_stack.pop();
                }
            }
            Selection::FragmentSpread(spread) => {
                let Some(fragment) = spread.fragment() else {
                    continue;
                };

                let Some(current_condition) = type_condition else {
                    // Fragment spreads don't apply if we're evaluating the non-match case.
                    continue;
                };

                if !type_relationships.type_condition_matches(fragment.type_condition(), current_condition) {
                    continue;
                }

                let defer = defer_map.get(&selection.id).copied();
                if let Some(defer) = defer {
                    defer_stack.push(defer);
                }

                collect_fields(
                    grouped_fields,
                    defer_stack,
                    &fragment
                        .selection_set()
                        .with_ids()
                        .map(|nested_selection| {
                            DeferrableSelection::with_defer(nested_selection, selection.parent_defer)
                        })
                        .collect::<Vec<_>>(),
                    type_condition,
                    defer_map,
                    type_relationships,
                );

                if defer.is_some() {
                    defer_stack.pop();
                }
            }
        }
    }
}

/// A field in a selection set after it's been through MergeSelectionSets
///
/// The same field can appear multiple times in a selection set, with different
/// child selection sets in each case.  This struct contains all of the selections
/// from those instances of the field.
struct MergedField<'a> {
    response_key: &'a str,

    /// The label of the defer for this selection.
    ///
    /// This should only be set if none of the parent fields have the same defer_label
    defer: Option<DeferId>,

    merged_selections: Vec<DeferrableSelection<'a>>,
}

/// Wrapper around a Selection that allows defer labels to be propagated where
/// neccesary
pub(super) struct DeferrableSelection<'a> {
    id: SelectionId,
    pub(super) selection: Selection<'a>,

    parent_defer: Option<DeferId>,
}

impl<'a> DeferrableSelection<'a> {
    pub fn without_defer((id, selection): (SelectionId, Selection<'a>)) -> Self {
        DeferrableSelection {
            id,
            selection,
            parent_defer: None,
        }
    }

    pub fn with_defer((id, selection): (SelectionId, Selection<'a>), parent_defer: Option<DeferId>) -> Self {
        DeferrableSelection {
            id,
            selection,
            parent_defer,
        }
    }
}

/// An implementation of MergeSelectionSets from the GraphQL spec.
///
/// This is a bit more complicated than the GraphQL spec outlines, because
/// we need to handle propagating the defer label in certain cases.
///
/// For example with this query:
///
/// ```graphql
/// query {
///   foo {
///     bar {
///       baz {
///         zap
///       }
///     }
///     ... @defer(name: "whatever") {
///       bar {
///         baz {
///           blorp
///         }
///       }
///     }
///   }
/// }
/// ```
///
/// In this scenario `bar` & `bar.baz` appear in both a deferred and
/// a non-deferred context - so rather than marking `bar` as deferred,
/// we need to propagate the label down into the next selection set. By
/// doing this recursively we should end up only marking fields that
/// are exclusively in a defer as deferrable.
///
/// This would have problems if we needed to know what the root of a
/// defer is, but I think we can mostly leave that up to the executor -
/// we generally just need to know whether fields are part of a defer
/// or not, and this should let us do that.
///
/// http://spec.graphql.org/October2021/#MergeSelectionSets()
fn merge_selection_sets<'a>(grouped_fields: IndexMap<&'a str, Vec<CollectedField<'a>>>) -> Vec<MergedField<'a>> {
    let mut output = Vec::with_capacity(grouped_fields.len());
    for (response_key, fields) in grouped_fields {
        if fields.len() == 1 {
            // Hooray, the easy case
            output.push(MergedField {
                response_key,
                defer: fields[0].defer,
                merged_selections: fields[0]
                    .field
                    .selection_set()
                    .with_ids()
                    .map(DeferrableSelection::without_defer)
                    .collect(),
            });
            continue;
        }

        if fields[0].field.selection_set().len() == 0 {
            // This looks like a leaf field so we can't merge selection sets.
            // Lets just pick the properties of the first field
            output.push(MergedField {
                response_key,
                defer: fields[0].defer,
                merged_selections: vec![],
            });
            continue;
        }

        // If there's any mismatch on defer labels we want to propagate them to child fields instead
        // of putting them on this level of the heirarchy.
        let all_defers_match = match fields.as_slice() {
            [head, tail @ ..] => tail.iter().all(|x| x.defer == head.defer),
            [] => unreachable!(),
        };

        let mut merged_field = MergedField {
            response_key: "",
            defer: None,
            merged_selections: vec![],
        };

        for field in fields {
            merged_field.response_key = field.field.response_key();
            let selections = field.field.selection_set().with_ids();
            if all_defers_match {
                merged_field.defer = field.defer;
                merged_field
                    .merged_selections
                    .extend(selections.map(DeferrableSelection::without_defer));
            } else {
                merged_field
                    .merged_selections
                    .extend(selections.map(|selection| DeferrableSelection::with_defer(selection, merged_field.defer)));
            }
        }
    }

    output
}

fn defers_for_type_condition(
    selections: &[DeferrableSelection<'_>],
    type_condition: Option<&str>,
    defer_map: &DeferMap,
    type_relationships: &dyn TypeRelationships,
) -> Vec<DeferId> {
    let mut output = vec![];
    for selection in selections {
        match selection.selection {
            Selection::Field(_) => {}
            Selection::InlineFragment(fragment) => {
                if let Some((required_condition, current_condition)) = fragment.type_condition().zip(type_condition) {
                    if !type_relationships.type_condition_matches(required_condition, current_condition) {
                        continue;
                    }
                }

                output.extend(defer_map.get(&selection.id).copied());
                let nested_selections = fragment
                    .selection_set()
                    .with_ids()
                    .map(DeferrableSelection::without_defer)
                    .collect::<Vec<_>>();
                output.extend(defers_for_type_condition(
                    &nested_selections,
                    type_condition,
                    defer_map,
                    type_relationships,
                ));
            }
            Selection::FragmentSpread(spread) => {
                let Some(fragment) = spread.fragment() else {
                    continue;
                };

                let Some(current_condition) = type_condition else {
                    // Fragment spreads don't apply if we're evaluating the non-match case.
                    continue;
                };

                if !type_relationships.type_condition_matches(fragment.type_condition(), current_condition) {
                    continue;
                }

                output.extend(defer_map.get(&selection.id).copied());
                let nested_selections = fragment
                    .selection_set()
                    .with_ids()
                    .map(DeferrableSelection::without_defer)
                    .collect::<Vec<_>>();
                output.extend(defers_for_type_condition(
                    &nested_selections,
                    type_condition,
                    defer_map,
                    type_relationships,
                ));
            }
        }
    }

    output
}

fn build_defer_map(plan: &CachingPlan) -> DeferMap {
    plan.defers().map(|defer| (defer.spread_id(), defer.id)).collect()
}
