use super::*;

pub(crate) type Extension<'a> = View<'a, ExtensionId, ExtensionRecord>;

pub(crate) struct ExtensionRecord {
    pub(crate) url: StringId,
    pub(crate) name: StringId,
}

impl Subgraphs {
    pub(crate) fn iter_extensions(&self) -> impl ExactSizeIterator<Item = Extension<'_>> {
        self.extensions
            .iter()
            .enumerate()
            .map(|(idx, record)| View { id: idx.into(), record })
    }
}
