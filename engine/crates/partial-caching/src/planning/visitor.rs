use cynic_parser::{
    common::OperationType,
    executable::{
        ids::SelectionId, iter::Iter, FieldSelection, FragmentDefinition, FragmentSpread, OperationDefinition,
        Selection,
    },
};
use registry_for_cache::{MetaField, MetaType, PartialCacheRegistry};

#[allow(unused_variables)]
pub trait Visitor {
    fn enter_selection(&mut self, id: SelectionId, selection: Selection<'_>) {}
    fn exit_selection(&mut self, id: SelectionId, selection: Selection<'_>) {}

    fn enter_field(&mut self, edge: FieldEdge<'_>) {}
    fn exit_field(&mut self, edge: FieldEdge<'_>) {}

    fn fragment_spread(&mut self, spread: FragmentSpread<'_>) {}
}

#[derive(Clone, Copy, Debug)]
pub struct FieldEdge<'a> {
    pub selection: FieldSelection<'a>,
    #[allow(dead_code)]
    pub container: Option<MetaType<'a>>,
    pub field: Option<MetaField<'a>>,
    pub field_type: Option<MetaType<'a>>,
}

pub struct VisitorContext<'a> {
    visitors: &'a mut [&'a mut dyn Visitor],
}

impl<'a> VisitorContext<'a> {
    pub fn new(visitors: &'a mut [&'a mut dyn Visitor]) -> Self {
        VisitorContext { visitors }
    }

    fn visit(&mut self, f: impl Fn(&mut dyn Visitor)) {
        for visitor in self.visitors.iter_mut() {
            f(*visitor);
        }
    }
}

pub fn visit_query(query: OperationDefinition<'_>, registry: &PartialCacheRegistry, ctx: &mut VisitorContext<'_>) {
    assert_eq!(query.operation_type(), OperationType::Query);

    let ty = registry.query_type();
    visit_selection_set(query.selection_set(), Some(ty), registry, ctx)
}

pub fn visit_fragment(fragment: FragmentDefinition<'_>, registry: &PartialCacheRegistry, ctx: &mut VisitorContext<'_>) {
    let ty = registry.lookup_type(fragment.type_condition());

    visit_selection_set(fragment.selection_set(), ty, registry, ctx)
}

fn visit_selection_set(
    selections: Iter<'_, Selection<'_>>,
    container: Option<MetaType<'_>>,
    registry: &PartialCacheRegistry,
    ctx: &mut VisitorContext<'_>,
) {
    for (id, selection) in selections.ids().zip(selections) {
        ctx.visit(|visitor| visitor.enter_selection(id, selection));
        match selection {
            Selection::Field(selection) => {
                let field = container.and_then(|container| container.field(selection.name()));
                let field_type = field.map(|field| field.ty().named_type());
                let edge = FieldEdge {
                    selection,
                    container,
                    field,
                    field_type,
                };

                ctx.visit(|visitor| visitor.enter_field(edge));

                visit_selection_set(selection.selection_set(), field_type, registry, ctx);

                ctx.visit(|visitor| visitor.exit_field(edge));
            }
            Selection::InlineFragment(fragment) => {
                let new_container = match fragment.type_condition() {
                    Some(ty) => registry.lookup_type(ty),
                    None => container,
                };
                visit_selection_set(fragment.selection_set(), new_container, registry, ctx);
            }
            Selection::FragmentSpread(spread) => ctx.visit(|visitor| visitor.fragment_spread(spread)),
        }
        ctx.visit(|visitor| visitor.exit_selection(id, selection));
    }
}
