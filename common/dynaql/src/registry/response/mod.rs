//! `QueryResponse` is an AST which aims to represent a result of a `DynaQL` response.
//!
//! This structure is the **resolved** version of a query, the point is not to have a logic of any
//! kind considering graph, the point is to be able to have a representation of a answer where we
//! are able to remove and add elements **BY NODE ID** where it would be translated into JSON.
//!
//! # Why do we need that?
//!
//! The purpose of this structure is to be shared across multiple layers / application. It allow us
//! to have an abstraction between the result of a query and the final representation for the user.
//!
//! If we create the final representation directly, then we can't add any metadata in the response
//! for other services or application to use.
//!
//! For instance, live-queries are working by knowing what data is requested by an user, and
//! process every events hapenning on the database to identify if the followed data changed. If the
//! followed data changed, so it means the server will have to compute the diff between those. To
//! be able to faithfully compute the diff, it's much more simplier to not use the path of this
//! data but to use the unique ID of the data you are modifying. Hence, this representation.

use crate::Error;
use core::fmt::{self, Display, Formatter};
use dynaql_value::ConstValue;
use serde::Serialize;
use std::fmt::Write;

#[derive(Debug, Serialize, Clone)]
pub struct QueryResponse<'a> {
    // stats: todo!(),
    /// Root of the whole struct which is a Container
    root: QueryResponseNode<'a>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResponseNode<'a> {
    /// If it's a node, it means there is an ID related to it.
    id: &'a str,
    /// Typename of the node
    typename: &'a str,
    /// Children which are (Relation_Name, Node)
    children: Vec<(&'a str, QueryResponseNode<'a>)>,
    /// Errors, not as `ServerError` yet as we do not have the position.
    errors: Vec<Error>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResponseContainer<'a> {
    /// Children which are (Relation_Name, Node)
    children: Vec<(&'a str, QueryResponseNode<'a>)>,
    /// Errors, not as `ServerError` yet as we do not have the position.
    errors: Vec<Error>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResponseList<'a> {
    children: Vec<QueryResponseNode<'a>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResponsePrimitive {
    value: ConstValue,
}

/// A Query Response Node
#[derive(Debug, Serialize, Clone)]
pub enum QueryResponseNode<'a> {
    Node(ResponseNode<'a>),
    Container(ResponseContainer<'a>),
    List(ResponseList<'a>),
    Primitive(ResponsePrimitive),
}

impl<'a> Display for QueryResponseNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            QueryResponseNode::Node(ResponseNode { children, .. }) => write_object(children, f),
            QueryResponseNode::Container(ResponseContainer { children, .. }) => {
                write_object(children, f)
            }
            QueryResponseNode::List(ResponseList { children, .. }) => write_list(children, f),
            QueryResponseNode::Primitive(ResponsePrimitive { value, .. }) => write!(f, "{}", value),
        }
    }
}

impl<'a> QueryResponseNode<'a> {
    /// Add a child to this node
    pub fn add_child(&mut self, field: &'a str, child: QueryResponseNode<'a>) {
        match self {
            QueryResponseNode::Node(node) => {
                node.children.push((field, child));
            }
            QueryResponseNode::Container(container) => {
                container.children.push((field, child));
            }
            QueryResponseNode::List(_) => {
                panic!("Can't add a children to a list")
            }
            QueryResponseNode::Primitive(_) => {
                panic!("Can't add a children to a primitive")
            }
        }
    }
}

/// Key must be a `&str` for now, so it means we'll need to add the escape sequence ourselves
fn write_object<'a, K: Display + 'a, V: Display + 'a>(
    object: impl IntoIterator<Item = &'a (K, V)>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    f.write_char('{')?;
    let mut iter = object.into_iter();
    if let Some((name, value)) = iter.next() {
        write!(f, "\"{}\":{}", name, value)?;
    }
    for (name, value) in iter {
        f.write_char(',')?;
        write!(f, "\"{}\":{}", name, value)?;
    }
    f.write_char('}')
}

fn write_list<'a, T: Display + 'a>(
    list: impl IntoIterator<Item = &'a T>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    f.write_char('[')?;
    let mut iter = list.into_iter();
    if let Some(item) = iter.next() {
        item.fmt(f)?;
    }
    for item in iter {
        f.write_char(',')?;
        item.fmt(f)?;
    }
    f.write_char(']')
}

#[cfg(test)]
mod tests {
    use super::{QueryResponseNode, ResponseContainer, ResponsePrimitive};

    #[test]
    fn should_transform_into_simple_json() {
        let primitive = QueryResponseNode::Primitive(ResponsePrimitive {
            value: dynaql_value::ConstValue::String("blbl".into()),
        });

        assert_eq!(primitive.to_string(), serde_json::json!("blbl").to_string());
    }

    #[test]
    fn should_transform_example_json() {
        let mut container = QueryResponseNode::Container(ResponseContainer {
            errors: vec![],
            children: vec![],
        });

        let mut container_title = QueryResponseNode::Container(ResponseContainer {
            errors: vec![],
            children: vec![],
        });

        let primitive = QueryResponseNode::Primitive(ResponsePrimitive {
            value: dynaql_value::ConstValue::String("example".to_string()),
        });
        container_title.add_child("title", primitive);
        container.add_child("glossary", container_title);

        let output_json = serde_json::json!({
            "glossary": {
                "title": "example",
            }
        });

        assert_eq!(container.to_string(), output_json.to_string());
    }
}
