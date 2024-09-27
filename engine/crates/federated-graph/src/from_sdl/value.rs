use cynic_parser::values::Value as ParserValue;

pub(super) trait IntoJson {
    fn into_json(self) -> Option<serde_json::Value>;
}

impl IntoJson for cynic_parser::values::Value<'_> {
    fn into_json(self) -> Option<serde_json::Value> {
        use serde_json::Value;

        Some(match self {
            ParserValue::Variable(_) => return None,
            ParserValue::Int(i) => Value::Number(i.as_i64().into()),
            ParserValue::Float(i) => Value::Number(serde_json::Number::from_f64(f64::from(i.value())).unwrap()),
            ParserValue::String(s) => Value::String(s.value().to_owned()),
            ParserValue::Boolean(b) => Value::Bool(b.value()),
            ParserValue::Null(_) => Value::Null,
            ParserValue::Enum(_enm) => return None,
            ParserValue::List(list) => Value::Array(
                list.items()
                    .map(|value| value.into_json())
                    .collect::<Option<Vec<_>>>()?,
            ),
            ParserValue::Object(obj) => Value::Object(
                obj.fields()
                    .map(|field| Some((field.name().to_owned(), field.value().into_json()?)))
                    .collect::<Option<serde_json::Map<_, _>>>()?,
            ),
        })
    }
}
