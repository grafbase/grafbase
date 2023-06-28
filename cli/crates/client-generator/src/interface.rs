use std::{borrow::Cow, fmt};

use crate::{
    comment::CommentBlock,
    r#type::{Property, StaticType},
};

pub struct Interface<'a> {
    identifier: StaticType<'a>,
    properties: Vec<Property<'a>>,
    description: Option<CommentBlock<'a>>,
}

impl<'a> Interface<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            identifier: StaticType::ident(name),
            properties: Vec::new(),
            description: None,
        }
    }

    pub fn push_property(&mut self, prop: Property<'a>) {
        self.properties.push(prop);
    }

    pub fn description(&mut self, comment: impl Into<CommentBlock<'a>>) {
        self.description = Some(comment.into());
    }
}

impl<'a> fmt::Display for Interface<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref comment) = self.description {
            writeln!(f, "{comment}")?;
        }

        writeln!(f, "interface {} {{", self.identifier)?;

        for prop in &self.properties {
            writeln!(f, "  {prop}")?;
        }

        f.write_str("}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{expect, expect_ts};
    use crate::{
        r#type::{ObjectTypeDef, Property, StaticType},
        statement::Export,
        Interface,
    };

    #[test]
    fn simple_interface() {
        let mut interface = Interface::new("BlogNode");
        interface.push_property(Property::new("id", StaticType::ident("string")));
        interface.push_property(Property::new("name", StaticType::ident("string")));
        interface.push_property(Property::new("owner", StaticType::ident("UserNode")));
        interface.push_property(Property::new("createdAt", StaticType::ident("Date")));

        let mut property = Property::new("updatedAt", StaticType::ident("Date"));
        property.optional();
        interface.push_property(property);

        let expected = expect![[r#"
            interface BlogNode {
              id: string
              name: string
              owner: UserNode
              createdAt: Date
              updatedAt?: Date
            }
        "#]];

        expect_ts(&interface, &expected);
    }

    #[test]
    fn interface_with_nested_object() {
        let mut object = ObjectTypeDef::new();
        object.push_property(Property::new("node", StaticType::ident("BlogSelect")));
        object.push_property(Property::new("age", StaticType::ident("number")));

        let mut interface = Interface::new("BlogCollectionSelect");
        interface.push_property(Property::new("fields", object));
        interface.push_property(Property::new("name", StaticType::ident("string")));

        let expected = expect![[r#"
            interface BlogCollectionSelect {
              fields: { node: BlogSelect; age: number }
              name: string
            }
        "#]];

        expect_ts(&interface, &expected);
    }

    #[test]
    fn export_interface() {
        let mut interface = Interface::new("User");

        let mut property = Property::new("id", StaticType::ident("string"));
        property.optional();

        interface.push_property(property);

        let expected = expect![[r#"
            export interface User {
              id?: string
            }
        "#]];

        expect_ts(&Export::new(interface), &expected);
    }
}
