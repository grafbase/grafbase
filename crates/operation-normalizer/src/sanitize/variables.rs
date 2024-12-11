use cynic_parser::executable::{Iter, VariableDefinition};

pub(super) fn sanitize(variables: Iter<'_, VariableDefinition<'_>>, rendered: &mut String) {
    let variables_count = variables.len();
    for (i, variable_definition) in variables.enumerate() {
        if i == 0 {
            rendered.push('(');
        }

        rendered.push('$');
        rendered.push_str(variable_definition.name());
        rendered.push_str(": ");
        rendered.push_str(&variable_definition.ty().to_string());

        if i == variables_count - 1 {
            rendered.push(')');
        } else {
            rendered.push_str(", ");
        }
    }
}
