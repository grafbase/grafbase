use crate::{
    builder::{
        extension::{ExtensionSdl, GrafbaseScalar},
        GraphContext, SchemaLocation,
    },
    extension::{ExtensionInputValueRecord, InjectionStage},
};
use cynic_parser::{
    common::{TypeWrappersIter, WrappingType},
    type_system::{Definition, EnumDefinition, InputObjectDefinition, InputValueDefinition, Type, TypeDefinition},
    ConstValue,
};
use federated_graph::Value;

use super::{value_path_to_string, ExtensionInputValueError, InputValueError};

pub(crate) struct ExtensionInputValueCoercer<'a, 'b> {
    pub ctx: &'a mut GraphContext<'b>,
    pub sdl: &'a ExtensionSdl,
    pub location: SchemaLocation,
    pub current_injection_stage: InjectionStage,
}

impl<'b> std::ops::Deref for ExtensionInputValueCoercer<'_, 'b> {
    type Target = GraphContext<'b>;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl std::ops::DerefMut for ExtensionInputValueCoercer<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

impl ExtensionInputValueCoercer<'_, '_> {
    pub fn coerce_argument(
        &mut self,
        def: InputValueDefinition<'_>,
        value: Option<&Value>,
    ) -> Result<Option<(ExtensionInputValueRecord, InjectionStage)>, ExtensionInputValueError> {
        self.value_path.clear();
        self.value_path.push(def.name().into());
        self.current_injection_stage = Default::default();
        let value = if let Some(value) = value {
            self.coerce_input_value(def.ty(), value)?
        } else if let Some(value) = def.default_value() {
            self.ingest_default_value(value)
        } else if def.ty().is_non_null() {
            return Err(InputValueError::MissingRequiredArgument(def.name().to_string()).into());
        } else {
            return Ok(None);
        };
        Ok(Some((value, self.current_injection_stage)))
    }

    fn coerce_input_value(
        &mut self,
        ty: Type<'_>,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        if ty.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type(ty.name(), value)?;
            for _ in ty.wrappers().filter(|w| matches!(w, WrappingType::List)) {
                value = ExtensionInputValueRecord::List(vec![value]);
            }
            return Ok(value);
        }

