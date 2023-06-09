use crate::common::{expect, expect_ts};
use grafbase_client_generator::{
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
    interface.push_property(Property::new("updatedAt", StaticType::ident("Date")).optional());

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
    interface.push_property(Property::new("id", StaticType::ident("string")));

    let expected = expect![[r#"
            export interface User {
              id: string
            }
        "#]];

    expect_ts(&Export::new(interface), &expected);
}
