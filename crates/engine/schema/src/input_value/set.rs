use crate::InputValueDefinitionId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputValueSet {
    All,
    SelectionSet(Vec<InputValueSelection>),
}

impl Default for InputValueSet {
    fn default() -> Self {
        Self::SelectionSet(Vec::new())
    }
}

impl InputValueSet {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::All => false,
            Self::SelectionSet(selection_set) => selection_set.is_empty(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueSelection {
    pub definition_id: InputValueDefinitionId,
    pub subselection: InputValueSet,
}

impl From<Vec<InputValueSelection>> for InputValueSet {
    fn from(mut selections: Vec<InputValueSelection>) -> Self {
        selections.sort_unstable_by_key(|selection| selection.definition_id);
        Self::SelectionSet(selections)
    }
}

impl FromIterator<InputValueSelection> for InputValueSet {
    fn from_iter<T: IntoIterator<Item = InputValueSelection>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
