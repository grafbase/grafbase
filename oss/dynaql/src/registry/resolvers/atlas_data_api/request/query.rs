mod delete_many;
mod delete_one;
mod find_many;
mod find_one;
mod insert_many;
mod insert_one;
mod update_one;

pub(super) use delete_many::DeleteMany;
pub(super) use delete_one::DeleteOne;
pub(super) use find_many::FindMany;
pub(super) use find_one::FindOne;
pub(super) use insert_many::InsertMany;
pub(super) use insert_one::InsertOne;
pub(super) use update_one::UpdateOne;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub(super) enum AtlasQuery {
    FindOne(FindOne),
    FindMany(FindMany),
    InsertOne(InsertOne),
    InsertMany(InsertMany),
    DeleteOne(DeleteOne),
    DeleteMany(DeleteMany),
    UpdateOne(UpdateOne),
}
