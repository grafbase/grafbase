use crate::{
    ExtensionDirectiveArgumentId, ExtensionDirectiveArgumentRecord, FieldSetRecord, TemplateEscaping, TemplateRecord,
    builder::{
        GraphBuilder,
        extension::{ExtensionSdl, GrafbaseScalar},
        sdl,
    },
    extension::{ExtensionInputValueRecord, InjectionStage},
};
use id_newtypes::IdRange;
use itertools::Itertools as _;

use super::{ExtensionInputValueError, InputValueError, can_coerce_to_int, value_path_to_string};

pub(crate) struct ExtensionDirectiveArgumentsCoercer<'a, 'b> {
    pub(super) ctx: &'a mut GraphBuilder<'b>,
    pub(super) sdl: &'a ExtensionSdl,
    pub(super) current_definition: sdl::SdlDefinition<'b>,
    pub(super) current_injection_stage: InjectionStage,
    pub(super) is_default_value: bool,
    pub(super) requirements: FieldSetRecord,
}

impl<'b> std::ops::Deref for ExtensionDirectiveArgumentsCoercer<'_, 'b> {
    type Target = GraphBuilder<'b>;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl std::ops::DerefMut for ExtensionDirectiveArgumentsCoercer<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

impl<'a> GraphBuilder<'a> {
    pub fn coerce_extension_directive_arguments(
        &mut self,
        current_definition: sdl::SdlDefinition<'a>,
        sdl: &ExtensionSdl,
        directive: sdl::DirectiveDefinition<'_>,
        arguments: Option<sdl::ConstValue<'a>>,
    ) -> Result<(IdRange<ExtensionDirectiveArgumentId>, FieldSetRecord), ExtensionInputValueError> {
        let start = self.graph.extension_directive_arguments.len();
        let mut coercer = ExtensionDirectiveArgumentsCoercer {
            ctx: self,
            sdl,
            current_definition,
            current_injection_stage: Default::default(),
            requirements: Default::default(),
            is_default_value: false,
        };
        if let Some(arguments) = arguments.and_then(|arg| arg.as_fields()) {
            let mut arguments = arguments.collect::<Vec<_>>();
            coercer.graph.extension_directive_arguments.reserve(arguments.len());

            for def in directive.arguments() {
                let name_id = coercer.ingest_str(def.name());
                let sdl_value = arguments
                    .iter()
                    .position(|arg| arg.name() == def.name())
                    .map(|ix| arguments.swap_remove(ix).value());

                let maybe_coerced_argument = coercer.coerce_argument(def, sdl_value)?;

                if let Some((value, injection_stage)) = maybe_coerced_argument {
                    coercer
                        .ctx
                        .graph
                        .extension_directive_arguments
                        .push(ExtensionDirectiveArgumentRecord {
                            name_id,
                            value,
                            injection_stage,
                        });
                }
            }

            if let Some(arg) = arguments.first() {
                return Err(InputValueError::UnknownArgument(arg.name().into()).into());
            }
        }

        let requirements = coercer.requirements;
        let argument_ids = (start..self.graph.extension_directive_arguments.len()).into();
        Ok((argument_ids, requirements))
    }
}

impl ExtensionDirectiveArgumentsCoercer<'_, '_> {
    pub fn coerce_argument(
        &mut self,
        def: sdl::InputValueDefinition<'_>,
        value: Option<sdl::ConstValue<'_>>,
    ) -> Result<Option<(ExtensionInputValueRecord, InjectionStage)>, ExtensionInputValueError> {
        self.value_path.clear();
        self.value_path.push(def.name().into());
        self.current_injection_stage = Default::default();
        let value = if let Some(value) = value {
            self.is_default_value = false;
            self.coerce_input_value(def.ty(), value)?
        } else if let Some(value) = def.default_value() {
            self.is_default_value = true;
            self.coerce_input_value(def.ty(), value)?
        } else if def.ty().is_non_null() {
            return Err(InputValueError::MissingRequiredArgument(def.name().to_string()).into());
        } else {
            return Ok(None);
        };
        Ok(Some((value, self.current_injection_stage)))
    }

    fn coerce_input_value(
        &mut self,
        ty: sdl::Type<'_>,
        value: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        if ty.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type_value(ty.name(), value)?;
            for _ in ty.wrappers().filter(|w| matches!(w, sdl::WrappingType::List)) {
                value = ExtensionInputValueRecord::List(vec![value]);
            }
            return Ok(value);
        }

