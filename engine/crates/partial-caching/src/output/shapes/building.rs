use cynic_parser::executable::{FieldSelection, Selection};
use indexmap::{IndexMap, IndexSet};

use crate::{
    parser_extensions::{DeferExt, FieldExt},
    query_subset::FilteredSelectionSet,
    CachingPlan, QuerySubset,
};

use super::{
    fragment_iter::FragmentIter, ConcreteShapeId, FieldRecord, ObjectShapeId, ObjectShapeRecord, OutputShapes,
};

pub fn build_output_shapes(plan: CachingPlan) -> OutputShapes {
    let mut builder = OutputShapesBuilder::default();
    let mut cache_partition_roots = vec![];

    for (subset, selection_set) in plan.cache_partitions() {
        let selections = selection_set.map(DeferrableSelection::without_defer).collect();

        let shape_id = build_output_shape(&mut builder, selections, subset);
        let concrete_id = ConcreteShapeId(shape_id.0);

        cache_partition_roots.push(concrete_id);
    }

    let nocache_partition_root = {
        let (subset, selection_set) = plan.nocache_partition();
        let selections = selection_set.map(DeferrableSelection::without_defer).collect();

        let shape_id = build_output_shape(&mut builder, selections, subset);
        ConcreteShapeId(shape_id.0)
    };

    OutputShapes {
        objects: builder.objects,
        cache_partition_roots,
        nocache_partition_root,
    }
}

fn build_output_shape(
    builder: &mut OutputShapesBuilder,
    selections: Vec<DeferrableSelection<'_>>,
    subset: &QuerySubset,
) -> super::ObjectShapeId {
    let type_conditions = FragmentIter::new(&selections, subset)
        .filter_map(|fragment| fragment.type_condition())
        .collect::<IndexSet<_>>();

    if type_conditions.is_empty() {
        let field_shapes = field_shapes_for_type_condition(builder, &selections, subset, None);

        builder.insert_concrete_object(field_shapes)
    } else {
        let unknown_typename_fields = field_shapes_for_type_condition(builder, &selections, subset, None);

        let known_typename_fields = type_conditions
            .into_iter()
            .map(|typename| {
                (
                    typename.to_string(),
                    field_shapes_for_type_condition(builder, &selections, subset, Some(typename)),
                )
            })
            .collect::<Vec<_>>();

        builder.insert_polymorphic_object(unknown_typename_fields, known_typename_fields)
    }
}

