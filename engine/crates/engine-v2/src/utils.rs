pub use schema::IdRange;

macro_rules! id_newtype {
    ($ty:ident.$field:ident[$name:ident] => $out:ty) => {
        impl std::ops::Index<$name> for $ty {
            type Output = $out;

            fn index(&self, index: $name) -> &$out {
                &self.$field[usize::from(index)]
            }
        }

        impl std::ops::IndexMut<$name> for $ty {
            fn index_mut(&mut self, index: $name) -> &mut $out {
                &mut self.$field[usize::from(index)]
            }
        }

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

        impl std::ops::Index<crate::utils::IdRange<$name>> for $ty {
            type Output = [$out];

            fn index(&self, range: crate::utils::IdRange<$name>) -> &Self::Output {
                let crate::utils::IdRange { start, end } = range;
                let start = usize::from(start);
                let end = usize::from(end);
                &self.$field[start..end]
            }
        }
    };
}

macro_rules! id_newtypes_u16 {
    ($($ty:ident.$field:ident[$name:ident] => $out:ident unless $msg:literal,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
            pub(crate) struct $name(std::num::NonZeroU16);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    Self(
                        u16::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU16::new(value + 1))
                            .expect($msg)
                    )
                }
            }

            crate::utils::id_newtype!{ $ty.$field[$name] => $out }
        )*
    }
}

pub(crate) use id_newtype;
pub(crate) use id_newtypes_u16;
