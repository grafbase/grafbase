use super::*;

pub(crate) type StringWalker<'a> = Walker<'a, StringId>;

impl<'a> StringWalker<'a> {
    pub(crate) fn as_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.id)
    }
}
