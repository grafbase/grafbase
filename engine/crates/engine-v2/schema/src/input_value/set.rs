use crate::InputValueDefinitionId;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueSet(Vec<InputValueSetItem>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueSetItem {
    pub id: InputValueDefinitionId,
    pub subselection: InputValueSet,
}

impl FromIterator<InputValueSetItem> for InputValueSet {
    fn from_iter<T: IntoIterator<Item = InputValueSetItem>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.id);
        Self(fields)
    }
}

impl std::ops::Deref for InputValueSet {
    type Target = [InputValueSetItem];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
