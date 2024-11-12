use graphql_parser::{query::Number, schema::Value};

pub(super) fn normalize<'a>(arguments: &mut [(&'a str, Value<'a, &'a str>)]) {
    arguments.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (_, argument) in arguments {
        match argument {
            Value::String(value) => {
                *value = String::new();
            }
            Value::Float(value) => {
                *value = 0.0;
            }
            Value::Int(value) => {
                *value = Number::from(0);
            }
            Value::List(list) => {
                list.clear();
            }
            Value::Object(map) => {
                map.clear();
            }
            _ => (),
        }
    }
}
