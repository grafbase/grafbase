use crate::config::StringId;

/// Very simple implementation of string interning.
#[derive(Default)]
pub(crate) struct Strings<'a>(indexmap::IndexSet<&'a str>);

impl<'a> Strings<'a> {
    /// Interns a str, avoiding allocation if the same string has already been
    /// interned.
    pub(crate) fn intern(&mut self, string: &'a str) -> StringId {
        if let Some(idx) = self.0.get_index_of(string) {
            return StringId(idx);
        }

        StringId(self.0.insert_full(string).0)
    }

    pub fn into_vec(self) -> Vec<String> {
        self.0.into_iter().map(|string| string.to_string()).collect()
    }
}
