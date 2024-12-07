use std::str::FromStr;

use cynic_parser::type_system::{self as ast};
use cynic_parser_deser::ConstDeserializer;

use super::{
    attach_input_value_set_to_field_arguments, attach_selection_set, parse_selection_set, AuthorizedDirective,
    CostDirective, Definition, DeprecatedDirective, Directive, DomainError, FieldId, GetArgumentsExt,
    InputValueDefinitionId, IntoJson, JoinFieldDirective, JoinImplementsDirective, JoinTypeDirective,
    JoinUnionMemberDirective, ListSize, ListSizeDirective, OverrideLabel, OverrideSource, State, StringId, Value,
};

pub(super) fn collect_definition_directives<'a>(
    definition_id: Definition,
    directives: impl Iterator<Item = ast::Directive<'a>>,
    state: &mut State<'a>,
) -> Result<Vec<Directive>, DomainError> {
    let mut out = Vec::new();
    for directive in directives {
        match directive.name() {
            "authorized" => {
                out.push(parse_authorized_type_directive(definition_id, directive, state)?);
            }
            "join__type" => {
                out.push(parse_join_type_directive(definition_id, directive, state)?);
            }
            "join__implements" => out.push(parse_join_implements(directive, state)?),
            "join__unionMember" => out.push(parse_join_union_member(directive, state)?),
            _ => out.extend(parse_common_directives(directive, state)),
        }
    }
    Ok(out)
}

pub(super) fn collect_field_directives<'a>(
    parent_id: Definition,
    field_id: FieldId,
    directives: impl Iterator<Item = ast::Directive<'a>>,
    state: &mut State<'a>,
) -> Result<Vec<Directive>, DomainError> {
    let mut out = Vec::new();
    for directive in directives {
        match directive.name() {
            "join__field" => {
                out.extend(parse_join_field_directive(parent_id, field_id, directive, state)?);
            }
            "authorized" => {
                out.extend(parse_authorized_field_directive(parent_id, field_id, directive, state));
            }
            "listSize" => {
                out.extend(parse_list_size_directive(field_id, directive, state)?);
            }
            _ => out.extend(parse_common_directives(directive, state)),
        }
    }
    Ok(out)
}

pub(super) fn collect_input_value_directives<'a>(
    directives: impl Iterator<Item = ast::Directive<'a>>,
    state: &mut State<'a>,
) -> Result<Vec<Directive>, DomainError> {
    Ok(directives
        .filter_map(|directive| parse_common_directives(directive, state))
        .collect())
}

pub(super) fn collect_enum_value_directives<'a>(
    directives: impl Iterator<Item = ast::Directive<'a>>,
    state: &mut State<'a>,
) -> Result<Vec<Directive>, DomainError> {
    Ok(directives
        .filter_map(|directive| parse_common_directives(directive, state))
        .collect())
}

fn parse_common_directives<'a>(directive: ast::Directive<'a>, state: &mut State<'a>) -> Option<Directive> {
    match directive.name() {
        "inaccessible" => Some(Directive::Inaccessible),
        "deprecated" => Some(parse_deprecated(directive, state)),
        "requiresScopes" => parse_requires_scopes(directive, state),
        "policy" => parse_policy(directive, state),
        "authenticated" => Some(Directive::Authenticated),
        "cost" => directive
            .deserialize::<CostDirective>()
            .map(|dir| Directive::Cost { weight: dir.weight })
            .ok(),
        _ => Some(parse_other(directive, state)),
    }
}

fn parse_join_implements(directive: ast::Directive<'_>, state: &mut State<'_>) -> Result<Directive, DomainError> {
    let Some(graph) = directive.get_argument("graph").and_then(|a| a.as_enum_value()) else {
        let error = DomainError("Missing graph argument in join__implements directive".to_owned());

        return Err(error);
    };

    let Some(interface) = directive.get_argument("interface").and_then(|a| a.as_str()) else {
        let error = DomainError("Missing interface argument in join__implements directive".to_owned());

        return Err(error);
    };

    let Some(subgraph_id) = state.graph_by_enum_str.get(graph).copied() else {
        let error = DomainError("Unknown graph in join__implements directive".to_owned());

        return Err(error);
    };

    let interface_id = match state.definition_names.get(interface) {
        Some(Definition::Interface(interface_id)) => *interface_id,
        _ => {
            let error = DomainError("Broken invariant: join__implements points to a non-interface type".to_owned());

            return Err(error);
        }
    };

    Ok(Directive::JoinImplements(JoinImplementsDirective {
        subgraph_id,
        interface_id,
    }))
}

