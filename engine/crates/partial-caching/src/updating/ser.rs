use cynic_parser::executable::{iter::Iter, FieldSelection, Selection};
use graph_entities::{QueryResponse, QueryResponseNode, ResponseContainer};
use serde::ser::{Error, SerializeMap};

use crate::{parser_extensions::FieldExt, query_subset::FieldIter, QuerySubset};

#[derive(Clone, Copy)]
struct SerializeContext<'a> {
    subset: &'a QuerySubset,
    response: &'a QueryResponse,
}

impl serde::Serialize for super::CacheUpdate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx = SerializeContext {
            subset: self.subset,
            response: self.response,
        };

        let Some(root_id) = self.response.root else {
            return Err(S::Error::custom("internal error: no root to serialize"));
        };

        let Some(container) = self.response.get_container_node(root_id) else {
            return Err(S::Error::custom("internal error: root is not a container"));
        };

        ObjectSerializer {
            ctx,
            selection_set: self.document.read(self.subset.operation).selection_set(),
            container,
        }
        .serialize(serializer)
    }
}

/// Serializes the fields of ResponseContainer that are present in ctx.subset
struct ObjectSerializer<'a> {
    ctx: SerializeContext<'a>,
    selection_set: Iter<'a, Selection<'a>>,
    container: &'a ResponseContainer,
}

impl serde::Serialize for ObjectSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut object = serializer.serialize_map(None)?;

        for (name, value_id) in self.container.iter() {
            let Some(field) = self.field_iter().find(|field| field.response_key() == name.as_str()) else {
                continue;
            };

            let Some(node) = self.ctx.response.get_node(*value_id) else {
                continue;
            };

            object.serialize_entry(
                name.as_str(),
                &ValueSerializer {
                    ctx: self.ctx,
                    field,
                    node,
                },
            )?;
        }

        object.end()
    }
}

/// Serializes a value of a ResponseContainer
struct ValueSerializer<'a> {
    ctx: SerializeContext<'a>,
    field: FieldSelection<'a>,
    node: &'a QueryResponseNode,
}

impl serde::Serialize for ValueSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.node {
            QueryResponseNode::Container(container) => ObjectSerializer {
                ctx: self.ctx,
                selection_set: self.field.selection_set(),
                container: container.as_ref(),
            }
            .serialize(serializer),
            QueryResponseNode::List(list) => {
                serializer.collect_seq(list.iter().filter_map(|id| self.ctx.response.get_node(id)).map(|node| {
                    ValueSerializer {
                        ctx: self.ctx,
                        field: self.field,
                        node,
                    }
                }))
            }
            QueryResponseNode::Primitive(value) => value.serialize(serializer),
        }
    }
}

impl<'a> ObjectSerializer<'a> {
    pub fn field_iter(&self) -> FieldIter<'a> {
        FieldIter::new(self.ctx.subset.selection_iter(&self.selection_set), self.ctx.subset)
    }
}
