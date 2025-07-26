use std::marker::PhantomData;

pub struct IdToOne<Id, V> {
    inner: Vec<V>,
    _marker: PhantomData<Id>,
}

impl<Id, V> IdToOne<Id, V>
where
    Id: Copy,
{
    pub fn init(x: V, n: usize) -> Self
    where
        V: Copy,
    {
        Self {
            inner: vec![x; n],
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &V)>
    where
        Id: From<usize>,
    {
        self.inner.iter().enumerate().map(|(ix, v)| (Id::from(ix), v))
    }
}

impl<Id, V> std::ops::Index<Id> for IdToOne<Id, V>
where
    usize: From<Id>,
    Id: Copy,
{
    type Output = V;
    fn index(&self, index: Id) -> &Self::Output {
        &self.inner[usize::from(index)]
    }
}

impl<Id, V> std::ops::IndexMut<Id> for IdToOne<Id, V>
where
    usize: From<Id>,
    Id: Copy,
{
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        &mut self.inner[usize::from(index)]
    }
}

impl<Id, V> IntoIterator for IdToOne<Id, V>
where
    Id: From<usize>,
{
    type Item = (Id, V);
    type IntoIter = IntoIter<Id, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter().enumerate(),
            _marker: PhantomData,
        }
    }
}

pub struct IntoIter<Id, V> {
    inner: std::iter::Enumerate<std::vec::IntoIter<V>>,
    _marker: PhantomData<Id>,
}

impl<Id, V> Iterator for IntoIter<Id, V>
where
    Id: From<usize>,
{
    type Item = (Id, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(ix, v)| (Id::from(ix), v))
    }
}
