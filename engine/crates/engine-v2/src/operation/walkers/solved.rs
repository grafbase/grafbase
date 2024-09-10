use crate::operation::{SolvedRequiredField, SolvedRequiredFieldSet};

use super::OperationWalker;

pub type SolvedRequiredFieldSetWalker<'a> = OperationWalker<'a, &'a SolvedRequiredFieldSet>;
pub type SolvedRequiredFieldWalker<'a> = OperationWalker<'a, &'a SolvedRequiredField>;

impl std::fmt::Debug for SolvedRequiredFieldSetWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SolvedRequiredFieldSet")
            .field(&self.item.iter().map(|field| self.walk(field)).collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Debug for SolvedRequiredFieldWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolvedRequiredField")
            .field("field", &self.walk(self.item.field_id))
            .finish()
    }
}
