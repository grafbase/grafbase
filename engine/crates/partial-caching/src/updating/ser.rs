use cynic_parser::{
    executable::{iter::Iter, FieldSelection, Selection},
    ExecutableDocument,
};
use graph_entities::{QueryResponse, QueryResponseNode, ResponseContainer};
use serde::ser::{Error, SerializeMap};

use crate::{query_subset::FilteredSelections, QuerySubset};

#[derive(Clone, Copy)]
struct SerializeContext<'a> {
    document: &'a ExecutableDocument,
    subset: &'a QuerySubset,
    response: &'a QueryResponse,
}

impl serde::Serialize for super::CacheUpdate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx = SerializeContext {
            document: self.document,
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

trait FieldExt {
    fn response_key(&self) -> &str;
}

impl FieldExt for FieldSelection<'_> {
    fn response_key(&self) -> &str {
        self.alias().unwrap_or(self.name())
    }
}

impl<'a> ObjectSerializer<'a> {
    pub fn field_iter(&self) -> FieldIter<'a> {
        FieldIter {
            iter_stack: vec![self.ctx.subset.selection_iter(self.selection_set)],
            subset: self.ctx.subset,
        }
    }
}

/// An iterator over the fields of a selection set.
///
/// This will recurse into any selection sets nested inside fragments.
struct FieldIter<'a> {
    iter_stack: Vec<FilteredSelections<'a>>,
    subset: &'a QuerySubset,
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldSelection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_iter) = self.iter_stack.last_mut() {
            let Some(selection) = current_iter.next() else {
                self.iter_stack.pop();
                continue;
            };

            match selection {
                Selection::Field(field) => return Some(field),
                Selection::InlineFragment(fragment) => {
                    self.iter_stack
                        .push(self.subset.selection_iter(fragment.selection_set()));
                }
                Selection::FragmentSpread(spread) => {
                    let Some(fragment) = spread.fragment() else { continue };

                    self.iter_stack
                        .push(self.subset.selection_iter(fragment.selection_set()));
                }
            }
        }

        None
    }
}
