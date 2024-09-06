use super::{ast, executable_ast};

pub(super) fn executable_value_to_type_system_value(value: executable_ast::Value<'_>) -> ast::Value<'_> {
    match value {
        executable_ast::Value::Variable(v) => ast::Value::Variable(v),
        executable_ast::Value::Int(v) => ast::Value::Int(v),
        executable_ast::Value::Float(v) => ast::Value::Float(v),
        executable_ast::Value::String(v) => ast::Value::String(v),
        executable_ast::Value::Boolean(v) => ast::Value::Boolean(v),
        executable_ast::Value::Enum(v) => ast::Value::Enum(v),
        executable_ast::Value::List(v) => {
            ast::Value::List(v.into_iter().map(executable_value_to_type_system_value).collect())
        }
        executable_ast::Value::Object(v) => ast::Value::Object(
            v.into_iter()
                .map(|(name, value)| (name, executable_value_to_type_system_value(value)))
                .collect(),
        ),
        executable_ast::Value::Null => ast::Value::Null,
    }
}

pub(super) trait IntoJson {
    fn into_json(self) -> Option<serde_json::Value>;
}

impl IntoJson for ast::Value<'_> {
    fn into_json(self) -> Option<serde_json::Value> {
        use serde_json::Value;

        Some(match self {
            ast::Value::Variable(_) => return None,
            ast::Value::Int(i) => Value::Number(i.into()),
            ast::Value::Float(i) => Value::Number(serde_json::Number::from_f64(f64::from(i)).unwrap()),
            ast::Value::String(s) | ast::Value::BlockString(s) => Value::String(s.to_owned()),
            ast::Value::Boolean(b) => Value::Bool(b),
            ast::Value::Null => Value::Null,
            ast::Value::Enum(_enm) => return None,
            ast::Value::List(list) => Value::Array(
                list.into_iter()
                    .map(|value| value.into_json())
                    .collect::<Option<Vec<_>>>()?,
            ),
            ast::Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(key, value)| Some((key.to_owned(), value.into_json()?)))
                    .collect::<Option<serde_json::Map<_, _>>>()?,
            ),
        })
    }
}
