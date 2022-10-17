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

#[derive(Debug, Serialize)]
pub struct QueryResponse<'a> {
    // stats: todo!(),
    /// Root of the whole struct which is a Container
    root: QueryResponseNode<'a>,
}

#[derive(Debug, Serialize)]
pub struct ResponseNode<'a> {
    parent: Option<&'a QueryResponseNode<'a>>,
    /// If it's a node, it means there is an ID related to it.
    id: &'a str,
    /// Typename of the node
    typename: &'a str,
    /// Children which are (Relation_Name, Node)
    children: Vec<(&'a str, QueryResponseNode<'a>)>,
    /// Errors, not as `ServerError` yet as we do not have the position.
    errors: Vec<Error>,
}

#[derive(Debug, Serialize)]
pub struct ResponseContainer<'a> {
    parent: Option<&'a QueryResponseNode<'a>>,
    /// Children which are (Relation_Name, Node)
    children: Vec<(&'a str, QueryResponseNode<'a>)>,
    /// Errors, not as `ServerError` yet as we do not have the position.
    errors: Vec<Error>,
}

#[derive(Debug, Serialize)]
pub struct ResponseList<'a> {
    parent: Option<&'a QueryResponseNode<'a>>,
    children: Vec<QueryResponseNode<'a>>,
}

#[derive(Debug, Serialize)]
pub struct ResponsePrimitive<'a> {
    parent: Option<&'a QueryResponseNode<'a>>,
    value: ConstValue,
}

/// A Query Response Node
#[derive(Debug, Serialize)]
pub enum QueryResponseNode<'a> {
    Node(ResponseNode<'a>),
    Container(ResponseContainer<'a>),
    List(ResponseList<'a>),
    Primitive(ResponsePrimitive<'a>),
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

fn write_object<'a, K: Display + 'a, V: Display + 'a>(
    object: impl IntoIterator<Item = &'a (K, V)>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    f.write_char('{')?;
    let mut iter = object.into_iter();
    if let Some((name, value)) = iter.next() {
        write!(f, "{}: {}", name, value)?;
    }
    for (name, value) in iter {
        f.write_char(',')?;
        write!(f, "{}: {}", name, value)?;
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
    use super::{QueryResponse, QueryResponseNode, ResponsePrimitive};
    use insta::assert_display_snapshot;

    #[test]
    fn should_transform_into_simple_json() {
        let primitive = QueryResponseNode::Primitive(ResponsePrimitive {
            parent: None,
            value: dynaql_value::ConstValue::String("blbl".into()),
        });
        assert_display_snapshot!(primitive);
    }
}
