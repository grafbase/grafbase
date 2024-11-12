use id_newtypes::IdRange;
use walker::Walk;

use crate::{Schema, SchemaField, SchemaFieldArgumentId};

impl std::fmt::Debug for SchemaField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.definition().name())
            .field(
                "arguments",
                &ArgumentsDebug {
                    schema: self.schema,
                    arguments: self.sorted_argument_ids,
                },
            )
            .finish()
    }
}

pub(super) struct ArgumentsDebug<'a> {
    pub schema: &'a Schema,
    pub arguments: IdRange<SchemaFieldArgumentId>,
}

impl std::fmt::Debug for ArgumentsDebug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.arguments
                    .walk(self.schema)
                    .map(|arg| (arg.definition().name(), arg.value())),
            )
            .finish()
    }
}
