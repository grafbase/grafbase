use dynomite::AttributeValue;
use std::collections::HashMap;
use std::str::FromStr;

pub fn value_to_attribute(value: serde_json::Value) -> AttributeValue {
    match value {
        serde_json::Value::Null => AttributeValue {
            null: Some(true),
            ..Default::default()
        },
        serde_json::Value::Bool(bool_val) => AttributeValue {
            bool: Some(bool_val),
            ..Default::default()
        },
        serde_json::Value::String(str_val) => AttributeValue {
            s: Some(str_val),
            ..Default::default()
        },
        serde_json::Value::Number(number_val) => AttributeValue {
            n: Some(number_val.to_string()),
            ..Default::default()
        },
        serde_json::Value::Array(array_val) => AttributeValue {
            l: Some(array_val.into_iter().map(value_to_attribute).collect()),
            ..Default::default()
        },
        serde_json::Value::Object(obj_val) => AttributeValue {
            m: Some(
                obj_val
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, (key, val)| {
                        acc.insert(key, value_to_attribute(val));
                        acc
                    }),
            ),
            ..Default::default()
        },
    }
}

pub fn attribute_to_value(value: AttributeValue) -> serde_json::Value {
    match value {
        AttributeValue {
            bool: Some(bool_value),
            ..
        } => serde_json::Value::Bool(bool_value),
        AttributeValue { l: Some(list), .. } => {
            serde_json::Value::Array(list.into_iter().map(attribute_to_value).collect())
        }
        AttributeValue {
            n: Some(number), ..
        } => serde_json::Value::Number(serde_json::Number::from_str(&number).expect("can't fail")),
        AttributeValue {
            s: Some(str_value), ..
        } => serde_json::Value::String(str_value),
        AttributeValue {
            ns: Some(number_set),
            ..
        } => serde_json::Value::Array(
            number_set
                .into_iter()
                .map(|str_value| {
                    serde_json::Value::Number(
                        serde_json::Number::from_str(&str_value).expect("can't fail"),
                    )
                })
                .collect(),
        ),
        AttributeValue {
            ss: Some(string_set),
            ..
        } => serde_json::Value::Array(
            string_set
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
        AttributeValue { null: Some(_), .. } => serde_json::Value::Null,
        AttributeValue {
            m: Some(object), ..
        } => serde_json::Value::Object(
            object
                .into_iter()
                .map(|(key, x)| (key, attribute_to_value(x)))
                .collect(),
        ),
        AttributeValue { b: Some(_), .. } => unimplemented!(),
        AttributeValue {
            bs: Some(_vec_bytes),
            ..
        } => unimplemented!(),
        _ => serde_json::Value::Null,
    }
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
