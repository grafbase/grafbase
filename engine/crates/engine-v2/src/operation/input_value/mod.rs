mod query;
mod variable;

pub(crate) use query::*;
use schema::Schema;
pub(crate) use variable::*;

use super::Variables;

#[derive(Clone, Copy)]
pub(crate) struct InputValueContext<'a> {
    pub schema: &'a Schema,
    pub query_input_values: &'a QueryInputValues,
    pub variables: &'a Variables,
}
