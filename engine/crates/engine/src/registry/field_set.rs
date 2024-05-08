use serde_json::{Map, Value};

/// Checks if all the fields of this FieldSet are present in the given JSON object
pub fn all_fieldset_fields_are_present(fieldset: &registry_v2::FieldSet, object: &Map<String, Value>) -> bool {
    selections_are_present(object, &fieldset.0)
}

fn selections_are_present(object: &Map<String, Value>, selections: &[registry_v2::Selection]) -> bool {
    selections.iter().all(|selection| {
        if !object.contains_key(&selection.field) {
            return false;
        }
        if selection.selections.is_empty() {
            return true;
        }

        match object.get(&selection.field) {
            Some(Value::Object(object)) => {
                // Make sure any sub-selections are also present
                selections_are_present(object, &selection.selections)
            }
            Some(Value::Null) => {
                // We assume the value is nullable if it's present as a null
                true
            }
            _ => false,
        }
    })
}

pub struct FieldSetDisplay<'a>(pub &'a registry_v2::FieldSet);

impl std::fmt::Display for FieldSetDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, selection) in self.0 .0.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", SelectionDisplay(selection))?;
        }
        Ok(())
    }
}

struct SelectionDisplay<'a>(&'a registry_v2::Selection);

impl std::fmt::Display for SelectionDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let registry_v2::Selection { field, selections } = self.0;
        write!(f, "{field}")?;
        if !selections.is_empty() {
            write!(f, " {{")?;
            for selection in selections {
                write!(f, " {}", SelectionDisplay(selection))?;
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}
