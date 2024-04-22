use std::{
    borrow::{Borrow, Cow},
    collections::BTreeMap,
    fmt::Write,
};

use reqwest::Url;
use url::form_urlencoded;

use super::{PathParameter, QueryParameter, QueryParameterEncodingStyle};
use crate::Error;

pub trait ParamApply {
    fn apply_path_parameter(self, param: &PathParameter, variable: serde_json::Value) -> Result<String, Error>;

    fn apply_query_parameters(self, params: &[QueryParameter], values: &[serde_json::Value]) -> Result<String, Error>;

    fn apply_body_parameters(
        self,
        encoding_styles: &BTreeMap<String, QueryParameterEncodingStyle>,
        variable: serde_json::Value,
    ) -> Result<String, Error>;
}

impl ParamApply for String {
    fn apply_path_parameter(self, param: &PathParameter, variable: serde_json::Value) -> Result<String, Error> {
        let name = &param.name;

        Ok(self.replace(&format!("{{{name}}}"), json_scalar_to_path_string(&variable)?.borrow()))
    }

    fn apply_query_parameters(self, params: &[QueryParameter], values: &[serde_json::Value]) -> Result<String, Error> {
        assert_eq!(params.len(), values.len());

        let mut url = Url::parse(&self).unwrap();

        if !params.is_empty() {
            let mut serializer = url.query_pairs_mut();

            for (param, value) in params.iter().zip(values.iter()) {
                urlencode_value(&param.name, value, param.encoding_style, &mut serializer)?;
            }

            serializer.finish();
            drop(serializer);
        }

        Ok(url.to_string())
    }

    fn apply_body_parameters(
        mut self,
        encoding_styles: &BTreeMap<String, QueryParameterEncodingStyle>,
        variable: serde_json::Value,
    ) -> Result<String, Error> {
        let mut serializer = url::form_urlencoded::Serializer::new(&mut self);

        let object = match variable {
            serde_json::Value::Object(object) => object,
            serde_json::Value::Null => return Ok(self),
            _ => return Err(Error::new("Internal error encoding body parameter")),
        };

        for (key, value) in object {
            let style = encoding_styles
                .get(&key)
                .unwrap_or(&QueryParameterEncodingStyle::FormExploded);

            urlencode_value(&key, &value, *style, &mut serializer)?;
        }

        serializer.finish();

        Ok(self)
    }
}

fn urlencode_value<T>(
    name: &str,
    value: &serde_json::Value,
    encoding_style: QueryParameterEncodingStyle,
    serializer: &mut form_urlencoded::Serializer<T>,
) -> Result<(), Error>
where
    T: form_urlencoded::Target,
{
    use serde_json::Value;

    // Scalars get serialized the same regardless.
    match value {
        Value::Null => return Ok(()),
        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serializer.append_pair(name, json_scalar_to_query_string(value)?.borrow());
            return Ok(());
        }
        _ => {}
    }

    // Query parameter encoding is a  pain.  I've handled three common styles here
    // but there's 5 other styles I'm just ignoring for now (can add them later, but
    // I'm kind of hoping nobody uses them)
    match encoding_style {
        QueryParameterEncodingStyle::Form => {
            let string_value = match value {
                Value::Array(values) => Cow::Owned(
                    values
                        .iter()
                        .map(json_scalar_to_query_string)
                        .collect::<Result<Vec<_>, _>>()?
                        .join(","),
                ),
                Value::Object(obj) => Cow::Owned(
                    obj.iter()
                        .map(|(key, value)| Ok(vec![Cow::Borrowed(key.as_str()), json_scalar_to_query_string(value)?]))
                        .collect::<Result<Vec<_>, Error>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                _ => {
                    unreachable!()
                }
            };
            serializer.append_pair(name, &string_value);
        }
        QueryParameterEncodingStyle::FormExploded => match value {
            Value::Array(values) => {
                for value in values {
                    serializer.append_pair(name, json_scalar_to_query_string(value)?.borrow());
                }
            }
            Value::Object(obj) => {
                for (key, value) in obj {
                    serializer.append_pair(key, json_scalar_to_query_string(value)?.borrow());
                }
            }
            _ => {
                unreachable!()
            }
        },
        QueryParameterEncodingStyle::DeepObject => {
            serializer.extend_pairs(DeepObjectIter::new(name, value));
        }
    }

    Ok(())
}

