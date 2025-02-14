mod error;
mod extension;
mod field_set;
mod input_value_set;
mod path;
mod schema;

pub(crate) use error::*;
pub(crate) use extension::*;
pub(crate) use path::*;

fn can_coerce_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
