use super::*;
use wrapping::Wrapping;

pub(super) fn emit_federation_builtins(ctx: &mut Context<'_>, join_graph_enum_id: federated::EnumDefinitionId) {
    let string_definition = ctx.definitions[&ctx.lookup_str("String").expect("String to be defined")];
    let boolean_definition = ctx.definitions[&ctx.lookup_str("Boolean").expect("Boolean to be defined")];
    let graph_str = ctx.insert_str("graph");
    let name_str = ctx.insert_str("name");
    let url_str = ctx.insert_str("url");
    let is_interface_object_str = ctx.insert_str("isInterfaceObject");
    let extension_str = ctx.insert_str("extension");
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

    // directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION
    {
        let directive_name = ctx.insert_str("unionMember");
        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name: directive_name,
            locations: federated::DirectiveLocations::UNION,
            repeatable: true,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: graph_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: federated::Definition::Enum(join_graph_enum_id),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        let member_str = ctx.insert_str("member");

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: member_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: string_definition,
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );
    }

    // directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE
    {
        let directive_name = ctx.insert_str("implements");

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name: directive_name,
            locations: federated::DirectiveLocations::OBJECT | federated::DirectiveLocations::INTERFACE,
            repeatable: true,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: graph_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: federated::Definition::Enum(join_graph_enum_id),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        let interface_str = ctx.insert_str("interface");

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: interface_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: string_definition,
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );
    }

    // directive @join__graph(name: String!, url: String) on ENUM_VALUE
    {
        let directive_name = ctx.insert_str("graph");
        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name: directive_name,
            locations: federated::DirectiveLocations::ENUM_VALUE,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: name_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: string_definition,
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: url_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: string_definition,
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );
    }

    // directive @join__field(
    //     graph: join__Graph
    //     requires: join__FieldSet
    //     provides: join__FieldSet
    //     type: String,
    //     external: Boolean,
    //     override: String,
    //     overrideLabel: String
    // ) on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
    {
        let directive_name = ctx.insert_str("field");
        let requires_str = ctx.insert_str("requires");
        let provides_str = ctx.insert_str("provides");

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name: directive_name,
            locations: federated::DirectiveLocations::FIELD_DEFINITION
                | federated::DirectiveLocations::INPUT_FIELD_DEFINITION,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: graph_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: federated::Definition::Enum(join_graph_enum_id),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: requires_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: federated::Definition::Scalar(join_fieldset_scalar),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: provides_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: federated::Definition::Scalar(join_fieldset_scalar),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        let argument = federated::InputValueDefinition {
            name: ctx.insert_str("type"),
            r#type: federated::Type {
                wrapping: Wrapping::default(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        };
        ctx.out
            .push_directive_definition_argument(directive_definition_id, argument);

        let argument = federated::InputValueDefinition {
            name: ctx.insert_str("external"),
            r#type: federated::Type {
                wrapping: Wrapping::default(),
                definition: boolean_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        };
        ctx.out
            .push_directive_definition_argument(directive_definition_id, argument);

        let argument = federated::InputValueDefinition {
            name: ctx.insert_str("override"),
            r#type: federated::Type {
                wrapping: Wrapping::default(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        };
        ctx.out
            .push_directive_definition_argument(directive_definition_id, argument);

        let argument = federated::InputValueDefinition {
            name: ctx.insert_str("overrideLabel"),
            r#type: federated::Type {
                wrapping: Wrapping::default(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        };
        ctx.out
            .push_directive_definition_argument(directive_definition_id, argument);
    }

    // https://specs.apollo.dev/join/v0.3/#@type
    //
    // directive @join__type(
    //   graph: join__Graph!,
    //   key: join__FieldSet,
    //   extension: Boolean! = false,
    //   resolvable: Boolean! = true,
    //   isInterfaceObject: Boolean! = false
    // ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR
    {
        let name = ctx.insert_str("type");
        let key_str = ctx.insert_str("key");
        let resolvable_str = ctx.insert_str("resolvable");

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name,
            locations: federated::DirectiveLocations::OBJECT
                | federated::DirectiveLocations::INTERFACE
                | federated::DirectiveLocations::UNION
                | federated::DirectiveLocations::ENUM
                | federated::DirectiveLocations::INPUT_OBJECT
                | federated::DirectiveLocations::SCALAR,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: graph_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: federated::Definition::Enum(join_graph_enum_id),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: key_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: federated::Definition::Scalar(join_fieldset_scalar),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: extension_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: boolean_definition,
                },
                directives: Vec::new(),
                description: None,
                default: Some(federated::Value::Boolean(false)),
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: resolvable_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: boolean_definition,
                },
                directives: Vec::new(),
                description: None,
                default: Some(federated::Value::Boolean(true)),
            },
        );

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: is_interface_object_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default(),
                    definition: boolean_definition,
                },
                directives: Vec::new(),
                description: None,
                default: Some(federated::Value::Boolean(false)),
            },
        );
    }

    // directive @join__owner(graph: join__Graph!) on OBJECT
    {
        let name = ctx.insert_str("owner");

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: join_namespace,
            name,
            locations: federated::DirectiveLocations::OBJECT,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: graph_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: federated::Definition::Enum(join_graph_enum_id),
                },
                directives: Vec::new(),
                description: None,
                default: None,
            },
        );
    }
}
