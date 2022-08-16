#![allow(dead_code)]

pub const DEFAULT_SCHEMA: &str = include_str!("../graphql/default/schema.graphql");
pub const DEFAULT_MUTATION: &str = include_str!("../graphql/default/mutation.graphql");
pub const DEFAULT_QUERY: &str = include_str!("../graphql/default/query.graphql");

pub const UPDATED_SCHEMA: &str = include_str!("../graphql/updated/schema.graphql");
pub const UPDATED_MUTATION: &str = include_str!("../graphql/updated/mutation.graphql");
pub const UPDATED_QUERY: &str = include_str!("../graphql/updated/query.graphql");

pub const RELATIONS_SCHEMA: &str = include_str!("../graphql/relations/schema.graphql");
pub const RELATIONS_MUTATION: &str = include_str!("../graphql/relations/mutation.graphql");
pub const RELATIONS_QUERY: &str = include_str!("../graphql/relations/query.graphql");
pub const RELATIONS_LINK_BLOG_TO_AUTHOR: &str = include_str!("../graphql/relations/link-blog-to-author.graphql");
pub const RELATIONS_UNLINK_BLOG_FROM_AUTHOR: &str =
    include_str!("../graphql/relations/unlink-blog-from-author.graphql");

pub const UNIQUE_SCHEMA: &str = include_str!("../graphql/unique/schema.graphql");
pub const UNIQUE_CREATE_MUTATION: &str = include_str!("../graphql/unique/create-mutation.graphql");
pub const UNIQUE_DELETE_MUTATION: &str = include_str!("../graphql/unique/delete-mutation.graphql");
pub const UNIQUE_PAGINATED_QUERY: &str = include_str!("../graphql/unique/paginated-query.graphql");
pub const UNIQUE_QUERY: &str = include_str!("../graphql/unique/query.graphql");

pub const CONCURRENCY_SCHEMA: &str = include_str!("../graphql/concurrency/schema.graphql");
pub const CONCURRENCY_MUTATION: &str = include_str!("../graphql/concurrency/mutation.graphql");
pub const CONCURRENCY_QUERY: &str = include_str!("../graphql/concurrency/query.graphql");