fn parse_join_union_member(directive: ast::Directive<'_>, state: &mut State<'_>) -> Result<Directive, DomainError> {
    let Some(cynic_parser::ConstValue::Enum(graph)) = directive.get_argument("graph") else {
        let error = DomainError("Missing graph argument in join__unionMember directive".to_owned());
        return Err(error);
    };

    let Some(cynic_parser::ConstValue::String(member)) = directive.get_argument("member") else {
        let error = DomainError("Missing member argument in join__unionMember directive".to_owned());
        return Err(error);
    };

    let Some(subgraph_id) = state.graph_by_enum_str.get(graph.name()).copied() else {
        let error = DomainError("Unknown graph in join__unionMember directive".to_owned());
        return Err(error);
    };

    let object_id = match state.definition_names.get(member.value()) {
        Some(Definition::Object(object_id)) => *object_id,
        _ => {
            let error = DomainError("Broken invariant: join__unionMember points to a non-existing type".to_owned());
            return Err(error);
        }
    };

    Ok(Directive::JoinUnionMember(JoinUnionMemberDirective {
        subgraph_id,
        object_id,
    }))
}

fn parse_deprecated<'a>(directive: ast::Directive<'a>, state: &mut State<'a>) -> Directive {
    Directive::Deprecated {
        reason: directive
            .deserialize::<DeprecatedDirective<'_>>()
            .ok()
            .and_then(|directive| directive.reason)
            .map(|str| state.insert_string(str)),
    }
}

fn parse_requires_scopes<'a>(directive: ast::Directive<'a>, state: &mut State<'a>) -> Option<Directive> {
    let scopes: Option<Vec<Vec<String>>> = directive
        .get_argument("scopes")
        .and_then(|scopes| scopes.into_json())
        .and_then(|scopes| serde_json::from_value(scopes).ok());
    let transformed = scopes?
        .into_iter()
        .map(|scopes| scopes.into_iter().map(|scope| state.insert_string(&scope)).collect())
        .collect();
    Some(Directive::RequiresScopes(transformed))
}

fn parse_policy<'a>(directive: ast::Directive<'a>, state: &mut State<'a>) -> Option<Directive> {
    let policies: Option<Vec<Vec<String>>> = directive
        .get_argument("policies")
        .and_then(|policies| policies.into_json())
        .and_then(|policies| serde_json::from_value(policies).ok());
    let transformed = policies?
        .into_iter()
        .map(|policies| {
            policies
                .into_iter()
                .map(|policy| state.insert_string(&policy))
                .collect()
        })
        .collect();
    Some(Directive::Policy(transformed))
}

fn parse_other<'a>(directive: ast::Directive<'a>, state: &mut State<'a>) -> Directive {
    let name = state.insert_string(directive.name());
    let arguments = directive
        .arguments()
        .map(|arg| -> (StringId, Value) { (state.insert_string(arg.name()), state.insert_value(arg.value(), None)) })
        .collect();
    Directive::Other { name, arguments }
}

fn parse_authorized_type_directive<'a>(
    definition_id: Definition,
    directive: ast::Directive<'a>,
    state: &mut State<'a>,
) -> Result<Directive, DomainError> {
    let fields = directive
        .get_argument("fields")
        .and_then(|arg| arg.as_str())
        .map(|fields| parse_selection_set(fields).and_then(|doc| attach_selection_set(&doc, definition_id, state)))
        .transpose()?
        .filter(|fields| !fields.is_empty());

    let metadata = directive
        .get_argument("metadata")
        .map(|metadata| state.insert_value(metadata, None));

    Ok(Directive::Authorized(AuthorizedDirective {
        fields,
        node: None,
        arguments: None,
        metadata,
    }))
}

fn parse_join_type_directive<'a>(
    definition_id: Definition,
    directive: ast::Directive<'a>,
    state: &mut State<'a>,
) -> Result<Directive, DomainError> {
    let subgraph_id = directive
        .get_argument("graph")
        .and_then(|arg| arg.as_enum_value())
        .map(|name| state.graph_by_enum_str[name])
        .expect("Missing graph argument in @join__type");
    let key = directive
        .get_argument("key")
        .and_then(|arg| arg.as_str())
        .map(|key| parse_selection_set(key).and_then(|doc| attach_selection_set(&doc, definition_id, state)))
        .transpose()?
        .filter(|key| !key.is_empty());
    let resolvable = directive
        .get_argument("resolvable")
        .and_then(|arg| arg.as_bool())
        .unwrap_or(true);

    let is_interface_object = directive
        .get_argument("isInterfaceObject")
        .map(|arg| matches!(arg.as_bool(), Some(true)))
        .unwrap_or(false);

    Ok(Directive::JoinType(JoinTypeDirective {
        subgraph_id,
        key,
        resolvable,
        is_interface_object,
    }))
}

