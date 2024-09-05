use std::cmp::Ordering;

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

// impl<W, T1, T2> Readable<W> for &(T1, T2)
// where
//     for<'x> &'x T1: Readable<W>,
//     for<'x> &'x T2: Readable<W>,
// {
//     type Reader<'a> = (Reader<'a, &'a T1, W>, Reader<'a, &'a T2, W>)
//     where
//         Self: 'a,
//         W: 'a;
//
//     fn read<'w>(self, world: &'w W) -> Self::Reader<'w>
//     where
//         Self: 'w,
//     {
//         let (a, b) = self;
//         (a.read(world), b.read(world))
//     }
// }

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

pub struct ReadableIterator<'w, I, W> {
    iter: I,
    world: &'w W,
}

impl<'w, I, W> Clone for ReadableIterator<'w, I, W>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            world: self.world,
        }
    }
}

impl<'w, I, W> ReadableIterator<'w, I, W> {
    pub fn new(iter: I, world: &'w W) -> Self {
        Self { iter, world }
    }
}

impl<'w, I, W> Iterator for ReadableIterator<'w, I, W>
where
    I: Iterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    type Item = Reader<'w, <I as Iterator>::Item, W>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.read(self.world))
    }
}

impl<'w, I, W> ExactSizeIterator for ReadableIterator<'w, I, W>
where
    I: ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'w, I, W> std::iter::FusedIterator for ReadableIterator<'w, I, W>
where
    I: std::iter::FusedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
}

impl<'w, I, W> std::iter::DoubleEndedIterator for ReadableIterator<'w, I, W>
where
    I: std::iter::DoubleEndedIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| item.read(self.world))
    }
}

impl<'w, I, W> std::fmt::Debug for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'w, I, W> std::cmp::Ord for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.len().cmp(&other.len()).then_with(|| {
            for (left, right) in self.clone().zip(other.clone()) {
                match left.cmp(&right) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            Ordering::Equal
        })
    }
}

impl<'w, I, W> std::cmp::PartialEq for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.clone().zip(other.clone()).all(|(left, right)| left == right)
    }
}

impl<'w, I, W> std::cmp::Eq for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::Eq,
{
}

impl<'w, I, W> std::cmp::PartialOrd for ReadableIterator<'w, I, W>
where
    I: std::iter::ExactSizeIterator,
    <I as Iterator>::Item: Readable<W> + 'w,
    I: Clone,
    <Self as Iterator>::Item: std::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.len().partial_cmp(&other.len()) {
            Some(Ordering::Equal) => (),
            other => return other,
        };

        for (left, right) in self.clone().zip(other.clone()) {
            match left.partial_cmp(&right) {
                Some(Ordering::Equal) => continue,
                other => return other,
            }
        }

        Some(Ordering::Equal)
    }
}
