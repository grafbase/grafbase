pub trait Invert {
    type Output;

    fn invert(&self) -> Self::Output;
}

impl<T> Invert for Option<T> {
    type Output = Option<()>;

    /// Returns [`Some(())`] if the option is [`None`], or [`None`] if the option is [`Some(_)`]
    fn invert(&self) -> Self::Output {
        match self {
            Some(_) => None,
            None => Some(()),
        }
    }
}
