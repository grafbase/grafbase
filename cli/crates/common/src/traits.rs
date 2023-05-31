pub trait Invert<T> {
    fn invert(&self) -> T;
}

impl<T> Invert<Option<()>> for Option<T> {
    /// Returns [`Some(())`] if the option is [`None`], or [`None`] if the option is [`Some(_)`]
    fn invert(&self) -> Option<()> {
        match self {
            Some(_) => None,
            None => Some(()),
        }
    }
}
