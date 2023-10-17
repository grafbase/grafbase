use serde_json::{Map, Value};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldSet(Vec<Selection>);

impl FieldSet {
    pub fn new(selections: impl IntoIterator<Item = Selection>) -> Self {
        FieldSet(selections.into_iter().collect())
    }

    /// Checks if all the fields of this FieldSet are present in the given JSON object
    pub fn all_fields_are_present(&self, object: &Map<String, Value>) -> bool {
        selections_are_present(object, &self.0)
    }
}

fn selections_are_present(object: &Map<String, Value>, selections: &[Selection]) -> bool {
    selections.iter().all(|selection| {
        if !object.contains_key(&selection.field) {
            return false;
        }
        if selection.selections.is_empty() {
            return true;
        }
        // Make sure any sub-selections are also present
        let Some(object) = object.get(&selection.field).and_then(Value::as_object) else {
            return false;
        };
        selections_are_present(object, &selection.selections)
    })
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct Selection {
    pub field: String,
    pub selections: Vec<Selection>,
}

impl std::fmt::Display for FieldSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, selection) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{selection}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Selection { field, selections } = self;
        write!(f, "{field}")?;
        if !selections.is_empty() {
            write!(f, " {{")?;
            for (i, selection) in selections.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?;
                }
                write!(f, "{selection}")?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}
