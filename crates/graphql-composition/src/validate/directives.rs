use super::*;

pub(super) fn validate(context: &mut ValidateContext<'_>) {
    for (_, directive) in context.subgraphs.iter_extra_directives_on_schema_definition() {
        let subgraphs::DirectiveProvenance::Linked {
            linked_schema_id,
            is_composed_directive,
        } = directive.provenance
        else {
            continue;
        };

        if let Some(extension_id) = context.get_extension_for_linked_schema(linked_schema_id) {
            context.mark_used_extension(extension_id);
        } else if !is_composed_directive {
            context.diagnostics.push_warning(format!(
                "Directive `{}` is not defined in any extension or composed directive",
                &context[directive.name]
            ));
        }
    }
}
