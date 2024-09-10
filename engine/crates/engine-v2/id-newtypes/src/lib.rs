mod bitset;
mod many;
mod range;
pub use bitset::*;
pub use many::*;
pub use range::*;

use grafbase_workspace_hack as _;

#[macro_export]
macro_rules! forward {
    ($(impl Index<$index:ident, Output = $output:tt> for $ty:ident$(.$field:ident)+,)*) => {
        $(
            impl std::ops::Index<$index> for $ty {
                type Output = $output;

                fn index(&self, index: $index) -> &Self::Output {
                    &self$(.$field)+[index]
                }
            }

            impl std::ops::Index<$crate::IdRange<$index>> for $ty {
                type Output = [$output];

                fn index(&self, range: $crate::IdRange<$index>) -> &Self::Output {
                    &self$(.$field)+[range]
                }
            }
        )*
    };
}
