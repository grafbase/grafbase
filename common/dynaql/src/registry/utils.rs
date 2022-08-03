pub use dynamodb::{attribute_to_value, value_to_attribute};
use dynaql_parser::types::{BaseType, Type};

fn to_base_type_str(ty: &BaseType) -> String {
    match ty {
        BaseType::Named(name) => name.to_string(),
        BaseType::List(ty_list) => to_base_type_str(&ty_list.base),
    }
}

pub fn type_to_base_type(value: &str) -> Option<String> {
    Type::new(value).map(|x| to_base_type_str(&x.base))
}

/// Merge JSON together
pub fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
            let a = a.as_object_mut().expect("can't fail");
            for (k, v) in b {
                merge(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}
