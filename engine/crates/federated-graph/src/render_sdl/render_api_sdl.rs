use super::display_utils::*;
use crate::{federated_graph::*, FederatedGraphV3};
use std::fmt::{self, Write as _};

/// Render a GraphQL SDL string for a federated graph. It does not include any
/// federation-specific directives, it only reflects the final API schema as visible
/// for consumers.
pub fn render_api_sdl(graph: &FederatedGraphV3) -> String {
    Renderer { graph }.to_string()
}

struct Renderer<'a> {
    graph: &'a FederatedGraphV3,
}

impl fmt::Display for Renderer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Renderer { graph } = self;
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

            f.write_str("enum ")?;
            f.write_str(&graph[r#enum.name])?;
            write_directives(f, r#enum.composed_directives, graph)?;
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

            write_leading_whitespace(f)?;

            f.write_str("type ")?;
            f.write_str(&graph[object.name])?;
            write_directives(f, object.composed_directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[object.fields.clone()] {
                    let field_name = &graph[field.name];

                    if field_name.starts_with("__") || has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_directives(f, field.composed_directives, graph)?;
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

            f.write_str("interface ")?;
            f.write_str(&graph[interface.name])?;
            write_directives(f, interface.composed_directives, graph)?;
            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[interface.fields.clone()] {
                    if has_inaccessible(&field.composed_directives, graph) {
                        continue;
                    }

                    let field_name = &graph[field.name];
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_directives(f, field.composed_directives, graph)?;
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

            f.write_str("input ")?;
            f.write_str(&graph[input_object.name])?;
            write_directives(f, input_object.composed_directives, graph)?;

            f.write_char(' ')?;

            write_block(f, |f| {
                for field in &graph[input_object.fields.clone()] {
                    if has_inaccessible(&field.directives, graph) {
                        continue;
                    }

                    let field_name = &graph[field.name];
                    f.write_str(INDENT)?;
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    f.write_str(&render_field_type(&field.r#type, graph))?;
                    write_directives(f, field.directives, graph)?;
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

            f.write_str("union ")?;
            f.write_str(&graph[union.name])?;
            write_directives(f, union.composed_directives, graph)?;
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

            f.write_str("scalar ")?;
            f.write_str(scalar_name)?;
            write_directives(f, scalar.composed_directives, graph)?;

            f.write_char('\n')?;
        }

        Ok(())
    }
}

fn has_inaccessible(directives: &Directives, graph: &FederatedGraphV3) -> bool {
    graph[directives.clone()]
        .iter()
        .any(|directive| matches!(directive, Directive::Inaccessible))
}
