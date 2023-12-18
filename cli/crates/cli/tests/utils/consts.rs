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

pub const SEARCH_SCHEMA: &str = include_str!("../graphql/search/schema.graphql");
pub const SEARCH_CREATE_LIST: &str = include_str!("../graphql/search/create-list.graphql");
pub const SEARCH_SEARCH_LIST: &str = include_str!("../graphql/search/search-list.graphql");
pub const SEARCH_CREATE_REQUIRED: &str = include_str!("../graphql/search/create-required.graphql");
pub const SEARCH_SEARCH_REQUIRED: &str = include_str!("../graphql/search/search-required.graphql");
pub const SEARCH_CREATE_OPTIONAL: &str = include_str!("../graphql/search/create-optional.graphql");
pub const SEARCH_SEARCH_OPTIONAL: &str = include_str!("../graphql/search/search-optional.graphql");
pub const SEARCH_PAGINATION: &str = include_str!("../graphql/search/search-pagination.graphql");
pub const SEARCH_METADATA_FIELDS: &str = include_str!("../graphql/search/search-metadata-fields.graphql");
pub const SEARCH_CREATE_PERSON: &str = include_str!("../graphql/search/create-person.graphql");
pub const SEARCH_SEARCH_PERSON: &str = include_str!("../graphql/search/search-person.graphql");

pub const ENVIRONMENT_SCHEMA: &str = include_str!("../graphql/environment/schema.graphql");

pub const AUTH_JWT_PROVIDER_SCHEMA: &str = include_str!("../graphql/auth/jwt.graphql");
pub const AUTH_QUERY_TODOS: &str = include_str!("../graphql/auth/query.graphql");
pub const AUTH_OIDC_PROVIDER_SCHEMA: &str = include_str!("../graphql/auth/oidc.graphql");
pub const AUTH_JWKS_PROVIDER_WITH_ISSUER_SCHEMA: &str = include_str!("../graphql/auth/jwks-issuer.graphql");
pub const AUTH_JWKS_PROVIDER_WITH_ENDPOINT_SCHEMA: &str = include_str!("../graphql/auth/jwks-endpoint.graphql");
pub const AUTH_JWKS_PROVIDER_WITH_ISSUER_ENDPOINT_SCHEMA: &str =
    include_str!("../graphql/auth/jwks-issuer-endpoint.graphql");
pub const AUTH_PUBLIC_GLOBAL_SCHEMA: &str = include_str!("../graphql/auth/public-global.graphql");
pub const AUTH_PUBLIC_TYPE_SCHEMA: &str = include_str!("../graphql/auth/public-type.graphql");
pub const AUTH_TYPE_FIELD_RESOLVER_SCHEMA: &str = include_str!("../graphql/auth/type-field-resolver.graphql");
pub const AUTH_CREATE_MUTATION: &str = include_str!("../graphql/auth/create.graphql");
pub const AUTH_QUERY_WITH_TEXT: &str = include_str!("../graphql/auth/query-with-text.graphql");
pub const AUTH_ENTRYPOINT_FIELD_RESOLVER_SCHEMA: &str =
    include_str!("../graphql/auth/entrypoint-field-resolver.graphql");
pub const AUTH_ENTRYPOINT_QUERY_TEXT: &str = include_str!("../graphql/auth/entrypoint-query-text.graphql");
pub const AUTH_ENTRYPOINT_MUTATION_TEXT: &str = include_str!("../graphql/auth/entrypoint-mutation-text.graphql");

pub const AUTHORIZER_SCHEMA: &str = include_str!("../graphql/auth/authorizer/authorizer.graphql");

pub const INTROSPECTION_QUERY: &str = include_str!("../graphql/introspection.graphql");

pub const RESERVED_DATES_SCHEMA: &str = include_str!("../graphql/reserved_dates/schema.graphql");
pub const RESERVED_DATES_NESTED_CREATION: &str = include_str!("../graphql/reserved_dates/nested-creation.graphql");
pub const RESERVED_DATES_CREATE_TODO: &str = include_str!("../graphql/reserved_dates/create-todo.graphql");
pub const RESERVED_DATES_CREATE_TODO_LIST: &str = include_str!("../graphql/reserved_dates/create-todo-list.graphql");

pub const DEFAULT_DIRECTIVE_CREATE_USER1: &str = include_str!("../graphql/default_directive/create-user1.graphql");
pub const DEFAULT_DIRECTIVE_CREATE_USER2: &str = include_str!("../graphql/default_directive/create-user2.graphql");
pub const DEFAULT_DIRECTIVE_SCHEMA: &str = include_str!("../graphql/default_directive/schema.graphql");

pub const OWNER_TODO_SCHEMA: &str = include_str!("../graphql/owner/global/todo/todo-schema.graphql");
pub const OWNER_TODO_MIXED_SCHEMA: &str = include_str!("../graphql/owner/global/todo/todo-mixed-schema.graphql");
pub const OWNER_TODO_OWNER_CREATE_SCHEMA: &str =
    include_str!("../graphql/owner/global/todo/todo-owner-create-schema.graphql");
pub const OWNER_TODO_CREATE: &str = include_str!("../graphql/owner/global/todo/todo-create.graphql");
pub const OWNER_TODO_GET: &str = include_str!("../graphql/owner/global/todo/todo-get.graphql");
pub const OWNER_TODO_UPDATE: &str = include_str!("../graphql/owner/global/todo/todo-update.graphql");
pub const OWNER_TODO_LIST: &str = include_str!("../graphql/owner/global/todo/todo-list.graphql");
pub const OWNER_TODO_DELETE: &str = include_str!("../graphql/owner/global/todo/todo-delete.graphql");

pub const OWNER_TWITTER_SCHEMA: &str = include_str!("../graphql/owner/global/twitter/twitter-schema.graphql");
pub const OWNER_TWITTER_USER_CREATE: &str = include_str!("../graphql/owner/global/twitter/user-create.graphql");
pub const OWNER_TWITTER_USER_GET_BY_ID: &str = include_str!("../graphql/owner/global/twitter/user-get-by-id.graphql");
pub const OWNER_TWITTER_USER_GET_BY_EMAIL: &str =
    include_str!("../graphql/owner/global/twitter/user-get-by-email.graphql");
pub const OWNER_TWITTER_TWEET_CREATE: &str = include_str!("../graphql/owner/global/twitter/tweet-create.graphql");
pub const OWNER_TWITTER_USER_AND_TWEETS_GET_BY_ID: &str =
    include_str!("../graphql/owner/global/twitter/user-and-tweets-get-by-id.graphql");

pub const BATCH_SCHEMA: &str = include_str!("../graphql/batch/schema.graphql");
pub const BATCH_CREATE: &str = include_str!("../graphql/batch/create.graphql");
pub const BATCH_UPDATE: &str = include_str!("../graphql/batch/update.graphql");
pub const BATCH_DELETE: &str = include_str!("../graphql/batch/delete.graphql");
pub const BATCH_COLLECT: &str = include_str!("../graphql/batch/collect.graphql");
