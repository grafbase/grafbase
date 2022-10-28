use crate::{Context, Response};
use async_trait::async_trait;
use dynaql_value::ConstValue;
use graph_entities::{
    QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId, ResponsePrimitive,
};
use worker::ResponseBody;

impl From<Response> for ResponseBody {
    fn from(value: Response) -> Self {
        let errors = if !value.errors.is_empty() {
            format!(
                ",\"errors\":{}",
                serde_json::to_string(&value.errors).expect("Unchecked")
            )
        } else {
            String::new()
        };

        ResponseBody::Body(format!("{{\"data\":{}{errors}}}", value.data.to_string()).into_bytes())
    }
}
