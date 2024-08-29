use crate::{Change, ChangeKind};

pub(super) struct Paths<'a, T>
where
    T: AsRef<str>,
{
    diff: &'a [Change],
    resolved_spans: &'a [T],
    source: &'a str,

    paths: Vec<([&'a str; 3], usize)>,
}

impl<'a, T> Paths<'a, T>
where
    T: AsRef<str>,
{
    pub(super) fn new(diff: &'a [Change], resolved_spans: &'a [T], source: &'a str) -> Self {
        let mut paths = diff
            .iter()
            .enumerate()
            .map(|(idx, diff)| (split_path(&diff.path), idx))
            .collect::<Vec<_>>();

        paths.sort();

        Paths {
            diff,
            source,
            resolved_spans,

            paths,
        }
    }

    pub(super) fn iter_top_level<'b: 'a>(&'b self) -> impl Iterator<Item = ChangeView<'a, T>> + 'b {
        self.paths
            .iter()
            .filter(|(change, _)| change[1].is_empty() && change[2].is_empty())
            .map(|(_, idx)| ChangeView { paths: self, idx: *idx })
    }

    pub(super) fn iter_second_level<'b: 'a>(&'b self, parent: &'b str) -> impl Iterator<Item = ChangeView<'a, T>> + 'b {
        self.paths
            .iter()
            .filter(move |(change, _)| change[0] == parent && !change[1].is_empty() && change[2].is_empty())
            .map(|(_, idx)| ChangeView { paths: self, idx: *idx })
    }

    pub(super) fn iter_exact<'b: 'a>(&'b self, path: [&'b str; 3]) -> impl Iterator<Item = ChangeView<'a, T>> + 'b {
        let first = self.paths.partition_point(|(diff_path, _)| diff_path < &path);
        self.paths[first..]
            .iter()
            .take_while(move |(diff_path, _)| diff_path == &path)
            .enumerate()
            .map(move |(idx, _)| ChangeView {
                paths: self,
                idx: first + idx,
            })
    }

    pub(crate) fn source(&self) -> &'a str {
        self.source
    }
}

pub(super) struct ChangeView<'a, T>
where
    T: AsRef<str>,
{
    paths: &'a Paths<'a, T>,
    idx: usize,
}

impl<'a, T: AsRef<str>> Clone for ChangeView<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: AsRef<str>> Copy for ChangeView<'a, T> {}

impl<'a, T> ChangeView<'a, T>
where
    T: AsRef<str>,
{
    pub(crate) fn kind(self) -> ChangeKind {
        self.paths.diff[self.idx].kind
    }

    pub(crate) fn resolved_str(self) -> &'a str {
        self.paths.resolved_spans[self.idx].as_ref()
    }

    pub(crate) fn path(self) -> &'a str {
        &self.paths.diff[self.idx].path
    }

    /// The second part of the path. E.g. "foo.bar" -> "bar".
    pub(crate) fn second_level(self) -> Option<&'a str>
    where
        T: AsRef<str>,
    {
        Some(split_path(&self.paths.diff[self.idx].path)[1]).filter(|s| !s.is_empty())
    }
}

fn split_path(path: &str) -> [&str; 3] {
    let mut segments = path.split('.');
    let path = std::array::from_fn(|_| segments.next().unwrap_or(""));
    debug_assert!(segments.next().is_none());
    path
}
