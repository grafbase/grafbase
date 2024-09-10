mod iter;

pub use iter::*;

pub trait Iter: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug {}
impl<T: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug> Iter for T {}

pub trait Walk<G> {
    type Walker<'a>
    where
        Self: 'a,
        G: 'a;

    fn walk<'a>(self, graph: &'a G) -> Self::Walker<'a>
    where
        Self: 'a;
}

impl<G> Walk<G> for () {
    type Walker<'a> = () where G: 'a;
    fn walk<'a>(self, _: &'a G) -> Self::Walker<'a>
    where
        Self: 'a,
    {
    }
}

pub type Walker<'a, T, G> = <T as Walk<G>>::Walker<'a>;

// / Convenient implementation to write:
// / `id.read(schema)` rather than `(*id).read(schema)` when id is a ref from the schema
impl<G, T: Copy + Walk<G>> Walk<G> for &T {
    type Walker<'a> = Walker<'a, T, G>
    where
        Self: 'a,
        G: 'a;

    fn walk<'a>(self, graph: &'a G) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        (*self).walk(graph)
    }
}

impl<G, T: Walk<G>> Walk<G> for Option<T> {
    type Walker<'a> = Option<Walker<'a, T, G>>
    where
        Self: 'a,
        G: 'a;

    fn walk<'a>(self, graph: &'a G) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        self.map(|value| value.walk(graph))
    }
}

impl<G, T> Walk<G> for &[T]
where
    for<'a> &'a T: Walk<G>,
{
    type Walker<'a> = WalkIterator<'a, std::slice::Iter<'a, T>, G>
    where
        Self: 'a,
        G: 'a;

    fn walk<'a>(self, graph: &'a G) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        WalkIterator::new(self.iter(), graph)
    }
}
