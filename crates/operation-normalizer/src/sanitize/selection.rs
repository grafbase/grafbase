use cynic_parser::executable::{Iter, Selection};

pub(crate) fn sanitize(selection_set: Iter<'_, Selection<'_>>, rendered: &mut String) {
    let selection_count = selection_set.len();
    for (i, selection) in selection_set.enumerate() {
        if i == 0 {
            rendered.push_str(" {");
        }

        match selection {
            Selection::Field(selection) => {
                rendered.push(' ');

                if let Some(alias) = selection.alias() {
                    rendered.push_str(alias);
                    rendered.push_str(": ");
                }

                rendered.push_str(selection.name());

                let arguments_count = selection.arguments().len();

                for (i, argument) in selection.arguments().enumerate() {
                    if i == 0 {
                        rendered.push('(');
                    }

                    rendered.push_str(argument.name());
                    rendered.push_str(": ");

                    super::value::sanitize(argument.value(), rendered);

                    if i == arguments_count - 1 {
                        rendered.push(')');
                    } else {
                        rendered.push_str(", ");
                    }
                }

                super::directives::sanitize(selection.directives(), rendered);
                super::selection::sanitize(selection.selection_set(), rendered);
            }
            Selection::InlineFragment(inline_fragment) => {
                rendered.push_str(" ...");

                if let Some(r#type) = inline_fragment.type_condition() {
                    rendered.push_str(" on ");
                    rendered.push_str(r#type);
                }

                super::directives::sanitize(inline_fragment.directives(), rendered);
                sanitize(inline_fragment.selection_set(), rendered);
            }
            Selection::FragmentSpread(fragment_spread) => {
                rendered.push_str(" ...");
                rendered.push_str(fragment_spread.fragment_name());
                rendered.push(' ');
            }
        }

        if i == selection_count - 1 {
            rendered.push_str(" }");
        }
    }
}
