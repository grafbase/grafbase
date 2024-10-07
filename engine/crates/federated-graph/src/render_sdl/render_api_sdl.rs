use super::display_utils::*;
use crate::{federated_graph::*, FederatedGraph};
use std::fmt::{self, Write as _};

/// Render a GraphQL SDL string for a federated graph. It does not include any
/// federation-specific directives, it only reflects the final API schema as visible
/// for consumers.
pub fn render_api_sdl(graph: &FederatedGraph) -> String {
    Renderer { graph }.to_string()
}

struct Renderer<'a> {
    graph: &'a FederatedGraph,
}

impl fmt::Display for Renderer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Renderer { graph } = self;

        // For spaces between blocks, to avoid a leading newline at the beginning of the file.
        let mut write_leading_whitespace = {
            let mut first_block = true;
            move |f: &mut fmt::Formatter<'_>| {
                if first_block {
                    first_block = false;
                    Ok(())
                } else {
                    f.write_char('\n')
                }
            }
        };

        for r#enum in graph.iter_enums() {
            if has_inaccessible(&r#enum.directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, r#enum.description, "", graph)?;
            f.write_str("enum ")?;
            f.write_str(&graph[r#enum.name])?;
            write_public_directives(f, r#enum.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for variant in graph.iter_enum_values(r#enum.id()) {
                    if has_inaccessible(&variant.composed_directives, graph) {
                        continue;
                    }

                    write_enum_variant(f, &variant, graph)?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for object in &graph.objects {
            let definition = graph.at(object.type_definition_id);

            if has_inaccessible(&definition.directives, graph) {
                continue;
            }

            if graph[object.fields.clone()].iter().all(|field| {
                let field_name = &graph[field.name];
                field_name.starts_with("__") || has_inaccessible(&field.composed_directives, graph)
            }) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, definition.description, "", graph)?;
            f.write_str("type ")?;
            f.write_str(definition.then(|def| def.name).as_str())?;
            write_public_directives(f, definition.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for (idx, field) in graph[object.fields.clone()].iter().enumerate() {
                    let field_name = &graph[field.name];

                    if field_name.starts_with("__") || has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    let field_id = FieldId(object.fields.start.0 + idx);

                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    write_field_arguments(f, graph.iter_field_arguments(field_id), graph)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_public_directives(f, field.composed_directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for interface in &graph.interfaces {
            let definition = graph.at(interface.type_definition_id);

            if has_inaccessible(&definition.directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, definition.description, "", graph)?;
            f.write_str("interface ")?;
            f.write_str(definition.then(|def| def.name).as_str())?;
            write_public_directives(f, definition.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for (idx, field) in graph[interface.fields.clone()].iter().enumerate() {
                    if has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    let field_name = &graph[field.name];
                    let field_id = FieldId(interface.fields.start.0 + idx);

                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_field_arguments(f, graph.iter_field_arguments(field_id), graph)?;
                    write_public_directives(f, field.composed_directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for input_object in graph.iter_input_objects() {
            if has_inaccessible(&input_object.directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, input_object.description, "", graph)?;
            f.write_str("input ")?;
            f.write_str(&graph[input_object.name])?;
            write_public_directives(f, input_object.directives, graph)?;

            f.write_char(' ')?;

            write_block(f, |f| {
                for field in graph.iter_input_object_fields(input_object.id()) {
                    let input_value_definition = field.then(|f| f.input_value_definition_id);
                    if has_inaccessible(&input_value_definition.directives, graph) {
                        continue;
                    }

                    write_description(f, input_value_definition.description, INDENT, graph)?;
                    let field_name = &graph[input_value_definition.name];
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&input_value_definition.r#type, graph))?;

                    if let Some(default) = &input_value_definition.default {
                        write!(f, " = {}", ValueDisplay(default, graph))?;
                    }

                    write_public_directives(f, input_value_definition.directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for union in &graph.unions {
            if has_inaccessible(&union.composed_directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, union.description, "", graph)?;
            f.write_str("union ")?;
            f.write_str(&graph[union.name])?;
            write_public_directives(f, union.composed_directives, graph)?;
            f.write_str(" =")?;

            let mut members = union.members.iter().peekable();

            while let Some(member) = members.next() {
                f.write_str(" ")?;
                f.write_str(
                    graph
                        .at(*member)
                        .then(|obj| obj.type_definition_id)
                        .then(|def| def.name)
                        .as_str(),
                )?;

                if members.peek().is_some() {
                    f.write_str(" |")?;
                }
            }

            f.write_char('\n')?;
        }

        for scalar in graph.iter_scalars() {
            let scalar_name = scalar.then(|scalar| scalar.name).as_str();

            if BUILTIN_SCALARS.contains(&scalar_name) || has_inaccessible(&scalar.directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, scalar.description, "", graph)?;
            f.write_str("scalar ")?;
            f.write_str(scalar_name)?;
            write_public_directives(f, scalar.directives, graph)?;

            f.write_char('\n')?;
        }

        Ok(())
    }
}

fn has_inaccessible(directives: &Directives, graph: &FederatedGraph) -> bool {
    graph[*directives]
        .iter()
        .any(|directive| matches!(directive, Directive::Inaccessible))
}

fn write_public_directives<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    directives: Directives,
    graph: &'a FederatedGraph,
) -> fmt::Result {
    for directive in graph[directives].iter().filter(|directive| match directive {
        Directive::Inaccessible | Directive::Policy(_) => false,

        Directive::Other { name, .. } if graph[*name] == "tag" => false,
        Directive::RequiresScopes(_)
        | Directive::Authenticated
        | Directive::Deprecated { .. }
        | Directive::Other { .. } => true,
    }) {
        f.write_str(" ")?;
        write_composed_directive(f, directive, graph)?;
    }

    Ok(())
}

fn write_enum_variant<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    enum_variant: &EnumValueRecord,
    graph: &'a FederatedGraph,
) -> fmt::Result {
    f.write_str(INDENT)?;
    write_description(f, enum_variant.description, INDENT, graph)?;
    f.write_str(&graph[enum_variant.value])?;
    write_public_directives(f, enum_variant.composed_directives, graph)?;
    f.write_char('\n')
}

fn write_field_arguments<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    args: impl Iterator<Item = ArgumentDefinition<'a>>,
    graph: &'a FederatedGraph,
) -> fmt::Result {
    let mut args = args
        .map(|arg| {
            let arg = arg.then(|arg| arg.input_value_definition_id);
            let name = &graph[arg.name];
            let r#type = render_field_type(&arg.r#type, graph);
            let directives = arg.directives;
            let default = arg.default.as_ref();
            (name, r#type, directives, default)
        })
        .peekable();

    if args.peek().is_none() {
        return Ok(());
    }

    f.write_str("(")?;

    while let Some((name, ty, directives, default)) = args.next() {
        f.write_str(name)?;
        f.write_str(": ")?;
        f.write_str(&ty)?;

        if let Some(default) = default {
            write!(f, " = {}", ValueDisplay(default, graph))?;
        }

        write_public_directives(f, directives, graph)?;

        if args.peek().is_some() {
            f.write_str(", ")?;
        }
    }

    f.write_str(")")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let empty = FederatedGraph::default();
        let sdl = render_api_sdl(&empty);
        assert!(sdl.is_empty());
    }
}
