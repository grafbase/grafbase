# Many-to-Many

For instance let's take a Schema representing a Blog where an user can publish
**MANY** post and where a Post can be written by **MULTIPLE** Users.

```graphql
type User @model {
  id: ID!
  firstname: String!
  lastname: String!
  postsPublished: [Post!]
}

type Post @model {
  id: ID!
  content: String!
  publishedBy: [User!]!
}
``` 

The relation here is represented by `publishedBy` can be
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
  link: ID
}

input UserInputPostPublishedPost {
  content: String!
}

input UserInput {
  firstname: String!
  lastname: String!
  postPublished: [UserInputPostPublished]
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
  userCreate(input: UserInput): UserCreatePayload
  userCreateBulk(input: UserBulkInput): UserBulkCreatePayload
  ...
}
```

#### Link

##### Specialized

##### Generic

```graphql
type UserUpdatePayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInput {
  firstname: String!
  lastname: String!
  postPublished: [UserUpdateInputPostPublished]
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  link: ID
}

input UserUpdateInput {
  firstname: String
  lastname: String
  postPublished: [UserUpdateInputPostPublished]
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```

#### Unlink

When you want to unlink a `Post` from a `User` it means we will be removing
the link between a `User` to a `Post` but also the link from the `Post` to the
`User`.

##### Generic

```graphql
type UserUpdatePayload {
  user: User
}

input UserInputPostPublishedPost {
  content: String!
}

input UserUpdateInput {
  firstname: String!
  lastname: String!
  postPublished: [UserUpdateInputPostPublished]
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
  postPublished: [UserUpdateInputPostPublished]
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```