///```ignore,graphql
/// directive @join__field(
///     graph: join__Graph,
///     requires: join__FieldSet,
///     provides: join__FieldSet,
///     type: String,
///     external: Boolean,
///     override: String,
///     usedOverridden: Boolean
/// ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
/// ```
fn parse_join_field_directive<'a>(
    parent_id: Definition,
    field_id: FieldId,
    directive: ast::Directive<'a>,
    state: &mut State<'a>,
) -> Result<Option<Directive>, DomainError> {
    let field_type = state.graph[field_id].r#type.clone();
    let is_external = directive
        .get_argument("external")
        .map(|arg| arg.as_bool().unwrap_or_default())
        .unwrap_or_default();

    if is_external {
        return Ok(None);
    }

    let Some(subgraph_id) = directive
        .get_argument("graph")
        .and_then(|arg| arg.as_enum_value())
        .and_then(|name| state.graph_by_enum_str.get(name).copied())
    else {
        return Ok(None);
    };

    let requires = directive
        .get_argument("requires")
        .and_then(|value| value.as_str())
        .map(|requires| parse_selection_set(requires).and_then(|doc| attach_selection_set(&doc, parent_id, state)))
        .transpose()?
        .filter(|requires| !requires.is_empty());

    let provides = directive
        .get_argument("provides")
        .and_then(|value| value.as_str())
        .map(|provides| {
            parse_selection_set(provides).and_then(|doc| attach_selection_set(&doc, field_type.definition, state))
        })
        .transpose()?
        .filter(|provides| !provides.is_empty());

    let r#type = directive
        .get_argument("type")
        .and_then(|arg| arg.as_str())
        .map(|ty| state.field_type_from_str(ty))
        .transpose()?;

    let r#override = directive
        .get_argument("override")
        .and_then(|arg| arg.as_str())
        .map(|name| {
            state
                .graph_by_name
                .get(name)
                .copied()
                .map(OverrideSource::Subgraph)
                .unwrap_or_else(|| OverrideSource::Missing(state.insert_string(name)))
        });

    let override_label = directive
        .get_argument("overrideLabel")
        .filter(|_| r#override.is_some())
        .and_then(|arg| arg.as_str())
        .and_then(|s| OverrideLabel::from_str(s).ok());

    Ok(Some(Directive::JoinField(JoinFieldDirective {
        subgraph_id,
        requires,
        provides,
        r#type,
        r#override,
        override_label,
    })))
}

fn parse_authorized_field_directive<'a>(
    parent_id: Definition,
    field_id: FieldId,
    directive: ast::Directive<'a>,
    state: &mut State<'a>,
) -> Result<Directive, DomainError> {
    let field_type = state.graph[field_id].r#type.clone();

    Ok(Directive::Authorized(AuthorizedDirective {
        arguments: directive
            .get_argument("arguments")
            .and_then(|value| value.as_str())
            .map(|arguments| {
                parse_selection_set(arguments)
                    .and_then(|fields| attach_input_value_set_to_field_arguments(fields, parent_id, field_id, state))
            })
            .transpose()?
            .filter(|arguments| !arguments.is_empty()),
        fields: directive
            .get_argument("fields")
            .and_then(|value| value.as_str())
            .map(|fields| {
                parse_selection_set(fields).and_then(|fields| attach_selection_set(&fields, parent_id, state))
            })
            .transpose()?
            .filter(|fields| !fields.is_empty()),
        node: directive
            .get_argument("node")
            .and_then(|value| value.as_str())
            .map(|fields| {
                parse_selection_set(fields)
                    .and_then(|fields| attach_selection_set(&fields, field_type.definition, state))
            })
            .transpose()?
            .filter(|node| !node.is_empty()),
        metadata: directive
            .get_argument("metadata")
            .map(|metadata| state.insert_value(metadata, None)),
    }))
}

fn parse_list_size_directive<'a>(
    field_id: FieldId,
    directive: ast::Directive<'a>,
    state: &mut State<'a>,
) -> Result<Option<Directive>, DomainError> {
    let field = &state.graph[field_id];

    let Ok(ListSizeDirective {
        assumed_size,
        slicing_arguments,
        sized_fields,
        require_one_slicing_argument,
    }) = directive.deserialize::<ListSizeDirective>()
    else {
        return Ok(None);
    };

    let argument_base_index = usize::from(field.arguments.0);
    let arguments = &state.graph.input_value_definitions[argument_base_index..argument_base_index + field.arguments.1];
    let slicing_arguments = slicing_arguments
        .iter()
        .filter_map(|argument| {
            let (index, _) = arguments
                .iter()
                .enumerate()
                .find(|(_, value)| state[value.name] == *argument)?;

            Some(InputValueDefinitionId::from(index + argument_base_index))
        })
        .collect();

    let child_type_id = field.r#type.definition;
    let sized_fields = sized_fields
        .iter()
        .filter_map(|field| state.selection_map.get(&(child_type_id, field)).copied())
        .collect();

    Ok(Some(Directive::ListSize(ListSize {
        assumed_size,
        slicing_arguments,
        sized_fields,
        require_one_slicing_argument,
    })))
}
