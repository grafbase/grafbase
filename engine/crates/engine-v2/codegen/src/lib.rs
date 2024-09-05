pub trait Iter: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator {}
impl<T: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator> Iter for T {}

pub trait Readable<W> {
    type Reader<'a>
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w;
}

pub type Reader<'a, T, W> = <T as Readable<W>>::Reader<'a>;

/// Convenient implementation to write:
/// `id.read(schema)` rather than `(*id).read(schema)` when id is a ref from the schema
// impl<W, T: Copy + Readable<W>> Readable<W> for &T {
//     type Reader<'a> = Reader<'a, T, W>
//     where
//         Self: 'a,
//         W: 'a;
//
//     fn read<'s>(self, world: &'s W) -> Self::Reader<'s>
//     where
//         Self: 's,
//     {
//         (*self).read(world)
//     }
// }

impl<W, T1, T2> Readable<W> for &(T1, T2)
where
    for<'x> &'x T1: Readable<W>,
    for<'x> &'x T2: Readable<W>,
{
    type Reader<'a> = (Reader<'a, &'a T1, W>, Reader<'a, &'a T2, W>)
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w,
    {
        let (a, b) = self;
        (a.read(world), b.read(world))
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
    type Reader<'a> = MapRead<'a, std::slice::Iter<'a, T>, W>
    where
        Self: 'a,
        W: 'a;

    fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
    where
        Self: 'w,
    {
        MapRead {
            world,
            iter: self.iter(),
        }
    }
}

pub struct MapRead<'w, I, W> {
    pub world: &'w W,
    pub iter: I,
}

impl<'w, I, W> Iterator for MapRead<'w, I, W>
where
    I: Iterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    type Item = Reader<'w, <I as Iterator>::Item, W>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.read(self.world))
    }
}

impl<'w, I, W> ExactSizeIterator for MapRead<'w, I, W>
where
    I: ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'w, I, W> std::iter::FusedIterator for MapRead<'w, I, W>
where
    I: std::iter::FusedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
}

impl<'w, I, W> std::iter::DoubleEndedIterator for MapRead<'w, I, W>
where
    I: std::iter::DoubleEndedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| item.read(self.world))
    }
}
