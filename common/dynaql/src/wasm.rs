use crate::Response;

use worker::ResponseBody;

impl From<Response> for ResponseBody {
    fn from(value: Response) -> Self {
        ResponseBody::Body(value.to_response_string().into_bytes())
    }
}
