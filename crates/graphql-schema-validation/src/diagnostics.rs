mod constructors;

pub(crate) use constructors::*;

#[derive(Default)]
pub struct Diagnostics {
    pub(crate) errors: Vec<miette::Report>,
}

impl Diagnostics {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &miette::Report> {
        self.errors.iter()
    }
}
