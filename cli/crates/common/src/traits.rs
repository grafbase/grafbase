pub trait Invert<T> {
    fn invert(&self) -> T;
}

impl<T> Invert<Option<()>> for Option<T> {
    fn invert(&self) -> Option<()> {
        match self {
            Some(_) => None,
            None => Some(()),
        }
    }
}
