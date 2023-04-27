#![allow(unused_crate_dependencies)]

use dynaql::registry::{MetaType, Registry};
use dynaql::resolver_utils::resolve_input_inner;
use dynaql::{Error, Value};

fn resolve(registry: &Registry, type_name: &str, value: serde_json::Value) -> Result<serde_json::Value, Error> {
    match (registry.types.get(type_name).unwrap(), value) {
        (MetaType::InputObject { input_fields, .. }, serde_json::Value::Object(mut input)) => {
            let resolved_fields = input_fields
                .iter()
                .map(|(name, input_type)| {
                    resolve_input_inner(
                        registry,
                        &mut vec![name.clone()],
                        &input_type.into(),
                        Value::from_json(input.remove(name).unwrap_or(serde_json::Value::Null)).unwrap(),
                    )
                    .map(|value| (name.clone(), value.into_json().unwrap()))
                })
                .collect::<Result<serde_json::Map<String, serde_json::Value>, _>>()?;
            Ok(serde_json::Value::Object(resolved_fields))
        }
        _ => unreachable!(),
    }
}

// FIXME There is certainly a better way to do this than testing a nested internal function. :/
#[test]
fn check_numerical_operation() {
    let registry: Registry = ::parser::parse_registry(
        r#"
            type Todo @model {
              title: String!
              done: Boolean! @default(value: false)
              starred: Boolean! @default(value: false)
              priority: Int! @default(value: 0)
              note: String
              dueDate: String
              subtasks: [Subtask]
            }

            type Subtask @model {
              title: String!
              done: Boolean! @default(value: false)
              starred: Boolean! @default(value: false)
              priority: Int! @default(value: 0)
              note: String
              dueDate: String
            }
        "#,
    )
    .unwrap();

    resolve(
        &registry,
        "TodoUpdateInput",
        serde_json::json!({"subtasks": {"link": "some id"}}),
    )
    .unwrap();
}
