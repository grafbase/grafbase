use std::{cell::RefMut, ops::Deref};

use operation::{PositionedResponseKey, ResponseKey};

use crate::response::ResponseValueId;

#[derive(Debug, Clone)]
pub(crate) struct ErrorPath(Vec<ErrorPathSegment>);

impl std::ops::Deref for ErrorPath {
    type Target = [ErrorPathSegment];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ErrorPathSegment {
    Field(ResponseKey),
    Index(usize),
    UnknownField(String),
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
        ErrorPathSegment::UnknownField(name)
    }
}

impl<Segment: Into<ErrorPathSegment>> From<(&Vec<ResponseValueId>, Segment)> for ErrorPath {
    fn from((path, segment): (&Vec<ResponseValueId>, Segment)) -> Self {
        let mut segments = Vec::with_capacity(path.len() + 1);
        for segment in path {
            segments.push(segment.into());
        }
        segments.push(segment.into());
        ErrorPath(segments)
    }
}

impl<Segment: Into<ErrorPathSegment>> From<(RefMut<'_, Vec<ResponseValueId>>, Segment)> for ErrorPath {
    fn from((path, segment): (RefMut<'_, Vec<ResponseValueId>>, Segment)) -> Self {
        (path.deref(), segment).into()
    }
}

impl From<RefMut<'_, Vec<ResponseValueId>>> for ErrorPath {
    fn from(path: RefMut<'_, Vec<ResponseValueId>>) -> Self {
        ErrorPath(path.iter().map(ErrorPathSegment::from).collect())
    }
}

impl From<&Vec<ResponseValueId>> for ErrorPath {
    fn from(path: &Vec<ResponseValueId>) -> Self {
        ErrorPath(path.iter().map(ErrorPathSegment::from).collect())
    }
}

impl From<Vec<ErrorPathSegment>> for ErrorPath {
    fn from(segments: Vec<ErrorPathSegment>) -> Self {
        ErrorPath(segments)
    }
}