fn json_scalar_to_path_string(value: &serde_json::Value) -> Result<Cow<'_, str>, Error> {
    use serde_json::Value;
    match value {
        Value::Bool(b) => Ok(Cow::Owned(b.to_string())),
        Value::Number(number) => Ok(Cow::Owned(number.to_string())),
        Value::String(string) => Ok(Cow::Borrowed(string)),
        Value::Null => Err(Error::new("HTTP path parameters cannot be null")),
        Value::Array(_) => Err(Error::new("HTTP path parameters cannot be arrays")),
        Value::Object(_) => Err(Error::new("HTTP path parameters cannot be objects")),
    }
}

fn json_scalar_to_query_string(value: &serde_json::Value) -> Result<Cow<'_, str>, Error> {
    use serde_json::Value;
    match value {
        Value::Bool(b) => Ok(Cow::Owned(b.to_string())),
        Value::Number(number) => Ok(Cow::Owned(number.to_string())),
        Value::String(string) => Ok(Cow::Borrowed(string)),
        Value::Null => Err(Error::new("HTTP query parameters cannot have nested nulls")),
        Value::Array(_) => Err(Error::new("HTTP query parameters cannot have nested arrays")),
        Value::Object(_) => Err(Error::new("HTTP query parameters cannot have nested objects")),
    }
}

struct DeepObjectIter<'a> {
    stack: Vec<DeepObjectStackEntry<'a>>,
    parameter_name: &'a str,
}

struct DeepObjectStackEntry<'a> {
    keys: Vec<Cow<'a, str>>,
    value: &'a serde_json::Value,
}

impl<'a> DeepObjectIter<'a> {
    pub fn new(parameter_name: &'a str, value: &'a serde_json::Value) -> Self {
        DeepObjectIter {
            stack: vec![DeepObjectStackEntry { keys: vec![], value }],
            parameter_name,
        }
    }
}

impl<'a> Iterator for DeepObjectIter<'a> {
    type Item = (String, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let DeepObjectStackEntry { keys, value } = self.stack.pop()?;
            match value {
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_) => {
                    let mut key_str = self.parameter_name.to_string();
                    for key in keys {
                        write!(&mut key_str, "[{key}]").unwrap();
                    }
                    return Some((key_str, json_scalar_to_query_string(value).unwrap()));
                }
                serde_json::Value::Array(vals) => {
                    self.stack.extend(vals.iter().enumerate().map(|(i, nested_value)| {
                        let mut new_keys = keys.clone();
                        new_keys.push(Cow::Owned(i.to_string()));
                        DeepObjectStackEntry {
                            keys: new_keys,
                            value: nested_value,
                        }
                    }));
                }
                serde_json::Value::Object(obj) => {
                    self.stack.extend(obj.iter().map(|(key, nested_value)| {
                        let mut new_keys = keys.clone();
                        new_keys.push(Cow::Borrowed(key));
                        DeepObjectStackEntry {
                            keys: new_keys,
                            value: nested_value,
                        }
                    }));
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::redundant_clone)]
mod tests {
    use serde_json::json;

    use super::*;
    use registry_v2::resolvers::variable_resolve_definition::VariableResolveDefinition;

    #[test]
    fn test_path_parameter() {
        let param = PathParameter {
            name: "id".into(),
            variable_resolve_definition: VariableResolveDefinition::DebugString("whatever".into()),
        };
        let url = "/users/{id}".to_string();
        insta::assert_snapshot!(url.clone().apply_path_parameter(&param, json!(1)).unwrap(), @"/users/1");
        insta::assert_snapshot!(url.clone().apply_path_parameter(&param, json!("1")).unwrap(), @"/users/1");
    }

