mod order_by_direction;
pub use order_by_direction::OrderByDirection;

use unindent::unindent;

pub struct DynaqlEnums;

impl DynaqlEnums {
    pub fn sdl() -> String {
        OrderByDirection::sdl()
    }
}

pub trait DynaqlEnum {
    fn ty() -> &'static str;
    fn values() -> Vec<String>;
    fn sdl() -> String {
        unindent(&format!(
            r#"
            enum {ty} {{
                {values}
            }}
            "#,
            ty = Self::ty(),
            values = Self::values().join("\n    ")
        ))
    }
}
