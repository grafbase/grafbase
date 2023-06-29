use async_graphql_parser::types::ObjectType;

use crate::{
    interface::Interface,
    r#type::{Property, StaticType},
    statement::Export,
};

/// Transpiles a GraphQL type definition into TypeScript interface.
pub(crate) fn generate<'a>(name: &'a str, description: Option<&'a str>, object: &'a ObjectType) -> Export<'a> {
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

        interface.push_property(property);
    }

    let mut export = Export::new(interface);

    if let Some(description) = description {
        export.description(description);
    }

    export
}
