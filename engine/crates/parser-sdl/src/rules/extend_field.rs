//! Implements parsing of the `@extendField` directive on types.
//!
//! This can be used to add additional directives to a generated field
//! of a generated type.

use engine::registry::MetaType;
use engine_parser::types::TypeKind;

use super::{
    directive::Directive,
    federation::{OverrideDirective, ProvidesDirective},
    visitor::{Visitor, VisitorContext},
};
use crate::directive_de::parse_directive;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtendFieldDirective {
    name: String,
    external: Option<bool>,
    inaccessible: Option<bool>,
    r#override: Option<OverrideDirective>,
    provides: Option<ProvidesDirective>,
    shareable: Option<bool>,
    #[serde(default)]
    tags: Vec<String>,
}

impl Directive for ExtendFieldDirective {
    fn definition() -> String {
        r#"
        directive @extendField(
            name: String!,
            external: Boolean,
            inaccessible: Boolean,
            override: ExtendFieldOverride,
            provides: ExtendFieldProvides,
            shareable: Boolean,
            tags: [String!],
        ) on OBJECT

        input ExtendFieldOverride {
            from: String!
        }

        input ExtendFieldProvides {
            fields: FieldSet!
        }
        "#
        .to_string()
    }
}

pub struct ExtendFieldVisitor;

impl<'a> Visitor<'a> for ExtendFieldVisitor {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        if ["Query", "Mutation"].contains(&type_definition.node.name.node.as_str()) {
            return;
        }

        let TypeKind::Object(_) = &type_definition.node.kind else {
            return;
        };

        let name = &type_definition.node.name.node;

        let metatype = ctx.registry.borrow().types.get(name.as_str()).cloned();
        let object = match metatype {
            Some(MetaType::Object(object)) => object,
            Some(_) => {
                ctx.report_error(
                    vec![type_definition.name.pos],
                    format!("You tried to extend the object {name} but {name} is not an object"),
                );
                return;
            }
            None => {
                // This error is reported elsewhere
                return;
            }
        };

        let directives = type_definition
            .node
            .directives
            .iter()
            .filter(|directive| directive.name.node == "extendField")
            .filter_map(
                |directive| match parse_directive::<ExtendFieldDirective>(directive, ctx.variables) {
                    Ok(parsed_directive) => {
                        if object.field_by_name(&parsed_directive.name).is_none() {
                            ctx.report_error(
                                vec![directive.pos],
                                format!(
                                    "You tried to extend the field {} which does not exist on {name}",
                                    &parsed_directive.name
                                ),
                            );
                            return None;
                        }
                        Some(parsed_directive)
                    }
                    Err(error) => {
                        ctx.append_errors(vec![error]);
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        if directives.is_empty() {
            return;
        }

        let mut registry = ctx.registry.borrow_mut();
        let Some(MetaType::Object(object)) = registry.types.get_mut(name.as_str()) else {
            unreachable!("Verified this above")
        };

        for directive in directives {
            let Some(field) = object.fields.get_mut(&directive.name) else {
                unreachable!("Verified this above");
            };

            if let Some(external) = directive.external {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().external = external;
            }
            if let Some(inaccessible) = directive.inaccessible {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().inaccessible = inaccessible;
            }
            if let Some(r#override) = directive.r#override {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().r#override = Some(r#override.from);
            }
            if let Some(provides) = directive.provides {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().provides = Some(provides.fields);
            }
            if let Some(shareable) = directive.shareable {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().shareable = shareable;
            }
            if !directive.tags.is_empty() {
                if field.federation.is_none() {
                    field.federation = Some(Default::default());
                }
                field.federation.as_mut().unwrap().tags.extend(directive.tags);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use engine::registry::RegistrySdlExt;
    use engine::{
        registry::{self, MetaField},
        Registry,
    };

    use crate::{ConnectorParsers, GraphqlDirective, OpenApiDirective, PostgresDirective};

    #[test]
    fn test_extending_field_on_connector_types() {
        let schema = r#"
            extend type Blah
              @extendField(
                name: "foo"
                external: true
                override: {from: "Blah"}
                provides: {fields: "id"}
              )
              @extendField(
                name: "bar"
                shareable: true
              )

            extend schema
              @openapi(name: "foo", namespace: false, schema: "http://example.com")
              @federation(version: "2.3")
        "#;

        let output = futures::executor::block_on(crate::parse(schema, &HashMap::new(), &FakeConnectorParser)).unwrap();

        insta::assert_snapshot!(output.registry.export_sdl(true), @r###"
        extend schema @link(
        	url: "https://specs.apollo.dev/federation/v2.3",
        	import: ["@key", "@tag", "@shareable", "@inaccessible", "@override", "@external", "@provides", "@requires", "@composeDirective", "@interfaceObject"]
        )
        type Blah {
        	foo: String! @external @override(from: "Blah") @provides(fields: "id")
        	bar: ID! @shareable
        	bloop: ID!
        }
        type Query {
        	blah: Blah
        }
        "###)
    }

    #[test]
    fn test_missing_field_error() {
        let schema = r#"
            extend type Blah
              @extendField(
                name: "nope"
                external: true
                override: {from: "Blah"}
                provides: {fields: "id"}
              )

            extend schema
              @openapi(name: "foo", namespace: false, schema: "http://example.com")
              @federation(version: "2.3")
        "#;

        let error =
            futures::executor::block_on(crate::parse(schema, &HashMap::new(), &FakeConnectorParser)).unwrap_err();

        insta::assert_snapshot!(error, @r###"[RuleError { locations: [Pos(3:15)], message: "You tried to extend the field nope which does not exist on Blah" }]"###)
    }

    struct FakeConnectorParser;

    #[async_trait::async_trait]
    impl ConnectorParsers for FakeConnectorParser {
        async fn fetch_and_parse_openapi(&self, _directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
            let mut registry = Registry::new();
            registry.types.insert(
                "Blah".into(),
                registry::ObjectType::new(
                    "Blah",
                    [
                        MetaField {
                            name: "foo".into(),
                            ty: "String!".into(),
                            ..MetaField::default()
                        },
                        MetaField {
                            name: "bar".into(),
                            ty: "ID!".into(),
                            ..MetaField::default()
                        },
                        MetaField {
                            name: "bloop".into(),
                            ty: "ID!".into(),
                            ..MetaField::default()
                        },
                    ],
                )
                .into(),
            );
            registry.query_root_mut().fields_mut().unwrap().insert(
                "customer".into(),
                MetaField {
                    name: "blah".into(),
                    ty: "Blah".into(),
                    ..MetaField::default()
                },
            );
            Ok(registry)
        }

        async fn fetch_and_parse_graphql(&self, _directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
            Err(Vec::new())
        }

        async fn fetch_and_parse_postgres(&self, _: &PostgresDirective) -> Result<Registry, Vec<String>> {
            Err(Vec::new())
        }
    }
}
