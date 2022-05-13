# One-to-Many | Many-to-One

For instance let's take a Schema representing a Blog where an user can publish
**MANY** post.

```graphql
type User @model {
  id: ID!
  firstname: String!
  lastname: String!
  postsPublished: [Post]
}

type Post @model {
  id: ID!
  content: String!
  """
  Here the user is **MANDATORY**.
  """
  publishedBy: User!
}
``` 

The relation here is represented by the field `pubished_by`. It is what can be
called a `One-to-Many` relationship.

An `User` can only have **MANY** `Post` published, and a `Post` **MUST** be published
by **ONE** `User`.

So what happens when you unlink an User from the Post?

## Generated Queries

### Create

You defined your schema, and now you want to be able to:

- Create a new User without any Post attached to it.
- Create a new User and create **MULTIPLE** Post attached to it.
- Create a new User and create **MULTIPLE** Post attached to it and also **MULTIPLE** links.

**Requirement**:

> **If the same post is linked multiple times**:
> Then we'll only link the post **ONCE**.

We'll split those two cases because it's interesting as we have a mandatory field.

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
  postsPublished: [UserInputPostPublished]
}

input UserBulkInput {
  users: [UserInput]
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

#### Post

For the Post, we have the `publishedBy` which is generated a little different.

```graphql
"""
You can choose between creating a new Post, or linking this user to an existing,
non-linked post.
"""

input PostInputUserPublishedByUser {
  firstname: String!
  lastname: String!
}

input PostInputUserPublishedBy {
  create: PostInputUserPublishedByUser 
  link: ID
}

input PostInput {
  content: String!
  publishedBy: PostInputUserPublishedBy!
}

input PostBulkInput {
  users: [PostInput]
}

type PostCreatePayload {
  post: Post
}

type PostBulkCreatePayload {
  posts: [Post]
}

type Mutation {
  postCreate(input: PostInput): PostCreatePayload
  postCreateBulk(input: PostBulkInput): PostBulkCreatePayload
  ...
}
```

#### Link

##### Specialized

Imagine you already have an user but you now want to link it, it won't happen
as this relation is mandatory for `Post`, so a `Post` can't exist without being
linked.

So it means the previous link specialized queries can't exist as a `Post` just
can't exist without being linked.

And it would be the same for any specialized relations ship.

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
  postsPublished: [UserUpdateInputPostPublished]
}

input UserUpdateInputPostPublished {
  create: UserInputPostPublishedPost
  link: ID
}

input UserUpdateInput {
  firstname: String
  lastname: String
  postsPublished: UserUpdateInputPostPublished
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```

When you link `User` to `Post` it will fail as a Post is already linked to an
user and a `Post` can only have one `User`.

#### Unlink

When you want to unlink a `Post` from a `User` it means we will be removing
the link between a `User` to a `Post` but also the link from the `Post` to the
`User`.

This link is **MANDATORY**, so what would happens?
The data stored would be **NON-COMPLIANT** to the defined schema if we would allow that.
So when the `User` would be fetched from the `Post` we would bubble up an error.

What we could do instead:
  - Erroring out when we unlink without linking it again to another `User`
  - Removing the `Post` but it would also means we would have side-effect on other
  entities linked to that `Post`.

**We should decide between**:
 - We allow the data-stored to be **NON-COMPLIANT**.
 - We return an error when you try to have **NON-COMPLIANT** data.
    -> This solution imply that to delete the link between the `User` and the `Post`
    you should update the Post and not the User.


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
  postsPublished: [UserUpdateInputPostPublished]
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
  postsPublished: UserUpdateInputPostPublished
}

type Mutation {
  userUpdate(input: UserUpdateInput): UserUpdatePayload
}
```

