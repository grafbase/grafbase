use engine_value::ConstValue;
use graph_entities::{ResponseContainer, ResponseList, ResponseNodeId, ResponsePrimitive};

use crate::{ContextExt, ContextField, ContextSelectionSetLegacy};

#[async_recursion::async_recursion]
pub async fn selection_set_into_node<'a>(value: ConstValue, ctx: &ContextSelectionSetLegacy<'a>) -> ResponseNodeId {
    match value {
        ConstValue::List(list) => {
            let mut container = ResponseList::default();
            for value in list {
                let id = selection_set_into_node(value, ctx).await;
                container.push(id);
            }
            ctx.response().await.insert_node(Box::new(container))
        }
        ConstValue::Object(value) => {
            let mut container = ResponseContainer::new_container();
            for (name, value) in value {
                let id = selection_set_into_node(value, ctx).await;
                container.insert(name.as_str(), id);
            }
            ctx.response().await.insert_node(container)
        }
        rest => {
            let node = ResponsePrimitive::new(rest.into());
            ctx.response().await.insert_node(node)
        }
    }
}

// TODO: Function is not proper, but own't really matter in the usage, still should be fixed later.
#[async_recursion::async_recursion]
pub async fn field_into_node<'a>(value: ConstValue, ctx: &ContextField<'a>) -> ResponseNodeId {
    match value {
        ConstValue::List(list) => {
            let mut container = ResponseList::default();
            for value in list {
                let id = field_into_node(value, ctx).await;
                container.push(id);
            }
            ctx.response().await.insert_node(Box::new(container))
        }
        ConstValue::Object(value) => {
            let mut container = ResponseContainer::new_container();
            for (name, value) in value {
                let id = field_into_node(value, ctx).await;
                container.insert(name.as_str(), id);
            }
            ctx.response().await.insert_node(container)
        }
        rest => {
            let node = ResponsePrimitive::new(rest.into());
            ctx.response().await.insert_node(node)
        }
    }
}