    #[test]
    fn test_form_query_parameters() {
        let id_param = QueryParameter {
            name: "id".into(),
            variable_resolve_definition: VariableResolveDefinition::DebugString("whatever".into()),
            encoding_style: QueryParameterEncodingStyle::Form,
        };
        let other_param = QueryParameter {
            name: "other".into(),
            ..id_param.clone()
        };
        let url = "https://example.com/users".to_string();

        // Test with no params
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[], &[]).unwrap(),
            @"https://example.com/users"
        );

        // Test an integer
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!(1)]).unwrap(),
            @"https://example.com/users?id=1"
        );

        // Test a string
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!("1")]).unwrap(),
            @"https://example.com/users?id=1"
        );

        // Test 2 parameters
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(
                &[id_param.clone(), other_param.clone()],
                &[json!("1"), json!(2)]
            ).unwrap(),
            @"https://example.com/users?id=1&other=2"
        );

        // test a list of strings
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!(["1", "2"])]).unwrap(),
            @"https://example.com/users?id=1%2C2"
        );

        // Test an object
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!({"one": "1", "two": "2"})]).unwrap(),
            @"https://example.com/users?id=one%2C+1%2C+two%2C+2"
        );
    }

    #[test]
    fn test_form_exploded_query_parameters() {
        let id_param = QueryParameter {
            name: "id".into(),
            variable_resolve_definition: VariableResolveDefinition::DebugString("whatever".into()),
            encoding_style: QueryParameterEncodingStyle::FormExploded,
        };
        let other_param = QueryParameter {
            name: "other".into(),
            ..id_param.clone()
        };
        let url = "https://example.com/users".to_string();

        // Test an integer
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!(1)]).unwrap(),
            @"https://example.com/users?id=1"
        );

        // Test a string
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!("1")]).unwrap(),
            @"https://example.com/users?id=1"
        );

        // Test 2 parameters
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(
                &[id_param.clone(), other_param.clone()],
                &[json!("1"), json!(2)]
            ).unwrap(),
            @"https://example.com/users?id=1&other=2"
        );

        // test a list of strings
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!(["1", "2"])]).unwrap(),
            @"https://example.com/users?id=1&id=2"
        );

        // Test an object
        insta::assert_snapshot!(
            url.clone().apply_query_parameters(&[id_param.clone()], &[json!({"one": "1", "two": "2"})]).unwrap(),
            @"https://example.com/users?one=1&two=2"
        );
    }

    #[test]
    fn test_deep_object_query_parameters() {
        let id_param = QueryParameter {
            name: "id".into(),
            variable_resolve_definition: VariableResolveDefinition::DebugString("whatever".into()),
            encoding_style: QueryParameterEncodingStyle::DeepObject,
        };
        let url = "https://example.com/users".to_string();

        let test = |value| {
            urlencoding::decode(
                &url.clone()
                    .apply_query_parameters(&[id_param.clone()], &[value])
                    .unwrap(),
            )
            .unwrap()
            .to_string()
        };

        // Test an object
        insta::assert_snapshot!(
            test(json!({"one": "1", "two": "2"})),
            @"https://example.com/users?id[two]=2&id[one]=1"
        );

        // Test a list
        insta::assert_snapshot!(
            test(json!([1, 2, 3])),
            @"https://example.com/users?id[2]=3&id[1]=2&id[0]=1"
        );

        // Test some nesting
        insta::assert_snapshot!(
            test(json!({"a_list": [{"one": 1, "two": 2}, {"other": "string"}]})),
            @"https://example.com/users?id[a_list][1][other]=string&id[a_list][0][two]=2&id[a_list][0][one]=1"
        );
    }
}
