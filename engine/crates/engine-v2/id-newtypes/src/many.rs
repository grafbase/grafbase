#[derive(serde::Serialize, serde::Deserialize)]
pub struct IdToMany<Id, V>(Vec<(Id, V)>);

impl<Id, V> Default for IdToMany<Id, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<Id: Ord + Copy, V> FromIterator<(Id, V)> for IdToMany<Id, V> {
    fn from_iter<T: IntoIterator<Item = (Id, V)>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl<Id: Ord + Copy, V> From<Vec<(Id, V)>> for IdToMany<Id, V> {
    fn from(mut relations: Vec<(Id, V)>) -> Self {
        relations.sort_unstable_by_key(|(id, _)| *id);
        Self(relations)
    }
}

impl<Id, V> AsRef<[(Id, V)]> for IdToMany<Id, V> {
    fn as_ref(&self) -> &[(Id, V)] {
        &self.0
    }
}

impl<Id, V> IdToMany<Id, V> {
    pub fn from_sorted_vec(relations: Vec<(Id, V)>) -> Self {
        Self(relations)
    }

    pub fn ids(&self) -> impl ExactSizeIterator<Item = Id> + '_
    where
        Id: Copy,
    {
        self.0.iter().map(|(id, _)| *id)
    }

    pub fn find_all(&self, id: Id) -> impl Iterator<Item = &V> + '_
    where
        Id: Copy + Ord,
    {
        ValueIterator {
            relations: self,
            id,
            idx: self.0.partition_point(|probe| probe.0 < id),
        }
    }
}

struct ValueIterator<'a, Id, V> {
    relations: &'a IdToMany<Id, V>,
    id: Id,
    idx: usize,
}

impl<'a, Id: Eq, V> Iterator for ValueIterator<'a, Id, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.relations.0.get(self.idx)?;
        if key == &self.id {
            self.idx += 1;
            Some(value)
        } else {
            None
        }
    }
}
