use crate::{
    execution::ExecStringId,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
};

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    items: Vec<ReadSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelection {
    pub response_position: usize,
    pub response_name: ExecStringId,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReadSelection> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a ReadSelectionSet {
    type Item = &'a ReadSelection;

    type IntoIter = <&'a Vec<ReadSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl FromIterator<ReadSelection> for ReadSelectionSet {
    fn from_iter<T: IntoIterator<Item = ReadSelection>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl ContextAwareDebug for ReadSelectionSet {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSelectionSet")
            .field("items", &ctx.debug(&self.items))
            .finish()
    }
}

impl ContextAwareDebug for ReadSelection {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSelection")
            .field("name", &ctx.strings[self.response_name].to_string())
            .field("subselection", &ctx.debug(&self.subselection))
            .finish()
    }
}
