use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

use operation::{PositionedResponseKey, ResponseKey};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct ErrorPath(Vec<ErrorPathSegment>);

impl std::ops::Deref for ErrorPath {
    type Target = Vec<ErrorPathSegment>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ErrorPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ErrorPathSegment {
    Field(ResponseKey),
    Index(usize),
    UnknownField(Box<str>),
}

pub trait InsertIntoErrorPath {
    fn insert_into(self, path: &mut ErrorPath);
}

impl InsertIntoErrorPath for ResponseKey {
    fn insert_into(self, path: &mut ErrorPath) {
        path.0.push(ErrorPathSegment::Field(self));
    }
}

impl InsertIntoErrorPath for PositionedResponseKey {
    fn insert_into(self, path: &mut ErrorPath) {
        path.0.push(ErrorPathSegment::Field(self.response_key));
    }
}

impl InsertIntoErrorPath for usize {
    fn insert_into(self, path: &mut ErrorPath) {
        path.0.push(ErrorPathSegment::Index(self));
    }
}

impl InsertIntoErrorPath for u32 {
    fn insert_into(self, path: &mut ErrorPath) {
        path.0.push(ErrorPathSegment::Index(self as usize));
    }
}

impl InsertIntoErrorPath for String {
    fn insert_into(self, path: &mut ErrorPath) {
        path.0.push(ErrorPathSegment::UnknownField(self.into_boxed_str()));
    }
}

trait InsertAllIntoErrorPath {
    fn insert_all_into(self, path: &mut ErrorPath);
}

impl<T: InsertIntoErrorPath> InsertAllIntoErrorPath for T {
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.insert_into(path);
    }
}

impl<'a, T> InsertAllIntoErrorPath for &'a [T]
where
    &'a T: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        for item in self {
            item.insert_into(path);
        }
    }
}

impl<'a, T> InsertAllIntoErrorPath for &'a Vec<T>
where
    &'a T: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.as_slice().insert_all_into(path);
    }
}

impl<T> InsertAllIntoErrorPath for Ref<'_, Vec<T>>
where
    for<'a> &'a T: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.deref().insert_all_into(path)
    }
}

impl<T> InsertAllIntoErrorPath for &Ref<'_, Vec<T>>
where
    for<'a> &'a T: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.deref().insert_all_into(path)
    }
}

impl<T> InsertAllIntoErrorPath for RefMut<'_, Vec<T>>
where
    for<'a> &'a T: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.deref().insert_all_into(path)
    }
}

impl<'a, T1, T2> InsertAllIntoErrorPath for &'a (&'a [T1], T2)
where
    &'a T1: InsertIntoErrorPath,
    &'a T2: InsertAllIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.0.insert_all_into(path);
        self.1.insert_all_into(path);
    }
}

impl<'a, T1, T2> InsertAllIntoErrorPath for &'a (&'a [T1], &'a [T2])
where
    &'a T1: InsertIntoErrorPath,
    &'a T2: InsertIntoErrorPath,
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.0.insert_all_into(path);
        self.1.insert_all_into(path);
    }
}

impl<T1: InsertAllIntoErrorPath, T2: InsertAllIntoErrorPath> InsertAllIntoErrorPath for (T1, T2) {
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.0.insert_all_into(path);
        self.1.insert_all_into(path);
    }
}

impl<T1: InsertAllIntoErrorPath, T2: InsertAllIntoErrorPath, T3: InsertAllIntoErrorPath> InsertAllIntoErrorPath
    for (T1, T2, T3)
{
    fn insert_all_into(self, path: &mut ErrorPath) {
        self.0.insert_all_into(path);
        self.1.insert_all_into(path);
        self.2.insert_all_into(path);
    }
}

impl<T: InsertAllIntoErrorPath> From<T> for ErrorPath {
    fn from(t: T) -> Self {
        let mut path = ErrorPath(Vec::new());
        t.insert_all_into(&mut path);
        path
    }
}
