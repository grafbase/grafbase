use async_graphql_parser::types::InputObjectType;

use crate::typescript_ast::{Export, Interface, Property, StaticType};

/// Transpiles a GraphQL input type into TypeScript interface.
pub(super) fn generate<'a>(name: &'a str, description: Option<&'a str>, object: &'a InputObjectType) -> Export<'a> {
    let mut property = Property::new("__typename", StaticType::string(name));
    property.optional();

    let mut interface = Interface::new(name);
    interface.push_property(property);

    for field in &object.fields {
        let name = field.node.name.node.as_str();
        let r#type = super::generate_base_type(&field.node.ty.node.base, field.node.ty.node.nullable);

        let mut property = Property::new(name, r#type);

        if let Some(ref description) = field.node.description {
            property.description(&description.node);
        }

        if field.node.ty.node.nullable {
            property.optional();
        }

        interface.push_property(property);
    }

    let mut export = Export::new(interface);

    if let Some(description) = description {
        export.description(description);
    }

    export
}
