mod delete_one;
pub(super) mod find_many;
pub(super) mod find_one;
mod insert_one;

pub(super) use delete_one::DeleteOne;
pub(super) use find_many::FindMany;
pub(super) use find_one::FindOne;
pub(super) use insert_one::InsertOne;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub(super) enum AtlasQuery {
    FindOne(FindOne),
    FindMany(FindMany),
    InsertOne(InsertOne),
    DeleteOne(DeleteOne),
}
