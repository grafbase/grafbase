use crate::{InstanceState, extension::api::wit};

pub use super::grafbase::sdk::hooks_types::*;

impl Host for InstanceState {}

impl From<wit::HttpRequestPartsParam> for HttpRequestPartsParam {
    fn from(param: wit::HttpRequestPartsParam) -> Self {
        HttpRequestPartsParam {
            method: param.method,
            url: param.url,
            headers: param.headers,
        }
    }
}
