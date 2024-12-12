use cynic_parser::Value;

pub(super) fn sanitize(value: Value<'_>, rendered: &mut String) {
    match value {
        Value::Variable(variable_value) => {
            rendered.push('$');
            rendered.push_str(variable_value.name());
        }
        Value::Int(_) | Value::Float(_) => rendered.push('0'),
        Value::String(_) => rendered.push_str("\"\""),
        Value::Boolean(boolean_value) => {
            if boolean_value.value() {
                rendered.push_str("true");
            } else {
                rendered.push_str("false");
            }
        }
        Value::Null(_) => rendered.push_str("null"),
        Value::Enum(enum_value) => rendered.push_str(enum_value.as_str()),
        Value::List(_) => {
            rendered.push_str("[]");
        }
        Value::Object(_) => {
            rendered.push_str("{}");
        }
    }
}
