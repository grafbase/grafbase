# =============================
# === Subgraph sanity check ===
# =============================
# Subgraph expects the right scope when accessing users and accounts.
POST http://localhost:4000/graphql
Content-Type: application/json
```graphql
query {
    user(id: 1) {
        id
    }
    accounts {
        id
    }
}
```

HTTP 200
{"data":{"user":null,"accounts":null},"errors":[{"message":"Insufficient scopes","locations":[{"line":2,"column":5}],"path":["user"]},{"message":"Insufficient scopes","locations":[{"line":5,"column":5}],"path":["accounts"]}]}

# with proper scopes
POST http://localhost:4000/graphql
Content-Type: application/json
Authorization: account,user
```graphql
query {
    user(id: 1) {
        id
    }
    accounts {
        id
    }
}
```

HTTP 200
{"data":{"user":{"id":1},"accounts":[{"id":1},{"id":2},{"id":3},{"id":4}]}}


# =================================
# === Auth service sanity check ===
# =================================
# Grants access to current user and user 1
POST http://localhost:4001/authorized-users
Content-Type: application/json
```json
{
    "current_user_id": 3
}
```

HTTP 200
{"authorized_users":[1,3]}



# ===============
# === Gateway ===
# ===============

# == Authentication ==
# Let's do a first simple request to our GraphQL server as a sanity check.
# The request will fail as our authentication extension expects a `current-user-id` header.
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
    __typename
}
```

HTTP 401
{"errors":[{"message":"Unauthenticated","extensions":{"code":"UNAUTHENTICATED"}}]}

# Now let's try it again with the appropriate header
POST http://localhost:5000/graphql
Content-Type: application/json
current-user-id: 3
```graphql
query {
    __typename
}
```

HTTP 200
{"data":{"__typename":"Query"}}

# It works!


# == Authorization: query ==
# The auth service specifies that we can always access the user with id 1 and our own id.
# For `Query.user(id: X)` we validate the argument before sending any request to the subgraph.
# So with with the current user id 3 we should be able to access the user 1 and 3.
# *1*
POST http://localhost:5000/graphql
Content-Type: application/json
current-user-id: 3
```graphql
query {
    user(id: 1) {
        id
        name
    }
}
```

HTTP 200
{"data":{"user":{"id":1,"name":"Alice"}}}

# *2*
POST http://localhost:5000/graphql
Content-Type: application/json
current-user-id: 3
```graphql
query {
    user(id: 2) {
        id
        name
    }
}
```

HTTP 200
{"data":{"user":null},"errors":[{"message":"Not authorized: cannot access user","locations":[{"line":2,"column":5}],"path":["user"],"extensions":{"code":"UNAUTHORIZED"}}]}

# *3*
POST http://localhost:5000/graphql
Content-Type: application/json
current-user-id: 3
```graphql
query {
    user(id: 3) {
        id
        name
    }
}
```

HTTP 200
{"data":{"user":{"id":3,"name":"Musti"}}}


# == Authorization: response ==
# The auth service specifies that we can always access the user with id 1 and our own id.
# As we cannot guess which accounts will be present, the gateway will automatically inspect the response
# and apply any decisions taken by the extension.
POST http://localhost:5000/graphql
Content-Type: application/json
current-user-id: 3
```graphql
query {
    accounts {
        id
        name
    }
}
```

HTTP 200
{"data":{"accounts":[{"id":1,"name":"Alice's account"},null,{"id":3,"name":"Musti's account"},null]},"errors":[{"message":"Not authorized: cannot access account","locations":[{"line":2,"column":5}],"path":["accounts",1],"extensions":{"code":"UNAUTHORIZED"}},{"message":"Not authorized: cannot access account","locations":[{"line":2,"column":5}],"path":["accounts",3],"extensions":{"code":"UNAUTHORIZED"}}]}

