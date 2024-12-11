mod directives;
mod fragment;
mod operation;
mod selection;
mod value;
mod variables;

use cynic_parser::{executable::ExecutableDefinition, ExecutableDocument};

/// Sanitizes a GraphQL document by removing all static data which could leak sensitive information.
pub fn sanitize(document: &ExecutableDocument) -> String {
    let mut rendered = String::new();

    let definitions = document.definitions();
    let definitions_count = definitions.len();

    for (i, definition) in definitions.enumerate() {
        match definition {
            ExecutableDefinition::Operation(operation) => operation::sanitize(&operation, &mut rendered),
            ExecutableDefinition::Fragment(definition) => fragment::sanitize(&definition, &mut rendered),
        }

        if i != definitions_count - 1 {
            rendered.push(' ');
        }
    }

    rendered
}
