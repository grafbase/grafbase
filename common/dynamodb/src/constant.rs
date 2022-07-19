pub const PK: &str = "__pk";
pub const SK: &str = "__sk";
pub const TYPE: &str = "__type";
pub const CREATED_AT: &str = "__created_at";
pub const UPDATED_AT: &str = "__updated_at";
pub const RELATION_NAMES: &str = "__relation_names";
pub const TYPE_INDEX_PK: &str = "__gsi1pk";
pub const TYPE_INDEX_SK: &str = "__gsi1sk";
pub const INVERTED_INDEX_PK: &str = "__gsi2pk";
pub const INVERTED_INDEX_SK: &str = "__gsi2sk";
// Used in rows created to enforce uniqueness. Refers to the `pk` holding that particular unique value.
pub const ITEM_PK: &str = "__item_pk";
