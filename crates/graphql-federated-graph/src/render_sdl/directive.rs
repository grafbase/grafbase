use std::fmt;

use crate::{
    directives::{
        AuthorizedDirective, JoinFieldDirective, JoinImplementsDirective, JoinTypeDirective, JoinUnionMemberDirective,
    },
    render_sdl::display_utils::render_field_type,
    Directive, ExtensionDirective, FederatedGraph, OverrideLabel, OverrideSource, Value,
};

use super::{
    display_utils::{DirectiveWriter, GraphEnumVariantName, InputValueDefinitionSetDisplay, SelectionSetDisplay},
    render_federated_sdl::ListSizeRender,
};

pub(crate) fn write_directive<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    directive: &Directive,
    graph: &'a FederatedGraph,
) -> fmt::Result {
    match directive {
        Directive::Authenticated => {
            DirectiveWriter::new("authenticated", f, graph)?;
        }
        Directive::Inaccessible => {
            DirectiveWriter::new("inaccessible", f, graph)?;
        }
        Directive::Deprecated { reason } => {
            let directive = DirectiveWriter::new("deprecated", f, graph)?;

            if let Some(reason) = reason {
                directive.arg("reason", Value::String(*reason))?;
            }
        }
        Directive::Policy(policies) => {
            let policies = Value::List(
                policies
                    .iter()
                    .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                    .collect(),
            );

            DirectiveWriter::new("policy", f, graph)?.arg("policies", policies)?;
        }

        Directive::RequiresScopes(scopes) => {
            let scopes = Value::List(
                scopes
                    .iter()
                    .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                    .collect(),
            );

            DirectiveWriter::new("requiresScopes", f, graph)?.arg("scopes", scopes)?;
        }
        Directive::Cost { weight } => {
            DirectiveWriter::new("cost", f, graph)?.arg("weight", Value::Int(*weight as i64))?;
        }
        Directive::Other { name, arguments }
        | Directive::ExtensionDirective(ExtensionDirective { name, arguments, .. }) => {
            let mut directive = DirectiveWriter::new(&graph[*name], f, graph)?;

            for (name, value) in arguments {
                directive = directive.arg(&graph[*name], value.clone())?;
            }
        }
        Directive::JoinField(directive) => {
            render_join_field_directive(f, directive, graph)?;
        }
        Directive::JoinType(directive) => {
            render_join_type_directive(f, directive, graph)?;
        }
        Directive::JoinUnionMember(directive) => {
            render_join_union_member_directive(f, directive, graph)?;
        }
        Directive::JoinImplements(directive) => {
            render_join_implements_directive(f, directive, graph)?;
        }
        Directive::Authorized(directive) => {
            render_authorized_directive(f, directive, graph)?;
        }
        Directive::ListSize(list_size) => {
            f.write_fmt(format_args!("{}", ListSizeRender { list_size, graph }))?;
        }
    }

    Ok(())
}

fn render_join_union_member_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &JoinUnionMemberDirective,
    graph: &FederatedGraph,
) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[directive.subgraph_id].name]);

    DirectiveWriter::new("join__unionMember", f, graph)?
        .arg("graph", subgraph_name)?
        .arg("member", Value::String(graph.view(directive.object_id).name))?;

    Ok(())
}

fn render_join_field_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &JoinFieldDirective,
    graph: &FederatedGraph,
) -> fmt::Result {
    let mut writer = DirectiveWriter::new("join__field", f, graph)?;
    if let Some(subgraph_id) = directive.subgraph_id {
        let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph_id].name]);
        writer = writer.arg("graph", subgraph_name)?;
    }

    if let Some(requires) = directive.requires.as_ref().filter(|requires| !requires.is_empty()) {
        writer = writer.arg("requires", SelectionSetDisplay(requires, graph))?;
    }

    if let Some(provides) = directive.provides.as_ref().filter(|provides| !provides.is_empty()) {
        writer = writer.arg("provides", SelectionSetDisplay(provides, graph))?;
    }

    if let Some(ty) = &directive.r#type {
        writer = writer.arg("type", render_field_type(ty, graph))?;
    }

    if let Some(r#override) = &directive.r#override {
        let name = match r#override {
            OverrideSource::Subgraph(subgraph_id) => &graph.at(*subgraph_id).then(|subgraph| subgraph.name),
            OverrideSource::Missing(string) => &graph[*string],
        };
        writer = writer.arg("override", name.as_str())?;
    }

    if let Some(override_label) = &directive.override_label {
        match override_label {
            OverrideLabel::Percent(_) => writer.arg("overrideLabel", format!("{override_label}")),
            OverrideLabel::Unknown => writer.arg("overrideLabel", ""),
        }?;
    }

    Ok(())
}

fn render_join_type_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &JoinTypeDirective,
    graph: &FederatedGraph,
) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[directive.subgraph_id].name]);

    let mut writer = DirectiveWriter::new("join__type", f, graph)?.arg("graph", subgraph_name)?;

    if let Some(key) = directive.key.as_ref().filter(|key| !key.is_empty()) {
        writer = writer.arg("key", SelectionSetDisplay(key, graph))?;
    }

    if !directive.resolvable {
        writer = writer.arg("resolvable", Value::Boolean(false))?;
    }

    if directive.is_interface_object {
        writer.arg("isInterfaceObject", Value::Boolean(true))?;
    }

    Ok(())
}

fn render_join_implements_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &JoinImplementsDirective,
    graph: &FederatedGraph,
) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[directive.subgraph_id].name]);

    DirectiveWriter::new("join__implements", f, graph)?
        .arg("graph", subgraph_name)?
        .arg("interface", Value::String(graph.view(directive.interface_id).name))?;

    Ok(())
}

fn render_authorized_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &AuthorizedDirective,
    graph: &FederatedGraph,
) -> fmt::Result {
    let mut writer = DirectiveWriter::new("authorized", f, graph)?;
    if let Some(fields) = directive.fields.as_ref() {
        writer = writer.arg("fields", SelectionSetDisplay(fields, graph))?;
    }
    if let Some(node) = directive.node.as_ref() {
        writer = writer.arg("node", SelectionSetDisplay(node, graph))?;
    }
    if let Some(arguments) = directive.arguments.as_ref() {
        writer = writer.arg("arguments", InputValueDefinitionSetDisplay(arguments, graph))?;
    }
    if let Some(metadata) = directive.metadata.as_ref() {
        writer.arg("metadata", metadata.clone())?;
    }
    Ok(())
}
