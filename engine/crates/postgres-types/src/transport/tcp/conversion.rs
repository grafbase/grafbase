use serde_json::{Map, Value};
use tokio_postgres::{
    types::{Kind, Type},
    Row,
};

pub(super) fn json_to_string(json: Vec<Value>) -> Vec<Option<String>> {
    json.iter()
        .map(|value| match value {
            Value::Null => None,
            Value::Bool(_) | Value::Number(_) | Value::Object(_) => serde_json::to_string(value).map(Some).unwrap(),
            Value::String(s) => Some(s.to_string()),
            Value::Array(_) => json_array_to_string_array(value),
        })
        .collect()
}

pub(super) fn json_array_to_string_array(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bool(_) | Value::Number(_) | Value::String(_) => serde_json::to_string(value).map(Some).unwrap(),
        Value::Object(_) => json_array_to_string_array(&Value::String(serde_json::to_string(value).unwrap())),
        Value::Array(arr) => {
            let vals = arr
                .iter()
                .map(json_array_to_string_array)
                .map(|value| value.unwrap_or_else(|| "NULL".to_string()))
                .collect::<Vec<_>>()
                .join(",");

            Some(format!("{{{vals}}}"))
        }
    }
}

pub(super) fn row_to_json(row: &Row) -> Value {
    let mut object = Map::new();

    for (i, column) in row.columns().iter().enumerate() {
        let name = column.name().to_string();
        let pg_value = row.as_text(i).expect("column must have a text value at this point");
        let value = pg_text_to_json(pg_value, column.type_());

        object.insert(name, value);
    }

    Value::Object(object)
}

fn pg_text_to_json(pg_value: Option<&str>, pg_type: &Type) -> Value {
    let Some(val) = pg_value else { return Value::Null };

    if let Kind::Array(elem_type) = pg_type.kind() {
        return pg_array_parse(val, elem_type);
    }

    match *pg_type {
        Type::BOOL => Value::Bool(val == "t"),
        Type::INT2 | Type::INT4 => {
            let val = val.parse::<i32>().expect("the database says this is an integer");
            Value::Number(serde_json::Number::from(val))
        }
        Type::FLOAT4 | Type::FLOAT8 => {
            let val = val.parse::<f64>().expect("the database says this is a f32");
            let num = serde_json::Number::from_f64(val);

            if let Some(num) = num {
                Value::Number(num)
            } else {
                Value::String(val.to_string())
            }
        }
        Type::JSON | Type::JSONB => serde_json::from_str(val).expect("the database says this is json"),
        _ => Value::String(val.to_string()),
    }
}

fn pg_array_parse(pg_array: &str, elem_type: &Type) -> Value {
    _pg_array_parse(pg_array, elem_type, false).0
}

fn _pg_array_parse(pg_array: &str, elem_type: &Type, nested: bool) -> (Value, usize) {
    let mut pg_array_chr = pg_array.char_indices();
    let mut level = 0;
    let mut quote = false;
    let mut entries: Vec<Value> = Vec::new();
    let mut entry = String::new();

    // skip bounds decoration
    if let Some('[') = pg_array.chars().next() {
        for (_, c) in pg_array_chr.by_ref() {
            if c == '=' {
                break;
            }
        }
    }

    fn push_checked(entry: &mut String, entries: &mut Vec<Value>, elem_type: &Type) {
        if !entry.is_empty() {
            // While in usual postgres response we get nulls as None and everything else
            // as Some(&str), in arrays we get NULL as unquoted 'NULL' string (while
            // string with value 'NULL' will be represented by '"NULL"'). So catch NULLs
            // here while we have quotation info and convert them to None.
            if entry == "NULL" {
                entries.push(pg_text_to_json(None, elem_type));
            } else {
                entries.push(pg_text_to_json(Some(entry), elem_type));
            }
            entry.clear();
        }
    }

    while let Some((mut i, mut c)) = pg_array_chr.next() {
        let mut escaped = false;

        if c == '\\' {
            escaped = true;
            (i, c) = pg_array_chr.next().unwrap();
        }

        match c {
            '{' if !quote => {
                level += 1;
                if level > 1 {
                    let (res, off) = _pg_array_parse(&pg_array[i..], elem_type, true);
                    entries.push(res);
                    for _ in 0..off - 1 {
                        pg_array_chr.next();
                    }
                }
            }
            '}' if !quote => {
                level -= 1;
                if level == 0 {
                    push_checked(&mut entry, &mut entries, elem_type);

                    if nested {
                        return (Value::Array(entries), i);
                    }
                }
            }
            '"' if !escaped => {
                if quote {
                    // end of quoted string, so push it manually without any checks
                    // for emptiness or nulls
                    entries.push(pg_text_to_json(Some(&entry), elem_type));
                    entry.clear();
                }
                quote = !quote;
            }
            ',' if !quote => {
                push_checked(&mut entry, &mut entries, elem_type);
            }
            _ => {
                entry.push(c);
            }
        }
    }

    (Value::Array(entries), 0)
}
