use super::{operations::RequestBody, Context};
use crate::{parsing::Ref, Error};
use openapiv3::v3_1::{self as openapiv3_1, Parameter};
use std::{collections::BTreeMap, rc::Rc};

/// Re-usable components that can be referenced in an OpenAPI v3 schema.
#[derive(Default)]
pub struct Components {
    pub(super) responses: BTreeMap<Ref, Vec<ResponseComponent>>,
    pub(super) request_bodies: BTreeMap<Ref, Rc<Vec<RequestBody>>>,
    pub(super) parameters: BTreeMap<Ref, Parameter>,
}

pub struct ResponseComponent {
    pub schema: Option<openapiv3_1::SchemaObject>,
    pub content_type: String,
}

impl Components {
    pub fn extend(&mut self, ctx: &mut Context, components: &openapiv3_1::Components) {
        for (name, response) in &components.responses {
            let Some(response) = response.as_item() else {
                ctx.errors.push(Error::TopLevelResponseWasReference(name.clone()));
                continue;
            };
            self.responses
                .insert(Ref::v3_response(name), convert_response(response));
        }

        for (name, request_body) in &components.request_bodies {
            let Some(request_body) = request_body.as_item() else {
                ctx.errors.push(Error::TopLevelRequestBodyWasReference(name.clone()));
                continue;
            };
            self.request_bodies.insert(
                Ref::v3_request_body(name),
                Rc::new(RequestBody::from_openapi_3_1(request_body)),
            );
        }

        for (name, parameter) in &components.parameters {
            let Some(parameter) = parameter.as_item() else {
                ctx.errors.push(Error::TopLevelParameterWasReference(name.clone()));
                continue;
            };
            self.parameters.insert(Ref::v3_parameter(name), parameter.clone());
        }
    }
}

fn convert_response(response: &openapiv3_1::Response) -> Vec<ResponseComponent> {
    response
        .content
        .iter()
        .map(|(content_type, content)| ResponseComponent {
            schema: content.schema.clone(),
            content_type: content_type.clone(),
        })
        .collect()
}
