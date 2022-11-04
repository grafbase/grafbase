use crate::{Context, Response};
use async_trait::async_trait;
use dynaql_value::ConstValue;
use graph_entities::{
    QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId, ResponsePrimitive,
};
use worker::ResponseBody;

impl From<Response> for ResponseBody {
    fn from(value: Response) -> Self {
        ResponseBody::Body(value.to_response_string().into_bytes())
    }
}
