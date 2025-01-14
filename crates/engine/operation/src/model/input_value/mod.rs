mod query;
mod variable;

pub use query::*;
use schema::Schema;
pub use variable::*;

use crate::Variables;

#[derive(Clone, Copy)]
pub struct InputValueContext<'a> {
    pub schema: &'a Schema,
    pub query_input_values: &'a QueryInputValues,
    pub variables: &'a Variables,
}

impl<'a> From<InputValueContext<'a>> for &'a Schema {
    fn from(ctx: InputValueContext<'a>) -> Self {
        ctx.schema
    }
}
