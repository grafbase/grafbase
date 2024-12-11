use cynic_parser::executable::FragmentDefinition;

pub(super) fn sanitize(definition: &FragmentDefinition<'_>, rendered: &mut String) {
    rendered.push_str("fragment ");
    rendered.push_str(definition.name());
    rendered.push_str(" on ");
    rendered.push_str(definition.type_condition());

    super::directives::sanitize(definition.directives(), rendered);
    super::selection::sanitize(definition.selection_set(), rendered);
}
