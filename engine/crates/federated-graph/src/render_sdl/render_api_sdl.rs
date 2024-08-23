use super::display_utils::*;
use crate::{federated_graph::*, FederatedGraphV4};
use std::fmt::{self, Write as _};

/// Render a GraphQL SDL string for a federated graph. It does not include any
/// federation-specific directives, it only reflects the final API schema as visible
/// for consumers.
pub fn render_api_sdl(graph: &FederatedGraphV4) -> String {
    Renderer { graph }.to_string()
}

struct Renderer<'a> {
    graph: &'a FederatedGraphV4,
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

        for r#enum in &graph.enums {
            if has_inaccessible(&r#enum.composed_directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, r#enum.description, "", graph)?;
            f.write_str("enum ")?;
            f.write_str(&graph[r#enum.name])?;
            write_public_directives(f, r#enum.composed_directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for variant in &graph[r#enum.values] {
                    if has_inaccessible(&variant.composed_directives, graph) {
                        continue;
                    }

                    write_enum_variant(f, variant, graph)?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for object in &graph.objects {
            if has_inaccessible(&object.composed_directives, graph) {
                continue;
            }

            if graph[object.fields.clone()].iter().all(|field| {
                let field_name = &graph[field.name];
                field_name.starts_with("__") || has_inaccessible(&field.composed_directives, graph)
            }) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, object.description, "", graph)?;
            f.write_str("type ")?;
            f.write_str(&graph[object.name])?;
            write_public_directives(f, object.composed_directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[object.fields.clone()] {
                    let field_name = &graph[field.name];

                    if field_name.starts_with("__") || has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    write_field_arguments(f, &graph[field.arguments], graph)?;
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
            if has_inaccessible(&interface.composed_directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, interface.description, "", graph)?;
            f.write_str("interface ")?;
            f.write_str(&graph[interface.name])?;
            write_public_directives(f, interface.composed_directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[interface.fields.clone()] {
                    if has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    let field_name = &graph[field.name];
                    write_description(f, field.description, INDENT, graph)?;
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_field_arguments(f, &graph[field.arguments], graph)?;
                    write_public_directives(f, field.composed_directives, graph)?;
                    f.write_char('\n')?;
                }

                Ok(())
            })?;

            f.write_char('\n')?;
        }

        for input_object in &graph.input_objects {
            if has_inaccessible(&input_object.composed_directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, input_object.description, "", graph)?;
            f.write_str("input ")?;
            f.write_str(&graph[input_object.name])?;
            write_public_directives(f, input_object.composed_directives, graph)?;

            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[input_object.fields] {
                    if has_inaccessible(&field.directives, graph) {
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

                    write_public_directives(f, field.directives, graph)?;
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
                f.write_str(&graph[graph[*member].name])?;

                if members.peek().is_some() {
                    f.write_str(" |")?;
                }
            }

            f.write_char('\n')?;
        }

        for scalar in &graph.scalars {
            let scalar_name = &graph[scalar.name];

            if BUILTIN_SCALARS.contains(&scalar_name.as_str()) || has_inaccessible(&scalar.composed_directives, graph) {
                continue;
            }

            write_leading_whitespace(f)?;

            write_description(f, scalar.description, "", graph)?;
            f.write_str("scalar ")?;
            f.write_str(scalar_name)?;
            write_public_directives(f, scalar.composed_directives, graph)?;

            f.write_char('\n')?;
        }

        Ok(())
    }
}

fn has_inaccessible(directives: &Directives, graph: &FederatedGraphV4) -> bool {
    graph[*directives]
        .iter()
        .any(|directive| matches!(directive, Directive::Inaccessible))
}

fn write_public_directives(
    f: &mut fmt::Formatter<'_>,
    directives: Directives,
    graph: &FederatedGraphV4,
) -> fmt::Result {
    for directive in graph[directives].iter().filter(|directive| match directive {
        Directive::Inaccessible | Directive::Policy(_) => false,

        Directive::Other { name, .. } if graph[*name] == "tag" => false,
        Directive::RequiresScopes(_)
        | Directive::Authenticated
        | Directive::Deprecated { .. }
        | Directive::Other { .. } => true,
    }) {
        write_composed_directive(f, directive, graph)?;
    }

    Ok(())
}

fn write_enum_variant(f: &mut fmt::Formatter<'_>, enum_variant: &EnumValue, graph: &FederatedGraphV4) -> fmt::Result {
    f.write_str(INDENT)?;
    write_description(f, enum_variant.description, INDENT, graph)?;
    f.write_str(&graph[enum_variant.value])?;
    write_public_directives(f, enum_variant.composed_directives, graph)?;
    f.write_char('\n')
}

fn write_field_arguments(
    f: &mut fmt::Formatter<'_>,
    args: &[InputValueDefinition],
    graph: &FederatedGraphV4,
) -> fmt::Result {
    if args.is_empty() {
        return Ok(());
    }

    let mut inner = args
        .iter()
        .map(|arg| {
            let name = &graph[arg.name];
            let r#type = render_field_type(&arg.r#type, graph);
            let directives = arg.directives;
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
        let empty = FederatedGraphV4::default();
        let sdl = render_api_sdl(&empty);
        assert!(sdl.is_empty());
    }
}