fn field_shapes_for_type_condition(
    builder: &mut OutputShapesBuilder,
    selections: &[DeferrableSelection<'_>],
    subset: &QuerySubset,
    type_condition: Option<&str>,
) -> Vec<FieldRecord> {
    let mut grouped_fields = IndexMap::new();

    collect_fields(&mut grouped_fields, &mut vec![], selections, subset, type_condition);

    let merged_fields = merge_selection_sets(grouped_fields, subset);

    let mut field_shapes = vec![];

    for field in merged_fields {
        let mut subselection_shape = None;
        if !field.merged_selections.is_empty() {
            subselection_shape = Some(build_output_shape(builder, field.merged_selections, subset));
        }
        field_shapes.push(FieldRecord {
            response_key: field.response_key.to_string(),
            defer_label: field.defer_label.map(ToString::to_string),
            subselection_shape,
        });
    }

    field_shapes
}

struct CollectedField<'a> {
    field: FieldSelection<'a>,

    /// The label of the defer this selection is in if any
    defer_label: Option<&'a str>,
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
    defer_stack: &mut Vec<&'a str>,
    selections: &[DeferrableSelection<'a>],
    subset: &'a QuerySubset,
    type_condition: Option<&'a str>,
) {
    for selection in selections {
        match selection.selection {
            Selection::Field(field) => {
                // If the current selection has applied a defer we take that, otherwise we take the propagated
                // defer label (if present)
                let defer_label = defer_stack.last().copied().or(selection.parent_defer_label);

                grouped_fields
                    .entry(field.response_key())
                    .or_default()
                    .push(CollectedField { field, defer_label });
            }
            Selection::InlineFragment(fragment) => {
                if fragment.type_condition() != type_condition {
                    // TODO: This needs to be smarter.  If there's no type_condition it doesn't matter what typename
                    // is. We also need to handle implements properly, which will require the registry.
                    //
                    // Will revisit later though...
                    continue;
                }

                let defer = fragment.defer_directive();
                if let Some(defer) = &defer {
                    defer_stack.push(defer.label);
                }

                collect_fields(
                    grouped_fields,
                    defer_stack,
                    &subset
                        .selection_iter(fragment.selection_set())
                        .map(|nested_selection| DeferrableSelection {
                            selection: nested_selection,
                            parent_defer_label: selection.parent_defer_label,
                        })
                        .collect::<Vec<_>>(),
                    subset,
                    type_condition,
                );

                if defer.is_some() {
                    defer_stack.pop();
                }
            }
            Selection::FragmentSpread(spread) => {
                let Some(fragment) = spread.fragment() else { continue };

                if type_condition != Some(fragment.type_condition()) {
                    // TODO: This needs to be smarter.  If there's no type_condition it doesn't matter what typename
                    // is. We also need to handle implements properly, which will require the registry.
                    //
                    // Will revisit later though...
                    continue;
                }

                let defer = spread.defer_directive();
                if let Some(defer) = &defer {
                    defer_stack.push(defer.label);
                }

                collect_fields(
                    grouped_fields,
                    defer_stack,
                    &subset
                        .selection_iter(fragment.selection_set())
                        .map(|nested_selection| DeferrableSelection {
                            selection: nested_selection,
                            parent_defer_label: selection.parent_defer_label,
                        })
                        .collect::<Vec<_>>(),
                    subset,
                    type_condition,
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
    defer_label: Option<&'a str>,

    merged_selections: Vec<DeferrableSelection<'a>>,
}

/// Wrapper around a Selection that allows defer labels to be propagated where
/// neccesary
pub(super) struct DeferrableSelection<'a> {
    pub(super) selection: Selection<'a>,

    parent_defer_label: Option<&'a str>,
}

impl<'a> DeferrableSelection<'a> {
    pub fn without_defer(selection: Selection<'a>) -> Self {
        DeferrableSelection {
            selection,
            parent_defer_label: None,
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
fn merge_selection_sets<'a>(
    grouped_fields: IndexMap<&'a str, Vec<CollectedField<'a>>>,
    subset: &'a QuerySubset,
) -> Vec<MergedField<'a>> {
    let mut output = Vec::with_capacity(grouped_fields.len());
    for (response_key, fields) in grouped_fields {
        if fields.len() == 1 {
            // Hooray, the easy case
            output.push(MergedField {
                response_key,
                defer_label: fields[0].defer_label,
                merged_selections: subset
                    .selection_iter(fields[0].field.selection_set())
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
                defer_label: fields[0].defer_label,
                merged_selections: vec![],
            });
            continue;
        }

        // If there's any mismatch on defer labels we want to propagate them to child fields instead
        // of putting them on this level of the heirarchy.
        let all_defers_match = match fields.as_slice() {
            [head, tail @ ..] => tail.iter().all(|x| x.defer_label == head.defer_label),
            [] => unreachable!(),
        };

        let mut merged_field = MergedField {
            response_key: "",
            defer_label: None,
            merged_selections: vec![],
        };

        for field in fields {
            merged_field.response_key = field.field.response_key();
            let selections = subset.selection_iter(field.field.selection_set());
            if all_defers_match {
                merged_field.defer_label = field.defer_label;
                merged_field
                    .merged_selections
                    .extend(selections.map(DeferrableSelection::without_defer));
            } else {
                merged_field
                    .merged_selections
                    .extend(selections.map(|selection| DeferrableSelection {
                        selection,
                        parent_defer_label: merged_field.defer_label,
                    }));
            }
        }
    }

    output
}

#[derive(Default)]
struct OutputShapesBuilder {
    objects: Vec<ObjectShapeRecord>,
}

impl OutputShapesBuilder {
    fn insert_concrete_object(&mut self, fields: Vec<FieldRecord>) -> ObjectShapeId {
        self.insert_record(ObjectShapeRecord::Concrete { fields })
    }

    fn insert_polymorphic_object(
        &mut self,
        fields_when_no_condition_matches: Vec<FieldRecord>,
        fields_for_typeconditions: Vec<(String, Vec<FieldRecord>)>,
    ) -> ObjectShapeId {
        let mut types = Vec::with_capacity(fields_for_typeconditions.len() + 1);
        types.push((None, self.insert_concrete_object(fields_when_no_condition_matches)));
        for (typename, fields) in fields_for_typeconditions {
            types.push((Some(typename), self.insert_concrete_object(fields)));
        }

        self.insert_record(ObjectShapeRecord::Polymorphic { types })
    }

    fn insert_record(&mut self, record: ObjectShapeRecord) -> ObjectShapeId {
        let id = ObjectShapeId(u16::try_from(self.objects.len()).expect("too many objects, what the hell"));
        self.objects.push(record);
        id
    }
}

impl CachingPlan {
    fn cache_partitions(&self) -> impl Iterator<Item = (&QuerySubset, FilteredSelectionSet<'_, '_>)> + '_ {
        self.cache_partitions.iter().map(|(_, subset)| {
            let operation = self.document.read(subset.operation);
            (subset, subset.selection_iter(operation.selection_set()))
        })
    }

    fn nocache_partition(&self) -> (&QuerySubset, FilteredSelectionSet<'_, '_>) {
        let operation = self.document.read(self.nocache_partition.operation);
        (
            &self.nocache_partition,
            self.nocache_partition.selection_iter(operation.selection_set()),
        )
    }
}
