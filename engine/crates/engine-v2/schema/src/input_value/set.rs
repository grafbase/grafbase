use crate::InputValueDefinitionId;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueSet(Vec<InputValueSetSelection>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueSetSelection {
    pub id: InputValueDefinitionId,
    pub subselection: InputValueSet,
}

impl FromIterator<InputValueSetSelection> for InputValueSet {
    fn from_iter<T: IntoIterator<Item = InputValueSetSelection>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.id);
        Self(fields)
    }
}

impl std::ops::Deref for InputValueSet {
    type Target = [InputValueSetSelection];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
