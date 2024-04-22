#[derive(Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub enum ConstraintType {
    #[default]
    Unique,
}

#[derive(Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct Constraint {
    // This is an option for backwards compatability reasons.
    // Constraints didn't always have a name.
    // Can possibly make it required in the future.
    name: Option<String>,
    fields: Vec<String>,
    // This is also here for backwards compatability
    field: String,
    pub r#type: ConstraintType,
}

impl Constraint {
    pub fn name(&self) -> &str {
        self.name
            .as_deref()
            .or_else(|| Some(self.fields.first()?))
            .unwrap_or(&self.field)
    }

    pub fn fields(&self) -> Vec<String> {
        if self.fields.is_empty() {
            return vec![self.field.clone()];
        }
        self.fields.clone()
    }

    pub fn unique(name: String, fields: Vec<String>) -> Constraint {
        Constraint {
            name: Some(name),
            fields,
            field: String::new(),
            r#type: ConstraintType::Unique,
        }
    }
}
