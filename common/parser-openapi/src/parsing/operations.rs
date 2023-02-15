use std::{rc::Rc, str::FromStr};

use openapiv3::{ReferenceOr, StatusCode};

use crate::Error;

use super::components::{Components, Ref};

#[non_exhaustive]
#[derive(Clone)]
pub struct OperationDetails {
    pub path: String,
    pub verb: Verb,
    pub operation_id: Option<String>,
    pub request_bodies: Rc<Vec<RequestBody>>,
    pub responses: Vec<Response>,
}

impl std::fmt::Debug for OperationDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationDetails")
            .field("path", &self.path)
            .field("verb", &self.verb)
            .field("operation_id", &self.operation_id)
            .finish_non_exhaustive()
    }
}

impl OperationDetails {
    pub fn new(
        path: String,
        verb: Verb,
        operation: &openapiv3::Operation,
        components: &Components,
    ) -> Result<Self, Error> {
        let request_bodies = match &operation.request_body {
            None => Rc::new(vec![]),
            Some(ReferenceOr::Reference { reference }) => {
                let reference = Ref::absolute(reference);
                Rc::clone(
                    components
                        .request_bodies
                        .get(&reference)
                        .ok_or(Error::UnresolvedReference(reference))?,
                )
            }
            Some(ReferenceOr::Item(request_body)) => Rc::new(RequestBody::from_openapi(request_body)),
        };

        let mut responses = Vec::with_capacity(operation.responses.responses.len());
        for (status_code, response) in &operation.responses.responses {
            match response {
                ReferenceOr::Reference { reference } => {
                    let reference = Ref::absolute(reference);
                    let response_components = components
                        .responses
                        .get(&reference)
                        .ok_or(Error::UnresolvedReference(reference))?;

                    for response_component in response_components {
                        responses.push(Response {
                            status_code: status_code.clone(),
                            content_type: response_component.content_type.clone(),
                            schema: response_component.schema.clone(),
                        });
                    }
                }
                ReferenceOr::Item(response) => {
                    for (content_type, media_type) in &response.content {
                        responses.push(Response {
                            status_code: status_code.clone(),
                            content_type: content_type.clone(),
                            schema: media_type.schema.clone(),
                        });
                    }
                }
            }
        }

        Ok(OperationDetails {
            path,
            verb,
            operation_id: operation.operation_id.clone(),
            request_bodies,
            responses,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Get,
    Post,
    Put,
    Delete,
    Unknown,
}

impl FromStr for Verb {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_ref() {
            "get" => Verb::Get,
            "post" => Verb::Post,
            "put" => Verb::Put,
            "delete" => Verb::Delete,
            _ => Verb::Unknown,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestBody {
    pub content_type: String,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
}

impl RequestBody {
    pub fn from_openapi(request_body: &openapiv3::RequestBody) -> Vec<RequestBody> {
        request_body
            .content
            .iter()
            .map(|(content_type, content)| RequestBody {
                schema: content.schema.clone(),
                content_type: content_type.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub content_type: String,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
}
