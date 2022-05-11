# Relations

A Relation is a way to link multiple `modelized` entities together.

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
  post_published: Post
}

type Post @model {
  id: ID!
  content: String!
  published_by: User
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

TODO: Error modelization?
TODO: Check with more complex relations.

```graphql
"""
You can choose between creating a new Post, or linking this user to an existing,
non-linked post.
"""
input UserInputPostPublished {
  create: UserInputPostPublishedPost
  connect: ID
}

"""
If you choose to create a new Post when you create a new User, as this is an
`One-to-One` relation, we cannot let you create a User inside the Post for the
User.
"""
input UserInputPostPublishedPost {
  content: String!
}

input UserInput {
  firstname: String!
  lastname: String!
  post_published: UserInputPostPublished
}

input UsersInput {
  users: [UsersInput]
}

type CreateUserPayload {
  user: User
}

type CreateUsersPayload {
  users: [User]
}

type Mutation {
  createUser(input: UserInput): CreateUserPayload
  createUsers(input: UsersInput): CreateUsersPayload
  ...
}
```

#### Connect

##### Specialized

Imagine you already have an user but you now want to connect it:

- Connect an existing User to an existing Post.

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
  post_published: Post @relation(name: "publish", connect: "publish")
}

type Post @model {
  ...
  published_by: User @relation(name: "publish", connect: "publishBy")
}

...

type Mutation {
  publishPost(post_id: ID!, user_id: ID!): Post
  publishByUser(user_id: ID!, post_id: ID!): User
}
```

##### Generic

Right now the advantage for the Generic one is it'll allow us to Create & Connect
in one query.

```graphql
type UpdateUserPayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  connect: ID
}


input UserUpdateInput {
  firstname: String
  lastname: String
  post_published: UserUpdateInputPostPublished
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```

If you try to connect to an already connected `One-to-One` relation, it'll fail.

#### Disconnect

##### Specialized

Imagine you already have an user but you now want to disconnect it:

- Disconnect an existing User to an existing Post.

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
  post_published: Post @relation(name: "publish", disconnect: "unpublish")
}

type Post @model {
  ...
  published_by: User @relation(name: "publish", disconnect: "unpublishBy")
}

...

type Mutation {
  unpublishPost(post_id: ID!): Post
  unpublishByUser(user_id: ID!): User
}
```

##### Generic

```graphql
type UpdateUserPayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  connect: ID
  """
  You must specify which Post you want to disconnect from the user.
  """
  disconnect: ID
}


input UserUpdateInput {
  firstname: String
  lastname: String
  post_published: UserUpdateInputPostPublished
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```

It would be possible to have a disconnect which doesn't need any `ID`.
