use cynic_parser::executable::{Directive, Iter};

pub(super) fn sanitize(directives: Iter<'_, Directive<'_>>, rendered: &mut String) {
    for directive in directives {
        rendered.push_str(" @");
        rendered.push_str(directive.name());

        let arguments = directive.arguments();
        let arguments_count = arguments.len();

        for (i, argument) in arguments.enumerate() {
            if i == 0 {
                rendered.push('(');
            }

            rendered.push_str(argument.name());
            rendered.push_str(": ");

            super::value::sanitize(argument.value(), rendered);

            if i == arguments_count - 1 {
                rendered.push(')');
            } else {
                rendered.push(',');
            }
        }
    }
}
