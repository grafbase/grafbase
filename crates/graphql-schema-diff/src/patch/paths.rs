use crate::{Change, ChangeKind};

pub(super) struct Paths<'a, T>
where
    T: AsRef<str>,
{
    diff: &'a [Change],
    resolved_spans: &'a [T],
    source: &'a str,

    /// All the diff entries, but sorted by path.
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

        paths.sort_unstable();

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
            .filter(move |(change, _)| change[0] == parent && !change[1].is_empty())
            .map(|(_, idx)| ChangeView { paths: self, idx: *idx })
    }

    pub(super) fn iter_exact<'b: 'a>(&'b self, path: [&'b str; 3]) -> impl Iterator<Item = ChangeView<'a, T>> + 'b {
        let first = self.paths.partition_point(|(diff_path, _)| diff_path < &path);
        self.paths[first..]
            .iter()
            .take_while(move |(diff_path, _)| diff_path == &path)
            .map(move |(_, idx)| ChangeView { paths: self, idx: *idx })
    }

    pub(crate) fn source(&self) -> &'a str {
        self.source
    }

    pub(crate) fn added_interface_impls<'b>(&'b self, prefix: &'b str) -> impl Iterator<Item = &'a str> + 'b {
        let start = self.paths.partition_point(|([found, _, _], _)| *found < prefix);

        self.paths[start..]
            .iter()
            .take_while(move |([name, _, _], _)| *name == prefix)
            .filter(|(_, idx)| matches!(self.diff[*idx].kind, ChangeKind::AddInterfaceImplementation))
            .map(|([_, interface, _], _)| interface.trim_start_matches('&'))
    }

    pub(crate) fn is_interface_impl_removed(&self, prefix: &str, interface: &str) -> bool {
        self.paths
            .binary_search_by(|(path, idx)| {
                [path[0], path[1].trim_start_matches('&')]
                    .cmp(&[prefix, interface])
                    .then(self.diff[*idx].kind.cmp(&ChangeKind::RemoveInterfaceImplementation))
            })
            .is_ok()
    }
}

pub(super) struct ChangeView<'a, T>
where
    T: AsRef<str>,
{
    paths: &'a Paths<'a, T>,
    idx: usize,
}

impl<T: AsRef<str>> Clone for ChangeView<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: AsRef<str>> Copy for ChangeView<'_, T> {}

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

    pub(crate) fn second_and_third_level(self) -> [&'a str; 2] {
        let [_first, second, third] = split_path(self.path());
        debug_assert!(!second.is_empty());
        debug_assert!(!third.is_empty());
        [second, third]
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
