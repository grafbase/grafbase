# Many-to-Many

For instance let's take a Schema representing a Blog where an user can publish
**MANY** post and where a Post can be written by **MULTIPLE** Users.

```graphql
type User @model {
  id: ID!
  firstname: String!
  lastname: String!
  posts_published: [Post!]
}

type Post @model {
  id: ID!
  content: String!
  published_by: [User!]!
}
``` 

The relation here is represented by the field `pubished_by`. It is what can be
called a `Many-to-Many` relationship.

An `User` can only have **MANY** `Post` published, and a `Post` **CAN** be published
by **MULTIPLE** `User`.

## Generated Queries

### Create

You defined your schema, and now you want to be able to:

- Create a new User without any Post attached to it.
- Create a new User and create **MULTIPLE** Post attached to it.
- Create a new User and create **MULTIPLE** Post attached to it and also **MULTIPLE** links.

**Requirement**:

> **If the same post is linked multiple times**:
> Then we'll only link the post **ONCE**.

#### User

```graphql
"""
You can choose between creating a new Post, or linking this user to an existing,
non-linked post.
"""
input UserInputPostPublished {
  create: UserInputPostPublishedPost
  connect: ID
}

input UserInputPostPublishedPost {
  content: String!
}

input UserInput {
  firstname: String!
  lastname: String!
  post_published: [UserInputPostPublished]
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

##### Generic

```graphql
type UpdateUserPayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInput {
  firstname: String!
  lastname: String!
  post_published: [UserUpdateInputPostPublished]
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  connect: ID
}

input UserUpdateInput {
  firstname: String
  lastname: String
  post_published: [UserUpdateInputPostPublished]
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```

#### Disconnect

When you want to disconnect a `Post` from a `User` it means we will be removing
the link between a `User` to a `Post` but also the link from the `Post` to the
`User`.

##### Generic

```graphql
type UpdateUserPayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInput {
  firstname: String!
  lastname: String!
  post_published: [UserUpdateInputPostPublished]
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
  post_published: [UserUpdateInputPostPublished]
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```
