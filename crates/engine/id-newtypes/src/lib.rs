mod bitset;
mod many;
mod one;
mod range;
pub use bitset::*;
pub use many::*;
pub use one::*;
pub use range::*;

#[macro_export]
macro_rules! forward {
    ($(impl Index<$index:ident, Output = $output:tt> for $ty:ident $(< $( $ltOrGeneric:tt $( : $bound:tt $(+ $bounds:tt )* )? ),+ >)? $(.$field:ident)+,)*) => {
        $(
            impl$(< $( $ltOrGeneric $( : $bound $(+ $bounds )* )? ),+ >)? std::ops::Index<$index> for $ty$(< $( $ltOrGeneric  ),+ >)?{
                type Output = $output;

                fn index(&self, index: $index) -> &Self::Output {
                    &self$(.$field)+[index]
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! forward_with_range {
    ($(impl Index<$index:ident, Output = $output:tt> for $ty:ident $(< $( $ltOrGeneric:tt $( : $bound:tt $(+ $bounds:tt )* )? ),+ >)? $(.$field:ident)+,)*) => {
        $(
            impl$(< $( $ltOrGeneric $( : $bound $(+ $bounds )* )? ),+ >)? std::ops::Index<$index> for $ty$(< $( $ltOrGeneric  ),+ >)?{
                type Output = $output;

                fn index(&self, index: $index) -> &Self::Output {
                    &self$(.$field)+[index]
                }
            }

            impl$(< $( $ltOrGeneric $( : $bound $(+ $bounds )* )? ),+ >)? std::ops::IndexMut<$index> for $ty$(< $( $ltOrGeneric  ),+ >)?{
                fn index_mut(&mut self, index: $index) -> &mut Self::Output {
                    &mut self$(.$field)+[index]
                }
            }

            impl$(< $( $ltOrGeneric $( : $bound $(+ $bounds )* )? ),+ >)? std::ops::Index<$crate::IdRange<$index>> for $ty$(< $( $ltOrGeneric  ),+ >)? {
                type Output = [$output];

                fn index(&self, range: $crate::IdRange<$index>) -> &Self::Output {
                    &self$(.$field)+[range]
                }
            }
        )*
    };
}
