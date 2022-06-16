# GraphQL Queries and Mutations

This document describes how to configure your environment 
to be able to run the code examples in the various language sub-directories.
It also includes templates of GraphQL queries and mutations that use the 
[blog](https://github.com/grafbase/grafbase/blob/main/templates/blog/grafbase/schema.graphql) template schema to:

* Create a user
* List the users
* Delete a user
* Create a blog post
* List the blog posts
* Delete a blog post
* Create a blog post comment
* List the blob post comments
* Delete a blog post comment

## Configuring your environment

Before any query or mutation works, you must set the following environment variables:

* API_KEY, which is your API key
* ENDPOINT, which is your project's endpoint

How you set an environment variable depends upon your operating system.
On Linux, macOS, or Unix:

```sh
export API_KEY=YOUR-API-KEY
export ENDPOINT=YOUR-ENDPOINT
```

On Microsoft Windows:

```sh
set API_KEY=YOUR-API-KEY
set ENDPOINT=YOUR-ENDPOINT
```

You can get the API keys for your project's branch by selecting the
**API Keys** tab of the project branch's **Settings** section of the dashboard.
  
You can get the endpoint for a project branch by selecting the 
**Branches** tab in the project's dashboard.

## Queries and Mutations

Save the following queries and mutations as
*FILENAME.json* so you can call them from the main program.

### Creating a user

The **create-user.json** file contains the following mutation that creates a user.
You must replace *USER-EMAIL* with the email address of the user
and *USER-NAME* with the name of the user.
It returns the ID of the new user.

```json
{"query":"mutation { userCreate(input: {email: \"USER-EMAIL\", name: \"USER-NAME\"}) { user { id } } }"}
```

### Listing the users

The **list-user.json** file contains the following query that lists the IDs of the first 10 users.

```json
{"query":"{ userCollection(first: 10) { edges { node { id } } } }"}
```

### Deleting a user

The **delete-user.json** file contains the following mutation that deletes a user.
You must replace *USER-ID* with the ID of a user.
It returns the ID of the deleted user.

```json
{"query":"mutation { userDelete(id: \"USER-ID\") { deletedId }}"}
```

### Creating a post

The **create-post.json** file contains the following mutation that creates a post.
You must replace *USER-ID* with the ID of a user,
*TITLE* with the title of the post,
and *CONTENT* with the contents of the post.
It returns the ID of the new post.

```json
{"query":"mutation { postCreate(input: { title: \"TITLE\", content: \"CONTENT\", user: {link: \"USER-ID\"} }) { post { id } } }"}
```

### Listing the posts

The **list-posts.json** file contains the following query that lists the IDs of first 10 posts.

```json
{"query":"{ postCollection(first: 10) { edges { node { id } } } }"}
```

### Deleting a post

The **create-user.json** file contains the following mutation that deletes a post.
You must replace *POST-ID* with the ID of a post.
It returns the ID of the deleted post.

```json
{"query":"mutation { postDelete(id: \"POST-ID\") { deletedId }}"}
```

### Creating a comment from a user on a post

The **create-comment.json** file contains the following mutation that creates a comment from a user about a post.
You must replace *USER-ID* with the ID of a user
*POST-ID* with the ID of a post,
and *CONTENT* with the contents of the comment.
It returns the ID of the new comment.

```json
{"query":"mutation { commentCreate(input: { post: {link: \"POST-ID\"}, content: \"CONTENT\", user: {link: \"USER-ID\"} }) { comment { id } } }"}
```

### Listing the comments

The **list-comment.json** file contains the following query that lists the IDs of the first 10 comments.

```json
{"query":"{ commentCollection(first: 10) { edges { node { id } } } }"}
```
### Deleting a comment

The **create-user.json** file contains the following mutation that deletes a comment.
You must replace *COMMENT-ID* with the ID of a comment.
It returns the ID of the deleted comment.

```json
{"query":"mutation { commentDelete(id: \"COMMENT-ID\") { deletedId }}"}
```

### Pagination in listings

Whenever you list any collection, you must supply a pagination direction,
either **first** or **last**, with an integer argument from 1 to 99.

But what happens if there are more collection items than you've specified?
We provide a mechanism to paginate the remaining items in the collection.

For example, if there are more than 2 users,
the following command:

```json
{
  userCollection(first: 2) {
    edges {
      node {
        id
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

Returns something like the following,
where *USER2-ID* is the ID of the last user shown:

```json
{
  "data": {
    "userCollection": {
      "edges": [
        {
          "node": {
            "id": "USER1-ID"
          }
        },
        {
          "node": {
            "id": "USER2-ID"
          }
        }
      ],
      "pageInfo": {
        "hasNextPage": true,
        "endCursor": "USER2-ID"
      }
    }
  }
}
```

To retrieve the next two users:

```json
{
  userCollection(first: 2, after: "USER1-ID") {
    edges {
      node {
        id
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

Until the value returned in **hasNextPage** is **false**.

Note that the collection is similar to a stack, 
in that the first user shown is the last user added to the collection.

You can retrieve collection items in the order in which they were added to the collection
using **last**, **hasPreviousPage**, and **startCursor**.