use cynic_parser::{
    executable::{ids::VariableDefinitionId, Argument, Directive, OperationDefinition},
    ExecutableDocument,
};
use indexmap::IndexSet;

use crate::query_subset::Partition;

pub fn variables_required(
    partition: &Partition,
    document: &ExecutableDocument,
    operation: OperationDefinition<'_>,
) -> IndexSet<VariableDefinitionId> {
    let mut variables = partition
        .selections
        .iter()
        .flat_map(|id| -> Box<dyn Iterator<Item = &str>> {
            match document.read(*id) {
                cynic_parser::executable::Selection::Field(field) => Box::new(
                    variables_used_in_argument(field.arguments())
                        .chain(variables_used_in_directive(field.directives())),
                ),
                cynic_parser::executable::Selection::InlineFragment(fragment) => {
                    Box::new(variables_used_in_directive(fragment.directives()))
                }
                cynic_parser::executable::Selection::FragmentSpread(spread) => {
                    Box::new(variables_used_in_directive(spread.directives()))
                }
            }
        })
        .collect::<IndexSet<_>>();

    variables.extend(partition.fragments.iter().flat_map(|id| {
        let fragment = document.read(*id);
        variables_used_in_directive(fragment.directives())
    }));

    variables.extend(variables_used_in_directive(operation.directives()));

    variables
        .into_iter()
        .flat_map(|name| {
            Some(
                operation
                    .variable_definitions()
                    .find(|variable| variable.name() == name)?
                    .id(),
            )
        })
        .collect()
}

fn variables_used_in_directive<'a>(directives: impl Iterator<Item = Directive<'a>>) -> impl Iterator<Item = &'a str> {
    directives.flat_map(|directive| variables_used_in_argument(directive.arguments()))
}

fn variables_used_in_argument<'a>(arguments: impl Iterator<Item = Argument<'a>>) -> impl Iterator<Item = &'a str> {
    arguments.flat_map(|argument| argument.value().variables_used().collect::<Vec<_>>())
}