        self.coerce_type(ty.name(), ty.wrappers(), value)
    }

    fn coerce_type(
        &mut self,
        name: &str,
        mut wrappers: TypeWrappersIter,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let Some(wrapper) = wrappers.next() else {
            if value.is_null() {
                return Ok(ExtensionInputValueRecord::Null);
            }
            return self.coerce_named_type(name, value);
        };

        match wrapper {
            WrappingType::NonNull => {
                if value.is_null() {
                    Err(InputValueError::UnexpectedNull {
                        expected: self.type_name(name, wrappers, Some(WrappingType::NonNull)),
                        path: self.path(),
                    }
                    .into())
                } else {
                    self.coerce_type(name, wrappers, value)
                }
            }
            WrappingType::List => match value {
                Value::List(array) => array
                    .iter()
                    .enumerate()
                    .map(|(ix, value)| {
                        self.value_path.push(ix.into());
                        let value = self.coerce_type(name, wrappers.clone(), value)?;
                        self.value_path.pop();
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(ExtensionInputValueRecord::List),
                _ => Err(InputValueError::MissingList {
                    actual: value.into(),
                    expected: self.type_name(name, wrappers, Some(WrappingType::List)),
                    path: self.path(),
                }
                .into()),
            },
        }
    }

    fn coerce_named_type(
        &mut self,
        name: &str,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        if matches!(name, "ID" | "String" | "Int" | "BigInt" | "Float" | "Boolean") {
            return self.coerce_scalar(name, value);
        }
        if let Some((_, scalar)) = self.sdl.grafbase_scalars.iter().find(|(s, _)| s == name) {
            return match scalar {
                GrafbaseScalar::InputValueSet => match value {
                    Value::String(s) => {
                        let selection_set = &self.ctx.federated_graph[*s];
                        self.current_injection_stage = self.current_injection_stage.max(InjectionStage::Query);
                        self.coerce_input_value_set(selection_set)
                            .map(Into::into)
                            .map_err(Into::into)
                    }
                    _ => Err(InputValueError::IncorrectScalarType {
                        actual: value.into(),
                        expected: name.to_string(),
                        path: self.path(),
                    }
                    .into()),
                },
            };
        }
        let Some(def) = self.sdl.parsed.definitions().find_map(|def| match def {
            Definition::Type(def) if def.name() == name => Some(def),
            _ => None,
        }) else {
            return Err(ExtensionInputValueError::UnknownType { name: name.to_string() });
        };
        match def {
            TypeDefinition::Scalar(def) => self.coerce_scalar(def.name(), value),
            TypeDefinition::Enum(def) => self.coerce_enum(def, value),
            TypeDefinition::InputObject(def) => self.coerce_input_objet(def, value),
            _ => Err(ExtensionInputValueError::NotAnInputType { name: name.to_string() }),
        }
    }

    fn coerce_input_objet(
        &mut self,
        def: InputObjectDefinition<'_>,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let Value::Object(fields) = value else {
            return Err(InputValueError::MissingObject {
                name: def.name().to_string(),
                actual: value.into(),
                path: self.path(),
            }
            .into());
        };

        let mut map = Vec::new();
        let mut fields = fields.iter().collect::<Vec<_>>();

        for input_value_def in def.fields() {
            if let Some(index) = fields
                .iter()
                .position(|(id, _)| self.federated_graph[*id] == input_value_def.name())
            {
                let (name_id, value) = fields.swap_remove(index);
                let name_id = self.get_or_insert_str(*name_id);
                self.value_path.push(name_id.into());

                let value = self.coerce_input_value(input_value_def.ty(), value)?;
                map.push((name_id, value));

                self.value_path.pop();
            } else if let Some(default_value) = input_value_def.default_value() {
                map.push((
                    self.strings.get_or_new(input_value_def.name()),
                    self.ingest_default_value(default_value),
                ));
            } else if input_value_def.ty().is_non_null() {
                self.value_path.push(input_value_def.name().into());
                let error = InputValueError::UnexpectedNull {
                    expected: self.type_name(input_value_def.ty().name(), input_value_def.ty().wrappers(), None),
                    path: self.path(),
                };
                return Err(error.into());
            }
        }

        if let Some((name, _)) = fields.first() {
            let error = InputValueError::UnknownInputField {
                input_object: def.name().to_string(),
                name: self.ctx.federated_graph[*name].to_string(),
                path: self.path(),
            };

            return Err(error.into());
        }

        Ok(ExtensionInputValueRecord::Map(map))
    }

    fn coerce_enum(
        &mut self,
        def: EnumDefinition<'_>,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let value_id = match value {
            Value::EnumValue(id) => self.federated_graph[*id].value,
            Value::UnboundEnumValue(id) => *id,
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: def.name().to_string(),
                    actual: value.into(),
                    path: self.path(),
                }
                .into());
            }
        };
        let string_value = &self.federated_graph[value_id];
        if def.values().any(|value| value.value() == string_value) {
            return Ok(ExtensionInputValueRecord::EnumValue(self.get_or_insert_str(value_id)));
        }
        Err(InputValueError::UnknownEnumValue {
            r#enum: def.name().to_string(),
            value: string_value.to_string(),
            path: self.path(),
        }
        .into())
    }

    fn coerce_scalar(
        &mut self,
        name: &str,
        value: &Value,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        match name {
            "String" | "ID" => match value {
                Value::String(id) => Some(ExtensionInputValueRecord::String(self.get_or_insert_str(*id))),
                _ => None,
            },
            "Float" => match value {
                Value::Int(n) => Some(ExtensionInputValueRecord::Float(*n as f64)),
                Value::Float(f) => Some(ExtensionInputValueRecord::Float(*f)),
                _ => None,
            },
            "Int" => match value {
                Value::Int(n) => {
                    let n = i32::try_from(*n).map_err(|_| InputValueError::IncorrectScalarValue {
                        actual: n.to_string(),
                        expected: name.to_string(),
                        path: self.path(),
                    })?;
                    Some(ExtensionInputValueRecord::Int(n))
                }
                Value::Float(f) if can_coerce_to_int(*f) => Some(ExtensionInputValueRecord::Int(*f as i32)),
                _ => None,
            },
            "BigInt" => match value {
                Value::Int(n) => Some(ExtensionInputValueRecord::BigInt(*n)),
                Value::Float(f) if can_coerce_to_int(*f) => Some(ExtensionInputValueRecord::BigInt(*f as i64)),
                _ => None,
            },
            "Boolean" => match value {
                Value::Boolean(b) => Some(ExtensionInputValueRecord::Boolean(*b)),
                _ => None,
            },
            _ => return Ok(self.ingest_arbitrary_scalar(value)),
        }
        .ok_or_else(|| {
            InputValueError::IncorrectScalarType {
                actual: value.into(),
                expected: name.to_string(),
                path: self.path(),
            }
            .into()
        })
    }

    fn ingest_arbitrary_scalar(&mut self, value: &Value) -> ExtensionInputValueRecord {
        match value {
            Value::Null => ExtensionInputValueRecord::Null,
            Value::String(id) => ExtensionInputValueRecord::String(self.get_or_insert_str(*id)),
            Value::UnboundEnumValue(id) => ExtensionInputValueRecord::EnumValue(self.get_or_insert_str(*id)),
            Value::Int(n) => ExtensionInputValueRecord::BigInt(*n),
            Value::Float(f) => ExtensionInputValueRecord::Float(*f),
            Value::Boolean(b) => ExtensionInputValueRecord::Boolean(*b),
            Value::EnumValue(id) => {
                let value = self.federated_graph[*id].value;
                ExtensionInputValueRecord::EnumValue(self.get_or_insert_str(value))
            }
            Value::Object(fields) => ExtensionInputValueRecord::Map(
                fields
                    .iter()
                    .map(|(name, value)| {
                        let name = self.get_or_insert_str(*name);
                        (name, self.ingest_arbitrary_scalar(value))
                    })
                    .collect(),
            ),
            Value::List(list) => {
                ExtensionInputValueRecord::List(list.iter().map(|value| self.ingest_arbitrary_scalar(value)).collect())
            }
        }
    }

    fn ingest_default_value(&mut self, value: ConstValue) -> ExtensionInputValueRecord {
        match value {
            ConstValue::Null(_) => ExtensionInputValueRecord::Null,
            ConstValue::String(s) => ExtensionInputValueRecord::String(self.strings.get_or_new(s.value())),
            ConstValue::Int(n) => ExtensionInputValueRecord::BigInt(n.value()),
            ConstValue::Float(f) => ExtensionInputValueRecord::Float(f.value()),
            ConstValue::Boolean(b) => ExtensionInputValueRecord::Boolean(b.value()),
            ConstValue::Enum(s) => ExtensionInputValueRecord::EnumValue(self.strings.get_or_new(s.as_str())),
            ConstValue::List(list) => ExtensionInputValueRecord::List(
                list.into_iter().map(|value| self.ingest_default_value(value)).collect(),
            ),
            ConstValue::Object(fields) => ExtensionInputValueRecord::Map(
                fields
                    .into_iter()
                    .map(|field| {
                        let name = self.strings.get_or_new(field.name());
                        (name, self.ingest_default_value(field.value()))
                    })
                    .collect(),
            ),
        }
    }

    fn type_name(&self, name: &str, iter: TypeWrappersIter, outer: Option<WrappingType>) -> String {
        let mut out = String::new();
        let mut wrappers = Vec::new();
        wrappers.extend(outer);
        wrappers.extend(iter);
        for wrapping in &wrappers {
            if let WrappingType::List = wrapping {
                out.push('[');
            }
        }
        out.push_str(name);
        for wrapping in wrappers.iter().rev() {
            match wrapping {
                WrappingType::NonNull => out.push('!'),
                WrappingType::List => out.push(']'),
            }
        }
        out
    }

    fn path(&self) -> String {
        value_path_to_string(self, &self.value_path)
    }
}

fn can_coerce_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
