use super::{directive::write_directive, display_utils::*};
use crate::{directives::*, federated_graph::*, FederatedGraph};
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
            if has_inaccessible(&r#enum.directives) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, r#enum.description, "", graph)?;
            f.write_str("enum ")?;
            f.write_str(&graph[r#enum.name])?;
            write_public_directives(f, &r#enum.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for variant in graph.iter_enum_values(r#enum.id()) {
                    if has_inaccessible(&variant.directives) {
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

            if has_inaccessible(&definition.directives) {
                continue;
            }

            if graph[object.fields.clone()].iter().all(|field| {
                let field_name = &graph[field.name];
                field_name.starts_with("__") || has_inaccessible(&field.directives)
            }) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, definition.description, "", graph)?;
            f.write_str("type ")?;
            f.write_str(definition.then(|def| def.name).as_str())?;
            write_public_directives(f, &definition.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[object.fields.clone()] {
                    let field_name = &graph[field.name];

                    if field_name.starts_with("__") || has_inaccessible(&field.directives) {
                        continue;
                    }

                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    write_field_arguments(f, &graph[field.arguments], graph)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_public_directives(f, &field.directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for interface in &graph.interfaces {
            let definition = graph.at(interface.type_definition_id);

            if has_inaccessible(&definition.directives) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, definition.description, "", graph)?;
            f.write_str("interface ")?;
            f.write_str(definition.then(|def| def.name).as_str())?;
            write_public_directives(f, &definition.directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[interface.fields.clone()] {
                    if has_inaccessible(&field.directives) {
                        continue;
                    }

                    let field_name = &graph[field.name];
                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    write_field_arguments(f, &graph[field.arguments], graph)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_public_directives(f, &field.directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for input_object in &graph.input_objects {
            if has_inaccessible(&input_object.directives) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, input_object.description, "", graph)?;
            f.write_str("input ")?;
            f.write_str(&graph[input_object.name])?;
            write_public_directives(f, &input_object.directives, graph)?;

            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[input_object.fields] {
                    if has_inaccessible(&field.directives) {
                        continue;
                    }

                    write_description(f, field.description, INDENT, graph)?;
                    let field_name = &graph[field.name];
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;

                    if let Some(default) = &field.default {
                        write!(f, " = {}", ValueDisplay(default, graph))?;
                    }

                    write_public_directives(f, &field.directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for union in &graph.unions {
            if has_inaccessible(&union.directives) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, union.description, "", graph)?;
            f.write_str("union ")?;
            f.write_str(&graph[union.name])?;
            write_public_directives(f, &union.directives, graph)?;
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

            if BUILTIN_SCALARS.contains(&scalar_name) || has_inaccessible(&scalar.directives) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, scalar.description, "", graph)?;
            f.write_str("scalar ")?;
            f.write_str(scalar_name)?;
            write_public_directives(f, &scalar.directives, graph)?;

            f.write_char('\n')?;
        }

        Ok(())
    }
}

fn has_inaccessible(directives: &[Directive]) -> bool {
    directives
        .iter()
        .any(|directive| matches!(directive, Directive::Inaccessible))
}

fn write_public_directives<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    directives: &[Directive],
    graph: &'a FederatedGraph,
) -> fmt::Result {
    for directive in directives.iter().filter(|directive| match directive {
        Directive::Inaccessible
        | Directive::Policy(_)
        | Directive::RequiresScopes(_)
        | Directive::Authenticated
        | Directive::Cost { .. }
        | Directive::JoinField(_)
        | Directive::JoinType(_)
        | Directive::JoinUnionMember(_)
        | Directive::JoinImplements(_)
        | Directive::Authorized(_)
        | Directive::ListSize(_) => false,

        Directive::Other { name, .. } if graph[*name] == "tag" => false,
        Directive::Deprecated { .. } | Directive::Other { .. } => true,
    }) {
        f.write_str(" ")?;
        write_directive(f, directive, graph)?;
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
    write_public_directives(f, &enum_variant.directives, graph)?;
    f.write_char('\n')
}

fn write_field_arguments<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    args: &[InputValueDefinition],
    graph: &'a FederatedGraph,
) -> fmt::Result {
    if args.is_empty() {
        return Ok(());
    }

    let mut inner = args
        .iter()
        .map(|arg| {
            let name = &graph[arg.name];
            let r#type = render_field_type(&arg.r#type, graph);
            let directives = &arg.directives;
            let default = arg.default.as_ref();
            (name, r#type, directives, default)
        })
        .peekable();

    f.write_str("(")?;

    while let Some((name, ty, directives, default)) = inner.next() {
        f.write_str(name)?;
        f.write_str(": ")?;
        f.write_str(&ty)?;

        if let Some(default) = default {
            write!(f, " = {}", ValueDisplay(default, graph))?;
        }

        write_public_directives(f, directives, graph)?;

        if inner.peek().is_some() {
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
