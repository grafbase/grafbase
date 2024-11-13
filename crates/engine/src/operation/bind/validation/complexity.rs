use engine_parser::types::OperationType;
use schema::Schema;

use crate::operation::{FieldWalker, SelectionSetWalker};

use super::{BindResult, OperationWalker};

pub fn control_complexity(schema: &Schema, operation: OperationWalker<'_>) -> BindResult<()> {
    if schema.settings.complexity_control.is_disabled() {
        return Ok(());
    }

    let base_cost = match operation.operation.ty {
        OperationType::Query | OperationType::Subscription => 0,
        OperationType::Mutation => 10,
    };

    let selection_set = operation.selection_set();

    todo!()
}

struct ComplexitySettings {
    list_size: usize,
}

fn selection_set_complexity(settings: &ComplexitySettings, selection_set: SelectionSetWalker<'_>) -> usize {
    selection_set
        .fields()
        .map(|field| field_complexity(settings, field))
        .sum()
}

fn field_complexity(settings: &ComplexitySettings, field: FieldWalker<'_>) -> usize {
    // TODO: If field is covered by skip/include then probably handle that?
    //
    // TODO: Handle propagation of list_size based on parent field/argument?
    //
    // TODO: Handle cost of the field itself:
    // If the field has a cost directive use that
    // If the type of the field has a cost directive use that instead
    // Otherwise: scalars/enums are free. interfaces, object, union are 1 by default
    //
    // Do we also want to handle the cost decalred on the individual types?
    // I would think yes, but looking at apollo it doesn't seem like they do that...

    // TODO: Consider estimating requires..
    //
    // TODO: Cost at least as far as apollo cares is `(list_size * (type_cost + argument_cost * requirement_cost))`
    todo!()
}
