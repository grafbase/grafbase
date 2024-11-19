use engine_parser::types::OperationType;
use schema::Schema;

use crate::{
    operation::{FieldWalker, OperationWalker, SelectionSetWalker},
    prepare::error::{PrepareError, PrepareResult},
};

use super::Variables;

pub fn control_complexity(schema: &Schema, operation: OperationWalker<'_>, variables: &Variables) -> PrepareResult<()> {
    if schema.settings.complexity_control.is_disabled() {
        return Ok(());
    }

    let base_cost = match operation.operation.ty {
        OperationType::Query | OperationType::Subscription => 0,
        OperationType::Mutation => 10,
    };

    let selection_set = operation.selection_set();

    let context = ComplexityContext {
        default_list_size: schema
            .settings
            .complexity_control
            .list_size()
            .expect("should be some unless disabled"),
        variables,
    };

    let cost = base_cost + selection_set_complexity(&context, selection_set, None);

    eprintln!("Complexity: {cost}");

    if let Some(limit) = schema.settings.complexity_control.limit() {
        if cost > limit && schema.settings.complexity_control.is_enforce() {
            return Err(PrepareError::ComplexityLimitReached);
        }
    }

    Ok(())
}

struct ComplexityContext<'a> {
    default_list_size: usize,
    variables: &'a Variables,
}

fn selection_set_complexity(
    context: &ComplexityContext<'_>,
    selection_set: SelectionSetWalker<'_>,
    field_multipliers: Option<FieldMultipliers<'_>>,
) -> usize {
    selection_set
        .fields()
        .map(|field| {
            let preset_list_size = field_multipliers.as_ref().and_then(|child_fields| {
                child_fields.fields.iter().find_map(|child_field| {
                    if field.name() == *child_field {
                        Some(child_fields.multiplier)
                    } else {
                        None
                    }
                })
            });
            field_complexity(context, field, preset_list_size)
        })
        .sum()
}

fn field_complexity(context: &ComplexityContext<'_>, field: FieldWalker<'_>, preset_list_size: Option<usize>) -> usize {
    let type_cost = field
        .definition()
        .map(|def| dbg!(def.cost()).unwrap_or_else(|| cost_for_type(def.ty().definition())))
        .unwrap_or(1) as usize;

    let list_size_directive = field.definition().and_then(|def| def.list_size());

    let child_count = calculate_child_count(context, field, list_size_directive, preset_list_size);

    let argument_cost = field
        .arguments()
        .into_iter()
        .map(|argument| cost_for_argument(argument, context.variables))
        .sum::<usize>();

    let this_field_count = child_count.this_field_count();
    let child_field_count = child_count.child_field_count();

    let child_cost = field
        .selection_set()
        .map(|selection_set| selection_set_complexity(context, selection_set, child_field_count))
        .unwrap_or_default();

    dbg!(this_field_count) * (dbg!(type_cost) + dbg!(argument_cost) + dbg!(child_cost))
}

fn cost_for_argument(
    argument: OperationWalker<'_, crate::operation::BoundFieldArgumentId>,
    _variables: &Variables,
) -> usize {
    let def = argument.definition();
    let argument_type = def.ty().definition();
    let argument_cost = def.cost().unwrap_or_else(|| cost_for_type(argument_type));

    argument_cost as usize
}

enum ListSizeHandling<'a> {
    ThisFieldIsTheList(usize),
    ChildFieldsAreTheList(FieldMultipliers<'a>),
}

impl<'a> ListSizeHandling<'a> {
    pub fn this_field_count(&self) -> usize {
        match self {
            ListSizeHandling::ThisFieldIsTheList(count) => *count,
            ListSizeHandling::ChildFieldsAreTheList(_) => 1,
        }
    }

    pub fn child_field_count(self) -> Option<FieldMultipliers<'a>> {
        match self {
            ListSizeHandling::ChildFieldsAreTheList(count) => Some(count),
            _ => None,
        }
    }
}

struct FieldMultipliers<'a> {
    multiplier: usize,
    fields: Vec<&'a str>,
}

fn calculate_child_count<'a>(
    context: &ComplexityContext<'_>,
    field: OperationWalker<'_, crate::operation::BoundFieldId>,
    list_size_directive: Option<schema::ListSizeDirective<'a>>,
    preset_list_size: Option<usize>,
) -> ListSizeHandling<'a> {
    let field_is_list = field
        .definition()
        .map(|def| def.ty().wrapping.is_list())
        .unwrap_or_default();

    if !field_is_list {
        return ListSizeHandling::ThisFieldIsTheList(1);
    }

    if let Some(size) = preset_list_size {
        return ListSizeHandling::ThisFieldIsTheList(size);
    }

    let Some(directive) = list_size_directive else {
        return ListSizeHandling::ThisFieldIsTheList(context.default_list_size);
    };

    let mut multiplier = directive.assumed_size.unwrap_or(context.default_list_size as u32) as usize;

    let mut slicing_arguments = directive.slicing_arguments().peekable();
    if slicing_arguments.peek().is_some() {
        let argument_size = slicing_arguments
            .filter_map(|argument| field.argument(argument.name())?.value(context.variables).as_usize())
            .max();

        if argument_size.is_none() && directive.require_one_slicing_argument {
            todo!("error")
        }

        multiplier = argument_size.unwrap_or(context.default_list_size);
    }

    let mut sized_fields = directive.sized_fields().peekable();
    if sized_fields.peek().is_none() {
        return ListSizeHandling::ThisFieldIsTheList(multiplier);
    }

    ListSizeHandling::ChildFieldsAreTheList(FieldMultipliers {
        multiplier,
        fields: sized_fields.map(|sized_field| sized_field.name()).collect(),
    })
}

fn cost_for_type(definition: schema::Definition<'_>) -> i32 {
    if let Some(cost) = definition.cost() {
        return cost;
    }

    match definition {
        schema::Definition::Enum(_) | schema::Definition::Scalar(_) => 0,
        schema::Definition::Interface(_) | schema::Definition::Object(_) | schema::Definition::Union(_) => 1,
        schema::Definition::InputObject(_) => 1,
    }
}
