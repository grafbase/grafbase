use std::collections::BTreeMap;

use openapi::v2::{Parameter, PathItem, Response, Spec};

use crate::parsing::Ref;

/// Re-usable components that can be referenced in an OpenAPI v2 schema.
#[derive(Default)]
pub struct Components {
    pub(super) responses: BTreeMap<Ref, Response>,
    pub(super) paths: BTreeMap<Ref, PathItem>,
    pub(super) parameters: BTreeMap<Ref, Parameter>,
}

impl Components {
    pub fn extend(&mut self, spec: &Spec) {
        if let Some(responses) = &spec.responses {
            for (name, response) in responses {
                self.responses.insert(Ref::v2_response(name), response.clone());
            }
        }

        for (name, path) in &spec.paths {
            self.paths.insert(Ref::v2_path(name), path.clone());
        }

        if let Some(parameters) = &spec.parameters {
            for (name, parameter) in parameters {
                self.parameters.insert(Ref::v2_parameter(name), parameter.clone());
            }
        }
    }
}
