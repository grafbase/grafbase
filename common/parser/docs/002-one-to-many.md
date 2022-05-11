# One-to-Many | Many-to-One

For instance let's take a Schema representing a Blog where an user can publish
**MANY** post.

```graphql
type User @model {
  id: ID!
  firstname: String!
  lastname: String!
  posts_published: [Post]
}

type Post @model {
  id: ID!
  content: String!
  """
  Here the user is **MANDATORY**.
  """
  published_by: User!
}
``` 

The relation here is represented by the field `pubished_by`. It is what can be
called a `One-to-Many` relationship.

An `User` can only have **MANY** `Post` published, and a `Post` **MUST** be published
by **ONE** `User`.

So what happens when you disconnect an User from the Post?

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

#### Post

For the Post, we have the `published_by` which is generated a little different.

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
  connect: ID
}

input PostInput {
  content: String!
  published_by: PostInputUserPublishedBy!
}

input PostsInput {
  users: [PostInput]
}

type CreatePostPayload {
  post: Post
}

type CreatePostsPayload {
  posts: [Post]
}

type Mutation {
  createPost(input: PostInput): CreatePostPayload
  createPosts(input: PostsInput): CreatePostsPayload
  ...
}
```

#### Connect

##### Specialized

Imagine you already have an user but you now want to connect it, it won't happen
as this relation is mandatory for `Post`, so a `Post` can't exist without being
connected.

So it means the previous connect specialized queries can't exist as a `Post` just
can't exist without being linked.

And it would be the same for any specialized relations ship.

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
  post_published: UserUpdateInputPostPublished
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```

When you connect `User` to `Post` it will fail as a Post is already linked to an
user and a `Post` can only have one `User`.

#### Disconnect

When you want to disconnect a `Post` from a `User` it means we will be removing
the link between a `User` to a `Post` but also the link from the `Post` to the
`User`.

This link is **MANDATORY**, so what would happens?
The data stored would be **NON-COMPLIANT** to the defined schema if we would allow that.
So when the `User` would be fetched from the `Post` we would bubble up an error.

What we could do instead:
  - Erroring out when we disconnect without connecting it again to another `User`
  - Removing the `Post` but it would also means we would have side-effect on other
  entities linked to that `Post`.

**We should decide between**:
 - We allow the data-stored to be **NON-COMPLIANT**.
 - We return an error when you try to have **NON-COMPLIANT** data.
    -> This solution imply that to delete the link between the `User` and the `Post`
    you should update the Post and not the User.


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
  post_published: UserUpdateInputPostPublished
}

type Mutation {
  updateUser(input: UserUpdateInput): UpdateUserPayload
}
```

