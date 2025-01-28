use super::*;
use graphql_federated_graph::Wrapping;

pub(super) fn emit_federation_builtins(ctx: &mut Context<'_>) {
    let string_definition = ctx.definitions[&ctx.lookup_str("String").expect("String to be defined")];
    let boolean_definition = ctx.definitions[&ctx.lookup_str("Boolean").expect("Boolean to be defined")];
    let graph_str = ctx.insert_str("graph");
    let name_str = ctx.insert_str("name");
    let url_str = ctx.insert_str("url");
    let join_namespace = Some(ctx.insert_str("join"));

    // join__FieldSet
    let join_fieldset_scalar = {
        let name = ctx.insert_str("FieldSet");
        ctx.out.push_scalar_definition(federated::ScalarDefinitionRecord {
            namespace: join_namespace,
            name,
            directives: Vec::new(),
            description: None,
        })
    };

    // join__Graph
    let join_graph_scalar = {
        let name = ctx.insert_str("Graph");
        ctx.out.push_scalar_definition(federated::ScalarDefinitionRecord {
            namespace: join_namespace,
            name,
            directives: Vec::new(),
            description: None,
        })
    };

    // directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION
    {
        let directive_name = ctx.insert_str("unionMember");
        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: graph_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: federated::Definition::Scalar(join_graph_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        let member_str = ctx.insert_str("member");

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: member_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name: directive_name,
            arguments: (arguments_start, 2),
            locations: federated::DirectiveLocations::UNION,
            repeatable: true,
        });
    }

    // directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE
    {
        let directive_name = ctx.insert_str("implements");
        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: graph_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: federated::Definition::Scalar(join_graph_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        let interface_str = ctx.insert_str("interface");

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: interface_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name: directive_name,
            arguments: (arguments_start, 2),
            locations: federated::DirectiveLocations::OBJECT | federated::DirectiveLocations::INTERFACE,
            repeatable: true,
        });
    }

    // directive @join__graph(name: String!, url: String!) on ENUM_VALUE
    {
        let directive_name = ctx.insert_str("graph");
        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: name_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: url_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name: directive_name,
            arguments: (arguments_start, 2),
            locations: federated::DirectiveLocations::ENUM_VALUE,
            repeatable: false,
        });
    }

    // directive @join__field(
    //     graph: join__Graph
    //     requires: join__FieldSet
    //     provides: join__FieldSet
    // ) on FIELD_DEFINITION
    {
        let directive_name = ctx.insert_str("field");
        let requires_str = ctx.insert_str("requires");
        let provides_str = ctx.insert_str("provides");

        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: graph_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: federated::Definition::Scalar(join_graph_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: requires_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: federated::Definition::Scalar(join_fieldset_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: provides_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: federated::Definition::Scalar(join_fieldset_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name: directive_name,
            locations: federated::DirectiveLocations::FIELD_DEFINITION,
            arguments: (arguments_start, 3),
            repeatable: false,
        });
    }

    // directive @join__type(
    //     graph: join__Graph
    //     key: join__FieldSet
    //     resolvable: Boolean = true
    // ) on OBJECT | INTERFACE
    {
        let name = ctx.insert_str("type");
        let key_str = ctx.insert_str("key");
        let resolvable_str = ctx.insert_str("resolvable");

        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: graph_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: federated::Definition::Scalar(join_graph_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: key_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: federated::Definition::Scalar(join_fieldset_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: resolvable_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: boolean_definition,
            },
            directives: Vec::new(),
            description: None,
            default: Some(federated::Value::Boolean(true)),
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name,
            locations: federated::DirectiveLocations::OBJECT | federated::DirectiveLocations::INTERFACE,
            arguments: (arguments_start, 3),
            repeatable: false,
        });
    }

    // directive @join__owner(graph: join__Graph!) on OBJECT
    {
        let name = ctx.insert_str("owner");
        let arguments_start = ctx.out.push_input_value_definition(federated::InputValueDefinition {
            name: graph_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: federated::Definition::Scalar(join_graph_scalar),
            },
            directives: Vec::new(),
            description: None,
            default: None,
        });

        ctx.out.directive_definitions.push(federated::DirectiveDefinition {
            namespace: join_namespace,
            name,
            arguments: (arguments_start, 1),
            locations: federated::DirectiveLocations::OBJECT,
            repeatable: false,
        });
    }
}
