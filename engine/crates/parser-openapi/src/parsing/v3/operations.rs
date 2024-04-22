use std::rc::Rc;

use indexmap::IndexMap;
use openapiv3::{Encoding, Parameter, ParameterSchemaOrContent, QueryStyle, ReferenceOr, StatusCode};
use registry_v2::resolvers::http::{ExpectedStatusCode, QueryParameterEncodingStyle, RequestBodyContentType};

use super::components::Components;
use crate::{graph::HttpMethod, parsing::Ref, Error};

#[non_exhaustive]
#[derive(Clone)]
pub struct OperationDetails {
    pub path: String,
    pub http_method: HttpMethod,
    pub operation_id: Option<String>,
    pub request_bodies: Rc<Vec<RequestBody>>,
    pub responses: Vec<Response>,
    pub(super) path_parameters: Vec<PathParameter>,
    pub(super) query_parameters: Vec<QueryParameter>,
}

impl std::fmt::Debug for OperationDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationDetails")
            .field("path", &self.path)
            .field("http_method", &self.http_method)
            .field("operation_id", &self.operation_id)
            .finish_non_exhaustive()
    }
}

impl OperationDetails {
    pub fn new(
        path: String,
        http_method: HttpMethod,
        operation: &openapiv3::Operation,
        components: &Components,
        inherited_parameters: &[ReferenceOr<Parameter>],
    ) -> Result<Self, Error> {
        let request_bodies = match &operation.request_body {
            None => Rc::new(vec![]),
            Some(ReferenceOr::Reference { reference }) => {
                let reference = Ref::absolute(reference);
                Rc::clone(
                    components
                        .request_bodies
                        .get(&reference)
                        .ok_or_else(|| reference.to_unresolved_error())?,
                )
            }
            Some(ReferenceOr::Item(request_body)) => Rc::new(RequestBody::from_openapi(request_body)),
        };

        let mut responses = Vec::with_capacity(operation.responses.responses.len());
        for (status_code, response) in &operation.responses.responses {
            match &response {
                ReferenceOr::Reference { reference } => {
                    let reference = Ref::absolute(reference);
                    let response_components = components
                        .responses
                        .get(&reference)
                        .ok_or_else(|| reference.to_unresolved_error())?;

                    for response_component in response_components {
                        responses.push(Response {
                            status_code: convert_status_code(status_code)?,
                            content_type: response_component.content_type.clone(),
                            schema: response_component.schema.clone(),
                        });
                    }
                }
                ReferenceOr::Item(response) => {
                    for (content_type, media_type) in &response.content {
                        responses.push(Response {
                            status_code: convert_status_code(status_code)?,
                            content_type: content_type.clone(),
                            schema: media_type.schema.clone(),
                        });
                    }
                }
            }
        }

        let mut path_parameters = Vec::new();
        let mut query_parameters = Vec::new();
        for parameter in &operation.parameters {
            let parameter = resolve_parameter(parameter, components)?;
            register_parameter(parameter, &mut path_parameters, &mut query_parameters, operation)?;
        }

        // We need to handle the parameters that were declared on the parent
        // of this operation.  These can be overriden by the parameters on the
        // operation so we need to skip any we've already seen.
        for parameter in inherited_parameters {
            let parameter = resolve_parameter(parameter, components)?;
            match parameter {
                Parameter::Query { parameter_data, .. } => {
                    if query_parameters
                        .iter()
                        .any(|existing_param| existing_param.name == parameter_data.name)
                    {
                        continue;
                    }
                }
                Parameter::Path { parameter_data, .. } => {
                    if path_parameters
                        .iter()
                        .any(|existing_param| existing_param.name == parameter_data.name)
                    {
                        continue;
                    }
                }
                _ => {}
            }
            register_parameter(parameter, &mut path_parameters, &mut query_parameters, operation)?;
        }

        Ok(OperationDetails {
            path,
            http_method,
            operation_id: operation.operation_id.clone(),
            request_bodies,
            responses,
            path_parameters,
            query_parameters,
        })
    }
}

fn resolve_parameter<'a>(
    parameter: &'a ReferenceOr<Parameter>,
    components: &'a Components,
) -> Result<&'a Parameter, Error> {
    let parameter = match parameter {
        ReferenceOr::Reference { reference } => {
            let reference = Ref::absolute(reference);
            components
                .parameters
                .get(&reference)
                .ok_or_else(|| reference.to_unresolved_error())?
        }
        ReferenceOr::Item(parameter) => parameter,
    };
    Ok(parameter)
}

