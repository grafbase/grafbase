mod range;
pub use range::IdRange;

#[macro_export]
macro_rules! debug_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = stringify!($name);
                write!(
                    f,
                    "{}#{}",
                    name.strip_suffix("Id").unwrap_or(name),
                    usize::from(*self)
                )
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = stringify!($name);
                write!(
                    f,
                    "{}#{}",
                    name.strip_suffix("Id").unwrap_or(name),
                    usize::from(*self)
                )
            }
        }
    };
}

#[macro_export]
macro_rules! index {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?$(.$field:ident)+[$name:ident] => $output:ty $(|proxy($ty2:ident$(.$field2:ident)+))*,)*) => {
        $(
            impl$(<$($lt),+>)? std::ops::Index<$name> for $ty$(<$($lt),+>)? {
                type Output = $output;

                fn index(&self, index: $name) -> &Self::Output {
                    &self$(.$field)+[usize::from(index)]
                }
            }

            impl$(<$($lt),+>)? std::ops::IndexMut<$name> for $ty$(<$($lt),+>)? {
                fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                    &mut self$(.$field)+[usize::from(index)]
                }
            }

            impl$(<$($lt),+>)? std::ops::Index<$crate::IdRange<$name>> for $ty$(<$($lt),+>)? {
                type Output = [$output];

                fn index(&self, range: $crate::IdRange<$name>) -> &Self::Output {
                let $crate::IdRange { start, end } = range;
                let start = usize::from(start);
                let end = usize::from(end);
                &self$(.$field)+[start..end]
                }
            }

            impl$(<$($lt),+>)? std::ops::IndexMut<$crate::IdRange<$name>> for $ty$(<$($lt),+>)? {
                fn index_mut(&mut self, range: $crate::IdRange<$name>) -> &mut Self::Output {
                    let $crate::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &mut self$(.$field)+[start..end]
                }
            }
            $(
                impl std::ops::Index<$name> for $ty2 {
                    type Output = $output;

                    fn index(&self, index: $name) -> &Self::Output {
                        &self$(.$field2)+[index]
                    }
                }
                impl std::ops::Index<$crate::IdRange<$name>> for $ty2 {
                    type Output = [$output];

                    fn index(&self, range: $crate::IdRange<$name>) -> &Self::Output {
                        &self$(.$field2)+[range]
                    }
                }
            )*
        )*
    };
}

#[macro_export]
macro_rules! NonZeroU32 {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $output:ty $(|max($max:expr))? $(|proxy($ty2:ident$(.$field2:ident)+))* ,)*) => {
        $(
            $crate::NonZeroU32! { $name $(|max($max))?, }
            $crate::index!{ $ty$(<$($lt),+>)?.$field[$name] => $output $(|proxy($ty2$(.$field2)+))*, }
        )*
    };
    ($($name:ident $(|max($max:expr))?,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(std::num::NonZeroU32);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $(assert!(value <= $max, "{} id {} exceeds maximum {}", stringify!($name), value, stringify!($max));)?
                    Self(
                        u32::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU32::new(value + 1))
                            .expect(concat!("Too many ", stringify!($name)))
                    )
                }
            }

            impl From<$name> for u32 {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as u32
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as usize
                }
            }

            $crate::debug_display! { $name }
        )*
    }
}

#[macro_export]
macro_rules! NonZeroU16 {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $output:ty $(| $(max($max:expr))?)? $(|proxy($ty2:ident$(.$field2:ident)+))*,)*) => {
        $(
            $crate::NonZeroU16! { $name $(|max($max))?, }
            $crate::index!{ $ty$(<$($lt),+>)?.$field[$name] => $output $(|proxy($ty2$(.$field2)+))*, }
        )*
    };
    ($($name:ident $(|max($max:expr))?,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(std::num::NonZeroU16);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $(assert!(value <= $max, "{} id {} exceeds maximum {}", stringify!($name), value, stringify($ty));)?
                    Self(
                        u16::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU16::new(value + 1))
                            .expect(concat!("Too many ", stringify!($name)))
                    )
                }
            }

            impl From<$name> for u16 {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as u16
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as usize
                }
            }

            $crate::debug_display! { $name }
        )*
    }
}

#[macro_export]
macro_rules! U8 {
    ($($ty:ident$(<$( $lt:lifetime ),+>)?.$field:ident[$name:ident] => $output:ty $(| $(max($max:expr))?)? $(|proxy($ty2:ident$(.$field2:ident)+))*,)*) => {
        $(
            $crate::NonZeroU16! { $name $(|max($max))?, }
            $crate::index!{ $ty$(<$($lt),+>)?.$field[$name] => $output $(|proxy($ty2$(.$field2)+))*, }
        )*
    };
    ($($name:ident $(|max($max:expr))?,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(u8);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    $(assert!(value <= $max, "{} id {} exceeds maximum {}", stringify!($name), value, stringify($ty));)?
                    Self(
                        u8::try_from(value)
                            .ok()
                            .expect(concat!("Too many ", stringify!($name)))
                    )
                }
            }

            impl From<$name> for u8 {
                fn from(id: $name) -> Self {
                    id.0
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    id.0 as usize
                }
            }

            $crate::debug_display! { $name }
        )*
    }
}
