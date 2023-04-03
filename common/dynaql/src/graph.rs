use crate::registry::MetaType;
use crate::{relations_edges, Context, ContextSelectionSet};
use dynaql_value::ConstValue;
use graph_entities::{
    QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId, ResponseNodeRelation,
    ResponsePrimitive,
};

#[async_recursion::async_recursion]
pub async fn selection_set_into_node<'a>(
    value: ConstValue,
    ctx: &ContextSelectionSet<'a>,
    root: &MetaType,
) -> ResponseNodeId {
    let node = match value {
        ConstValue::List(list) => {
            let mut container = ResponseList::default();
            for value in list {
                let id = selection_set_into_node(value, ctx, root).await;
                container.push(id);
            }
            QueryResponseNode::List(container)
        }
        ConstValue::Object(value) => {
            let mut container = ResponseContainer::new_container();
            let relations = relations_edges(ctx, root);
            for (name, value) in value {
                let id = selection_set_into_node(value, ctx, root).await;
                let relation = name.to_string();
                let rel = if let Some(rel) = relations.get(name.as_str()) {
                    ResponseNodeRelation::relation(
                        relation,
                        rel.name.clone(),
                        rel.relation.0.clone(),
                        rel.relation.1.clone(),
                    )
                } else {
                    ResponseNodeRelation::NotARelation {
                        field: relation.into(),
                        response_key: None,
                    }
                };
                container.insert(rel, id);
            }
            QueryResponseNode::Container(container)
        }
        rest => {
            let node = ResponsePrimitive::new(rest);
            QueryResponseNode::Primitive(node)
        }
    };

    ctx.response_graph.write().await.new_node_unchecked(node)
}

// TODO: Function is not proper, but own't really matter in the usage, still should be fixed later.
#[async_recursion::async_recursion]
pub async fn field_into_node<'a>(value: ConstValue, ctx: &Context<'a>) -> ResponseNodeId {
    let node = match value {
        ConstValue::List(list) => {
            let mut container = ResponseList::default();
            for value in list {
                let id = field_into_node(value, ctx).await;
                container.push(id);
            }
            QueryResponseNode::List(container)
        }
        ConstValue::Object(value) => {
            let mut container = ResponseContainer::new_container();
            for (name, value) in value {
                let id = field_into_node(value, ctx).await;
                let relation = name.to_string();
                let rel = ResponseNodeRelation::NotARelation {
                    field: relation.into(),
                    response_key: None,
                };
                container.insert(rel, id);
            }
            QueryResponseNode::Container(container)
        }
        rest => {
            let node = ResponsePrimitive::new(rest);
            QueryResponseNode::Primitive(node)
        }
    };

    ctx.response_graph.write().await.new_node_unchecked(node)
}