fn register_parameter(
    parameter: &Parameter,
    path_parameters: &mut Vec<PathParameter>,
    query_parameters: &mut Vec<QueryParameter>,
    operation: &openapiv3::Operation,
) -> Result<(), Error> {
    match parameter {
        Parameter::Path { parameter_data, .. } => {
            path_parameters.push(PathParameter {
                name: parameter_data.name.clone(),
                schema: match &parameter_data.format {
                    ParameterSchemaOrContent::Schema(schema) => Some(schema.clone()),
                    ParameterSchemaOrContent::Content(_) => None,
                },
            });
        }
        Parameter::Query {
            parameter_data, style, ..
        } => query_parameters.push(QueryParameter {
            name: parameter_data.name.clone(),
            schema: match &parameter_data.format {
                ParameterSchemaOrContent::Schema(schema) => Some(schema.clone()),
                ParameterSchemaOrContent::Content(_) => None,
            },
            encoding_style: query_param_encoding_style(style, parameter_data.explode.unwrap_or(true)).ok_or_else(
                || {
                    Error::UnsupportedQueryParameterStyle(
                        parameter_data.name.clone(),
                        operation.operation_id.clone().unwrap_or_default(),
                        query_style_description(style).to_owned(),
                    )
                },
            )?,
            required: parameter_data.required,
        }),
        _ => {}
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct RequestBody {
    pub content_type: RequestBodyContentType,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
    pub required: bool,
}

impl RequestBody {
    pub fn from_openapi(request_body: &openapiv3::RequestBody) -> Vec<RequestBody> {
        request_body
            .content
            .iter()
            .filter_map(|(content_type, content)| {
                Some(RequestBody {
                    schema: content.schema.clone(),
                    content_type: request_body_content_type(content_type, &content.encoding)?,
                    required: request_body.required,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: ExpectedStatusCode,
    pub content_type: String,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
}

#[derive(Clone, Debug)]
pub(super) struct PathParameter {
    pub name: String,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
}

#[derive(Clone, Debug)]
pub(super) struct QueryParameter {
    pub name: String,
    pub schema: Option<ReferenceOr<openapiv3::Schema>>,
    pub encoding_style: QueryParameterEncodingStyle,
    pub required: bool,
}

fn query_param_encoding_style(query_style: &QueryStyle, explode: bool) -> Option<QueryParameterEncodingStyle> {
    match (query_style, explode) {
        (QueryStyle::Form, true) => Some(QueryParameterEncodingStyle::FormExploded),
        (QueryStyle::Form, false) => Some(QueryParameterEncodingStyle::Form),
        (QueryStyle::DeepObject, _) => Some(QueryParameterEncodingStyle::DeepObject),
        _ => None,
    }
}

fn query_style_description(query_style: &QueryStyle) -> &str {
    match query_style {
        QueryStyle::Form => "form",
        QueryStyle::SpaceDelimited => "spaceDelimited",
        QueryStyle::PipeDelimited => "pipeDelimited",
        QueryStyle::DeepObject => "deepObject",
    }
}

fn request_body_content_type(
    content_type: &str,
    encoding: &IndexMap<String, Encoding>,
) -> Option<RequestBodyContentType> {
    match content_type {
        "application/json" => Some(RequestBodyContentType::Json),
        "application/x-www-form-urlencoded" => Some(RequestBodyContentType::FormEncoded(
            encoding
                .iter()
                .filter_map(|(field, encoding)| {
                    Some((
                        field.clone(),
                        query_param_encoding_style(encoding.style.as_ref()?, encoding.explode)?,
                    ))
                })
                .collect(),
        )),
        _ => None,
    }
}

fn convert_status_code(code: &openapiv3::StatusCode) -> Result<ExpectedStatusCode, Error> {
    match code {
        StatusCode::Code(code) => Ok(ExpectedStatusCode::Exact(*code)),
        StatusCode::Range(1) => Ok(ExpectedStatusCode::Range(100..200)),
        StatusCode::Range(2) => Ok(ExpectedStatusCode::Range(200..300)),
        StatusCode::Range(3) => Ok(ExpectedStatusCode::Range(300..400)),
        StatusCode::Range(4) => Ok(ExpectedStatusCode::Range(400..500)),
        StatusCode::Range(5) => Ok(ExpectedStatusCode::Range(500..600)),
        _ => Err(Error::UnknownStatusCodeRange(code.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use registry_v2::resolvers::http::ExpectedStatusCode;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("200", ExpectedStatusCode::Exact(200))]
    #[case("201", ExpectedStatusCode::Exact(201))]
    #[case("2XX", ExpectedStatusCode::Range(200..300))]
    #[case("5XX", ExpectedStatusCode::Range(500..600))]
    fn test_status_codes(#[case] input: &str, #[case] expected: ExpectedStatusCode) {
        let status_code =
            serde_json::from_value::<openapiv3::StatusCode>(serde_json::Value::String(input.to_string())).unwrap();

        assert_eq!(convert_status_code(&status_code).unwrap(), expected);
    }
}
