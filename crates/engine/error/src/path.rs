use std::cell::{Ref, RefMut};

use operation::{PositionedResponseKey, ResponseKey};

#[derive(Debug, Clone)]
pub struct ErrorPath(Vec<ErrorPathSegment>);

impl std::ops::Deref for ErrorPath {
    type Target = [ErrorPathSegment];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum ErrorPathSegment {
    Field(ResponseKey),
    Index(usize),
    UnknownField(Box<str>),
}

impl From<ResponseKey> for ErrorPathSegment {
    fn from(key: ResponseKey) -> Self {
        ErrorPathSegment::Field(key)
    }
}

impl From<PositionedResponseKey> for ErrorPathSegment {
    fn from(key: PositionedResponseKey) -> Self {
        ErrorPathSegment::Field(key.response_key)
    }
}

impl From<usize> for ErrorPathSegment {
    fn from(index: usize) -> Self {
        ErrorPathSegment::Index(index)
    }
}

impl From<u32> for ErrorPathSegment {
    fn from(index: u32) -> Self {
        ErrorPathSegment::Index(index as usize)
    }
}

impl From<String> for ErrorPathSegment {
    fn from(name: String) -> Self {
        ErrorPathSegment::UnknownField(name.into_boxed_str())
    }
}

impl From<Vec<ErrorPathSegment>> for ErrorPath {
    fn from(segments: Vec<ErrorPathSegment>) -> Self {
        ErrorPath(segments)
    }
}

impl<S> From<RefMut<'_, Vec<S>>> for ErrorPath
where
    for<'a> &'a S: Into<ErrorPathSegment>,
{
    fn from(path: RefMut<'_, Vec<S>>) -> Self {
        ErrorPath(path.iter().map(Into::into).collect())
    }
}

impl<S> From<Ref<'_, Vec<S>>> for ErrorPath
where
    for<'a> &'a S: Into<ErrorPathSegment>,
{
    fn from(path: Ref<'_, Vec<S>>) -> Self {
        ErrorPath(path.iter().map(Into::into).collect())
    }
}

impl<S> From<&Vec<S>> for ErrorPath
where
    for<'a> &'a S: Into<ErrorPathSegment>,
{
    fn from(path: &Vec<S>) -> Self {
        ErrorPath(path.iter().map(Into::into).collect())
    }
}

impl<S1: Into<ErrorPath>, S2: Into<ErrorPathSegment>> From<(S1, S2)> for ErrorPath {
    fn from((path, segment): (S1, S2)) -> Self {
        let mut path: ErrorPath = path.into();
        path.0.push(segment.into());
        path
    }
}
