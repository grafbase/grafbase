mod iter;

pub use iter::*;

pub trait Iter: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug {}
impl<T: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug> Iter for T {}

pub trait Readable<W> {
    type Reader<'a>
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w;
}

impl<W> Readable<W> for () {
    type Reader<'a> = () where W: 'a;
    fn read<'w>(self, _world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w,
    {
    }
}

pub type Reader<'a, T, W> = <T as Readable<W>>::Reader<'a>;

// / Convenient implementation to write:
// / `id.read(schema)` rather than `(*id).read(schema)` when id is a ref from the schema
impl<W, T: Copy + Readable<W>> Readable<W> for &T {
    type Reader<'a> = Reader<'a, T, W>
    where
        Self: 'a,
        W: 'a;

    fn read<'s>(self, world: &'s W) -> Self::Reader<'s>
    where
        Self: 's,
    {
        (*self).read(world)
    }
}

impl<W, T: Readable<W>> Readable<W> for Option<T> {
    type Reader<'a> = Option<Reader<'a, T, W>>
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w,
    {
        self.map(|value| value.read(world))
    }
}

impl<W, T> Readable<W> for &[T]
where
    for<'x> &'x T: Readable<W>,
{
    type Reader<'a> = ReadableIterator<'a, std::slice::Iter<'a, T>, W>
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w,
    {
        ReadableIterator::new(self.iter(), world)
    }
}
