# Relations

A Relation is a way to link multiple `modelized` entities together.

## Directive

This describe the behavior of the optional `@relation` directive.

```graphql
directive @relation(
  """
  The name of the relation
  """
  name: String!
) on FIELD_DEFINITION
```

You'll need to use this directive when their are multiple relations to the same
`Entity` or when you want explicitly relations.

## What is a relation?

We'll go through every possibility for a relation in Grafbase SDL to see what is
expected for the generation and the behavior.

## One-to-One

For instance let's take a Schema representing a Blog where an user can publish
**ONE** post.

```graphql
type User @model {
  id: ID!
  firstname: String!
  lastname: String!
  postPublished: Post
}

type Post @model {
  id: ID!
  content: String!
  publishedBy: User
}
```

The relation here is represented by the field `pubished_by`. It is what can be
called a `One-to-One` relationship.

An `User` can only have at most **ONE** `Post` published, and a `Post` can only be published
by at most **ONE** `User`.

### Generated Queries

#### Create

You defined your schema, and now you want to be able to:

- Create a new User without any Post attached to it.
- Create a new User and create a Post attached.
- Create a new User and link it to a Post already created. (If the Post is already linked to another User, it will fail).

```graphql
"""
You can choose between creating a new Post, or linking this user to an existing,
non-linked post.
"""
input UserCreateInputPostPublished {
  create: UserInputPostPublishedPost
  link: ID
}

"""
If you choose to create a new Post when you create a new User, as this is an
`One-to-One` relation, we cannot let you create a User inside the Post for the
User.
"""
input UserCreateInputPostPublished {
  content: String!
}

input UserCreateInput {
  firstname: String!
  lastname: String!
  postPublished: UserCreateInputPostPublished
}

input UserBulkInput {
  users: [UserBulkInput]
}

type UserCreatePayload {
  user: User
}

type UserBulkCreatePayload {
  users: [User]
}

type Mutation {
  userCreate(input: UserCreateInput): UserCreatePayload
  userCreateBulk(input: UserBulkInput): UserBulkCreatePayload
  ...
}
```

#### Link

##### Specialized

Imagine you already have an user but you now want to link it:

- Link an existing User to an existing Post.

If you do not name your relation with `@relation`:

```graphql
type Mutation {
  postPublishedBy(post_id: ID!, user_id: ID!): Post
  userPostPublished(user_id: ID!, post_id: ID!): User
}
```

If you add a name to your relation with `@relation`:

```graphql
type User @model {
  ...
  postPublished: Post @relation(name: "publish", link: "publish")
}

type Post @model {
  ...
  publishedBy: User @relation(name: "publish", link: "publishBy")
}

...

type Mutation {
  publishPost(post_id: ID!, user_id: ID!): Post
  publishByUser(user_id: ID!, post_id: ID!): User
}
```

##### Generic

Right now the advantage for the Generic one is it'll allow us to Create & Link
in one query.

```graphql
type UserUpdatePayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  link: ID
}

input UserUpdateInput {
  firstname: String
  lastname: String
  postPublished: UserUpdateInputPostPublished
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```

If you try to link to an already linked `One-to-One` relation, it'll fail.

#### Unlink

##### Specialized

Imagine you already have an user but you now want to unlink it:

- Unlink an existing User to an existing Post.

If you do not name your relation with `@relation`:

```graphql
type Mutation {
  postPublishedBy(post_id: ID!, user_id: ID!): Post
  userPostPublished(user_id: ID!, post_id: ID!): User
}
```

If you add a name to your relation with `@relation`:

```graphql
type User @model {
  ...
  postPublished: Post @relation(name: "publish", unlink: "unpublish")
}

type Post @model {
  ...
  publishedBy: User @relation(name: "publish", unlink: "unpublishBy")
}

...

type Mutation {
  unpublishPost(post_id: ID!): Post
  unpublishByUser(user_id: ID!): User
}
```

##### Generic

```graphql
type UserUpdatePayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  link: ID
  """
  You must specify which Post you want to unlink from the user.
  """
  unlink: ID
}

input UserUpdateInput {
  firstname: String
  lastname: String
  postPublished: UserUpdateInputPostPublished
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```

It would be possible to have a unlink which doesn't need any `ID`.
