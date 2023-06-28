use async_graphql_parser::types::InputObjectType;

use crate::{
    r#type::{Property, StaticType},
    statement::Export,
    Interface,
};

pub(super) fn generate<'a>(name: &'a str, description: Option<&'a str>, object: &'a InputObjectType) -> Export<'a> {
    let mut interface = Interface::new(name);

    for field in &object.fields {
        let name = field.node.name.node.as_str();
        let mut r#type = StaticType::from_graphql(&field.node.ty.node.base);

        if field.node.ty.node.nullable {
            r#type.or(StaticType::null());
        }

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
