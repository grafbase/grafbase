#![allow(dead_code)]

pub const DEFAULT_SCHEMA: &str = include_str!("../graphql/default/schema.graphql");
pub const DEFAULT_CREATE: &str = include_str!("../graphql/default/create.graphql");
pub const DEFAULT_UPDATE: &str = include_str!("../graphql/default/update.graphql");
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
pub const RELATIONS_UNLINK_AUTHORS_FROM_BLOG: &str =
    include_str!("../graphql/relations/unlink-authors-from-blog.graphql");
pub const REALTIONS_LINK_SECONDARY_AUTHOR_TO_BLOG: &str =
    include_str!("../graphql/relations/link-secondary-author-to-blog.graphql");
pub const REALTIONS_RENAME_AUTHOR: &str = include_str!("../graphql/relations/rename-author.graphql");

pub const UNIQUE_SCHEMA: &str = include_str!("../graphql/unique/schema.graphql");
pub const UNIQUE_CREATE_MUTATION: &str = include_str!("../graphql/unique/create-mutation.graphql");
pub const UNIQUE_DELETE_MUTATION: &str = include_str!("../graphql/unique/delete-mutation.graphql");
pub const UNIQUE_PAGINATED_QUERY: &str = include_str!("../graphql/unique/paginated-query.graphql");
pub const UNIQUE_QUERY: &str = include_str!("../graphql/unique/query.graphql");
pub const UNIQUE_QUERY_BY_NAME: &str = include_str!("../graphql/unique/query-by-name.graphql");
pub const UNIQUE_UPDATE_MUTATION: &str = include_str!("../graphql/unique/update-mutation.graphql");
pub const UNIQUE_UPDATE_UNIQUE_MUTATION: &str = include_str!("../graphql/unique/update-unique-mutation.graphql");
pub const UNIQUE_UPDATE_BY_NAME_MUTATION: &str = include_str!("../graphql/unique/update-by-name-mutation.graphql");
pub const UNIQUE_UPDATE_UNIQUE_BY_NAME_MUTATION: &str =
    include_str!("../graphql/unique/update-unique-by-name-mutation.graphql");

pub const LENGTH_SCHEMA: &str = include_str!("../graphql/length/schema.graphql");
pub const LENGTH_CREATE_MUTATION: &str = include_str!("../graphql/length/create-mutation.graphql");
pub const LENGTH_UPDATE_MUTATION: &str = include_str!("../graphql/length/update-mutation.graphql");

pub const CONCURRENCY_SCHEMA: &str = include_str!("../graphql/concurrency/schema.graphql");
pub const CONCURRENCY_MUTATION: &str = include_str!("../graphql/concurrency/mutation.graphql");
pub const CONCURRENCY_QUERY: &str = include_str!("../graphql/concurrency/query.graphql");

pub const PAGINATION_SCHEMA: &str = include_str!("../graphql/pagination/schema.graphql");
pub const PAGINATION_CREATE_TODO: &str = include_str!("../graphql/pagination/create-todo.graphql");
pub const PAGINATION_CREATE_TODO_LIST: &str = include_str!("../graphql/pagination/create-todo-list.graphql");
pub const PAGINATION_PAGINATE_TODOS: &str = include_str!("../graphql/pagination/paginate-todos.graphql");
pub const PAGINATION_PAGINATE_TODO_LISTS: &str = include_str!("../graphql/pagination/paginate-todo-lists.graphql");

pub const COERCION_SCHEMA: &str = include_str!("../graphql/coercion/schema.graphql");
pub const COERCION_CREATE_DUMMY: &str = include_str!("../graphql/coercion/create-dummy.graphql");

pub const SCALARS_SCHEMA: &str = include_str!("../graphql/scalars/schema.graphql");
pub const SCALARS_CREATE_OPTIONAL: &str = include_str!("../graphql/scalars/create-optional.graphql");
pub const SCALARS_CREATE_REQUIRED: &str = include_str!("../graphql/scalars/create-required.graphql");

pub const ENVIRONMENT_SCHEMA: &str = include_str!("../graphql/environment/schema.graphql");

pub const JWT_PROVIDER_SCHEMA: &str = include_str!("../graphql/auth/schema.graphql");
pub const JWT_PROVIDER_QUERY: &str = include_str!("../graphql/auth/query.graphql");

pub const INTROSPECTION_QUERY: &str = include_str!("../graphql/introspection.graphql");

pub const RESERVED_DATES_SCHEMA: &str = include_str!("../graphql/reserved_dates/schema.graphql");
pub const RESERVED_DATES_NESTED_CREATION: &str = include_str!("../graphql/reserved_dates/nested-creation.graphql");
pub const RESERVED_DATES_CREATE_TODO: &str = include_str!("../graphql/reserved_dates/create-todo.graphql");
pub const RESERVED_DATES_CREATE_TODO_LIST: &str = include_str!("../graphql/reserved_dates/create-todo-list.graphql");

pub const DEFAULT_DIRECTIVE_CREATE_USER1: &str = include_str!("../graphql/default_directive/create-user1.graphql");
pub const DEFAULT_DIRECTIVE_CREATE_USER2: &str = include_str!("../graphql/default_directive/create-user2.graphql");
pub const DEFAULT_DIRECTIVE_SCHEMA: &str = include_str!("../graphql/default_directive/schema.graphql");