        self.coerce_type_value(ty.name(), ty.wrappers(), value)
    }

    fn coerce_type_value(
        &mut self,
        name: &str,
        mut wrappers: sdl::TypeWrappersIter,
        value: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let Some(wrapper) = wrappers.next() else {
            if value.is_null() {
                return Ok(ExtensionInputValueRecord::Null);
            }
            return self.coerce_named_type_value(name, value);
        };

        match wrapper {
            sdl::WrappingType::NonNull => {
                if value.is_null() {
                    Err(InputValueError::UnexpectedNull {
                        expected: self.type_name(name, wrappers, Some(sdl::WrappingType::NonNull)),
                        path: self.path(),
                    }
                    .into())
                } else {
                    self.coerce_type_value(name, wrappers, value)
                }
            }
            sdl::WrappingType::List => match value {
                sdl::ConstValue::List(array) => array
                    .into_iter()
                    .enumerate()
                    .map(|(ix, value)| {
                        self.value_path.push(ix.into());
                        let value = self.coerce_type_value(name, wrappers.clone(), value)?;
                        self.value_path.pop();
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(ExtensionInputValueRecord::List),
                _ => Err(InputValueError::MissingList {
                    actual: value.into(),
                    expected: self.type_name(name, wrappers, Some(sdl::WrappingType::List)),
                    path: self.path(),
                }
                .into()),
            },
        }
    }

    fn coerce_named_type_value(
        &mut self,
        name: &str,
        value: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        if matches!(name, "ID" | "String" | "Int" | "Float" | "Boolean") {
            return self.coerce_scalar_value(name, value);
        }
        if let Some((_, scalar)) = self.sdl.grafbase_scalars.iter().find(|(s, _)| s == name) {
            let Some(value) = value.as_str() else {
                return Err(InputValueError::IncorrectScalarType {
                    actual: value.into(),
                    expected: name.to_string(),
                    path: self.path(),
                }
                .into());
            };
            return match scalar {
                GrafbaseScalar::InputValueSet => {
                    self.current_injection_stage = self.current_injection_stage.max(InjectionStage::Query);
                    self.coerce_input_value_set(value)
                        .map(ExtensionInputValueRecord::InputValueSet)
                        .map_err(Into::into)
                }
                GrafbaseScalar::FieldSet => {
                    self.current_injection_stage = self.current_injection_stage.max(InjectionStage::Response);
                    let field_set = self.coerce_field_set(value)?;
                    self.requirements = self.requirements.union(&field_set);
                    Ok(ExtensionInputValueRecord::FieldSet(field_set))
                }
                GrafbaseScalar::UrlTemplate | GrafbaseScalar::JsonTemplate => {
                    self.current_injection_stage = self.current_injection_stage.max(InjectionStage::Query);
                    let template = TemplateRecord::new(
                        value.to_string(),
                        match scalar {
                            GrafbaseScalar::UrlTemplate => TemplateEscaping::Url,
                            GrafbaseScalar::JsonTemplate => TemplateEscaping::Json,
                            _ => unreachable!(),
                        },
                    )?;
                    let id = self.graph.templates.len().into();
                    self.graph.templates.push(template);
                    Ok(ExtensionInputValueRecord::Template(id))
                }
            };
        }
        let Some(def) = self.sdl.doc.definitions().find_map(|def| match def {
            sdl::Definition::Type(def) if def.name() == name => Some(def),
            _ => None,
        }) else {
            return Err(ExtensionInputValueError::UnknownType { name: name.to_string() });
        };
        match def {
            sdl::TypeDefinition::Scalar(def) => self.coerce_scalar_value(def.name(), value),
            sdl::TypeDefinition::Enum(def) => self.coerce_enum_value(def, value),
            sdl::TypeDefinition::InputObject(def) => self.coerce_input_objet_value(def, value),
            _ => Err(ExtensionInputValueError::NotAnInputType { name: name.to_string() }),
        }
    }

    fn coerce_input_objet_value(
        &mut self,
        input_object: sdl::InputObjectDefinition<'_>,
        obj: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let sdl::ConstValue::Object(object) = obj else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: obj.into(),
                path: self.path(),
            }
            .into());
        };

        let mut map = Vec::new();
        let mut fields = object.fields().collect::<Vec<_>>();

        if input_object.directives().any(|dir| dir.name() == "oneOf") {
            if fields.len() != 1 {
                return Err(InputValueError::ExactlyOneFIeldMustBePresentForOneOfInputObjects {
                    name: input_object.name().to_string(),
                    message: if fields.is_empty() {
                        "No field was provided".to_string()
                    } else {
                        format!(
                            "{} fields ({}) were provided",
                            fields.len(),
                            fields
                                .iter()
                                .format_with(",", |field, f| f(&format_args!("{}", field.name())))
                        )
                    },
                    path: self.path(),
                }
                .into());
            }
            let name = fields[0].name();
            if let Some(input_field) = input_object.fields().find(|input_field| input_field.name() == name) {
                let name_id = self.ingest_str(input_field.name());
                self.value_path.push(name_id.into());
                let field = fields.pop().unwrap();
                let value = self.coerce_input_value(input_field.ty(), field.value())?;
                map.push((name_id, value));
                self.value_path.pop();
            }
        } else {
            for input_value_def in input_object.fields() {
                let name_id = self.ingest_str(input_value_def.name());
                self.value_path.push(name_id.into());

                let value = if let Some(index) = fields.iter().position(|field| field.name() == input_value_def.name())
                {
                    let field = fields.swap_remove(index);
                    self.coerce_input_value(input_value_def.ty(), field.value())?
                } else if let Some(default_value) = input_value_def.default_value() {
                    self.coerce_input_value(input_value_def.ty(), default_value)?
                } else if input_value_def.ty().is_non_null() {
                    let error = InputValueError::UnexpectedNull {
                        expected: self.type_name(input_value_def.ty().name(), input_value_def.ty().wrappers(), None),
                        path: self.path(),
                    };
                    return Err(error.into());
                } else {
                    self.value_path.pop();
                    continue;
                };

                map.push((name_id, value));
                self.value_path.pop();
            }
        }

        if let Some(field) = fields.first() {
            let error = InputValueError::UnknownInputField {
                input_object: input_object.name().to_string(),
                name: field.name().to_string(),
                path: self.path(),
            };

            return Err(error.into());
        }

        Ok(ExtensionInputValueRecord::Map(map))
    }

    fn coerce_enum_value(
        &mut self,
        def: sdl::EnumDefinition<'_>,
        value: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        let string_value = match value {
            sdl::ConstValue::Enum(enm) => enm.as_str(),
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: def.name().to_string(),
                    actual: value.into(),
                    path: self.path(),
                }
                .into());
            }
        };
        if def.values().any(|value| value.value() == string_value) {
            return Ok(ExtensionInputValueRecord::EnumValue(self.ingest_str(string_value)));
        }
        Err(InputValueError::UnknownEnumValue {
            r#enum: def.name().to_string(),
            value: string_value.to_string(),
            path: self.path(),
        }
        .into())
    }

    fn coerce_scalar_value(
        &mut self,
        name: &str,
        value: sdl::ConstValue<'_>,
    ) -> Result<ExtensionInputValueRecord, ExtensionInputValueError> {
        use sdl::ConstValue;
        match name {
            "String" | "ID" => match value {
                ConstValue::String(s) => Some(ExtensionInputValueRecord::String(self.ingest_str(s.value()))),
                _ => None,
            },
            "Float" => match value {
                ConstValue::Int(n) => Some(ExtensionInputValueRecord::Float(n.value() as f64)),
                ConstValue::Float(f) => Some(ExtensionInputValueRecord::Float(f.value())),
                _ => None,
            },
            "Int" => match value {
                ConstValue::Int(n) => {
                    let n = i32::try_from(n.value()).map_err(|_| InputValueError::IncorrectScalarValue {
                        actual: n.to_string(),
                        expected: name.to_string(),
                        path: self.path(),
                    })?;
                    Some(ExtensionInputValueRecord::Int(n))
                }
                ConstValue::Float(f) if can_coerce_to_int(f.value()) => {
                    Some(ExtensionInputValueRecord::Int(f.value() as i32))
                }
                _ => None,
            },
            "Boolean" => match value {
                ConstValue::Boolean(b) => Some(ExtensionInputValueRecord::Boolean(b.value())),
                _ => None,
            },
            _ => return Ok(self.ingest_arbitrary_value(value)),
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

    fn ingest_arbitrary_value(&mut self, value: sdl::ConstValue<'_>) -> ExtensionInputValueRecord {
        match value {
            sdl::ConstValue::Null(_) => ExtensionInputValueRecord::Null,
            sdl::ConstValue::String(s) => ExtensionInputValueRecord::String(self.ingest_str(s.value())),
            sdl::ConstValue::Int(n) => ExtensionInputValueRecord::I64(n.value()),
            sdl::ConstValue::Float(f) => ExtensionInputValueRecord::Float(f.value()),
            sdl::ConstValue::Boolean(b) => ExtensionInputValueRecord::Boolean(b.value()),
            sdl::ConstValue::Enum(s) => ExtensionInputValueRecord::EnumValue(self.ingest_str(s.as_str())),
            sdl::ConstValue::List(list) => ExtensionInputValueRecord::List(
                list.into_iter()
                    .map(|value| self.ingest_arbitrary_value(value))
                    .collect(),
            ),
            sdl::ConstValue::Object(fields) => ExtensionInputValueRecord::Map(
                fields
                    .into_iter()
                    .map(|field| {
                        let name = self.ingest_str(field.name());
                        (name, self.ingest_arbitrary_value(field.value()))
                    })
                    .collect(),
            ),
        }
    }

    pub(super) fn type_name(
        &self,
        name: &str,
        iter: sdl::TypeWrappersIter,
        outer: Option<sdl::WrappingType>,
    ) -> String {
        let mut out = String::new();
        let mut wrappers = Vec::new();
        wrappers.extend(outer);
        wrappers.extend(iter);
        for wrapping in &wrappers {
            if let sdl::WrappingType::List = wrapping {
                out.push('[');
            }
        }
        out.push_str(name);
        for wrapping in wrappers.iter().rev() {
            match wrapping {
                sdl::WrappingType::NonNull => out.push('!'),
                sdl::WrappingType::List => out.push(']'),
            }
        }
        out
    }

    pub(super) fn path(&self) -> String {
        value_path_to_string(self, &self.value_path)
    }
}
