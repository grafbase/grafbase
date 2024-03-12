mod range;
pub use range::IdRange;

#[macro_export]
macro_rules! id_newtype {
    ($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $($output:tt)*) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                usize::from(*self).fmt(f)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                usize::from(*self).fmt(f)
            }
        }

        impl From<$name> for usize {
            fn from(id: $name) -> Self {
                (id.0.get() - 1) as usize
            }
        }

        impl$(<$($lt),+>)? std::ops::Index<$name> for $ty$(<$($lt),+>)? {
            type Output = $($output)*;

            fn index(&self, index: $name) -> &Self::Output {
                &self.$field[usize::from(index)]
            }
        }

        impl$(<$($lt),+>)? std::ops::IndexMut<$name> for $ty$(<$($lt),+>)? {
            fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                &mut self.$field[usize::from(index)]
            }
        }

        impl$(<$($lt),+>)? std::ops::Index<$crate::IdRange<$name>> for $ty$(<$($lt),+>)? {
            type Output = [$($output)*];

            fn index(&self, range: $crate::IdRange<$name>) -> &Self::Output {
                let $crate::IdRange { start, end } = range;
                let start = usize::from(start);
                let end = usize::from(end);
                &self.$field[start..end]
            }
        }

        impl$(<$($lt),+>)? std::ops::IndexMut<$crate::IdRange<$name>> for $ty$(<$($lt),+>)? {
            fn index_mut(&mut self, range: $crate::IdRange<$name>) -> &mut Self::Output {
                let $crate::IdRange { start, end } = range;
                let start = usize::from(start);
                let end = usize::from(end);
                &mut self.$field[start..end]
            }
        }
    };
}

#[macro_export]
macro_rules! U32 {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $output:ty | unless $msg:literal $(max $max:expr)?,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
            pub struct $name(std::num::NonZeroU32);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $(assert!(value <= $max, "{}", $msg);)?
                    Self(
                        u32::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU32::new(value + 1))
                            .expect($msg)
                    )
                }
            }

            impl From<$name> for u32 {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as u32
                }
            }

            $crate::id_newtype!{ $ty$(<$($lt),+>)?.$field[$name] => $output }
        )*
    }
}

#[macro_export]
macro_rules! U16 {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $output:ty | unless $msg:literal $(max $max:expr)?,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
            pub struct $name(std::num::NonZeroU16);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $(assert!(value <= $max, "{}", $msg);)?
                    Self(
                        u16::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU16::new(value + 1))
                            .expect($msg)
                    )
                }
            }

            impl From<$name> for u16 {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as u16
                }
            }

            $crate::id_newtype!{ $ty$(<$($lt),+>)?.$field[$name] => $output }
        )*
    }
}
