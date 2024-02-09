use super::{ResponseBuilder, ResponseDataPart, ResponsePart};
use crate::response::{ResponseData, ResponseObject, ResponseValue};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseDataPartId(u32);

impl From<usize> for ResponseDataPartId {
    fn from(value: usize) -> Self {
        ResponseDataPartId(value.try_into().expect("Too many parts"))
    }
}

impl From<ResponseDataPartId> for usize {
    fn from(id: ResponseDataPartId) -> Self {
        id.0 as usize
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseObjectId {
    part_id: ResponseDataPartId,
    index: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseListId {
    part_id: ResponseDataPartId,
    offset: u32,
    length: u32,
}

impl std::ops::Index<ResponseDataPartId> for ResponseBuilder {
    type Output = ResponseDataPart;

    fn index(&self, index: ResponseDataPartId) -> &Self::Output {
        &self.parts[usize::from(index)]
    }
}

impl std::ops::Index<ResponseObjectId> for ResponseBuilder {
    type Output = ResponseObject;

    fn index(&self, index: ResponseObjectId) -> &Self::Output {
        &self.parts[usize::from(index.part_id)].objects[index.index as usize]
    }
}

impl std::ops::IndexMut<ResponseObjectId> for ResponseBuilder {
    fn index_mut(&mut self, index: ResponseObjectId) -> &mut Self::Output {
        &mut self.parts[usize::from(index.part_id)].objects[index.index as usize]
    }
}

impl std::ops::Index<ResponseListId> for ResponseBuilder {
    type Output = [ResponseValue];

    fn index(&self, index: ResponseListId) -> &Self::Output {
        &self.parts[usize::from(index.part_id)][index]
    }
}

impl std::ops::IndexMut<ResponseListId> for ResponseBuilder {
    fn index_mut(&mut self, index: ResponseListId) -> &mut Self::Output {
        &mut self.parts[usize::from(index.part_id)][index]
    }
}

impl std::ops::Index<ResponseDataPartId> for ResponseData {
    type Output = ResponseDataPart;

    fn index(&self, index: ResponseDataPartId) -> &Self::Output {
        &self.parts[usize::from(index)]
    }
}

impl std::ops::Index<ResponseObjectId> for ResponseData {
    type Output = ResponseObject;

    fn index(&self, index: ResponseObjectId) -> &Self::Output {
        &self.parts[usize::from(index.part_id)].objects[index.index as usize]
    }
}

impl std::ops::Index<ResponseListId> for ResponseData {
    type Output = [ResponseValue];

    fn index(&self, index: ResponseListId) -> &Self::Output {
        &self.parts[usize::from(index.part_id)][index]
    }
}

impl std::ops::Index<ResponseListId> for ResponseDataPart {
    type Output = [ResponseValue];

    fn index(&self, index: ResponseListId) -> &Self::Output {
        let start = index.offset as usize;
        let end = (index.offset + index.length) as usize;
        &self.lists[start..end]
    }
}

impl std::ops::IndexMut<ResponseListId> for ResponseDataPart {
    fn index_mut(&mut self, index: ResponseListId) -> &mut Self::Output {
        let start = index.offset as usize;
        let end = (index.offset + index.length) as usize;
        &mut self.lists[start..end]
    }
}

impl ResponsePart {
    pub fn push_object(&mut self, object: ResponseObject) -> ResponseObjectId {
        let offset = self.data.objects.len() as u32;
        self.data.objects.push(object);
        ResponseObjectId {
            part_id: self.id,
            index: offset,
        }
    }

    pub fn push_list(&mut self, value: &[ResponseValue]) -> ResponseListId {
        let offset = self.data.lists.len() as u32;
        let length = value.len() as u32;
        self.data.lists.extend_from_slice(value);
        ResponseListId {
            part_id: self.id,
            offset,
            length,
        }
    }
}
