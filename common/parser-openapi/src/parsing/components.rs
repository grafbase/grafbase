use std::{collections::BTreeMap, rc::Rc};

use openapiv3::ReferenceOr;

use crate::Error;

use super::{operations::RequestBody, Context};

/// Re-usable components that can be referenced in an OpenAPI schema.
#[derive(Default)]
pub struct Components {
    pub responses: BTreeMap<Ref, Vec<ResponseComponent>>,
    pub request_bodies: BTreeMap<Ref, Rc<Vec<RequestBody>>>,
}

pub struct ResponseComponent {
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
    pub content_type: String,
}

impl Components {
    pub fn extend(&mut self, ctx: &mut Context, components: &openapiv3::Components) {
        for (name, response) in &components.responses {
            let Some(response) = response.as_item() else {
                ctx.errors.push(Error::TopLevelResponseWasReference(name.clone()));
                continue;
            };
            self.responses.insert(Ref::response(name), convert_response(response));
        }

        for (name, request_body) in &components.request_bodies {
            let Some(request_body) = request_body.as_item() else {
                ctx.errors.push(Error::TopLevelRequestBodyWasReference(name.clone()));
                continue;
            };
            self.request_bodies
                .insert(Ref::response(name), Rc::new(RequestBody::from_openapi(request_body)));
        }
    }
}

fn convert_response(response: &openapiv3::Response) -> Vec<ResponseComponent> {
    response
        .content
        .iter()
        .map(|(content_type, content)| ResponseComponent {
            schema: content.schema.clone(),
            content_type: content_type.clone(),
        })
        .collect()
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ref(String);

impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Ref {
    pub fn absolute(absolute: &str) -> Ref {
        Ref(absolute.to_string())
    }

    pub fn schema(name: &str) -> Ref {
        Ref(format!("#/components/schemas/{name}"))
    }

    pub fn response(name: &str) -> Ref {
        Ref(format!("#/components/responses/{name}"))
    }

    pub fn request_body(name: &str) -> Ref {
        Ref(format!("#/components/request_bodies/{name}"))
    }
}
