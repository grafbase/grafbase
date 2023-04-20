# Welcome to Grafbase!
# Define your data models, integrate auth, permission rules, custom resolvers, search, and more with Grafbase.

# Integrate Auth
# https://grafbase.com/docs/auth
#
# schema @auth(providers: [{ type: oidc, issuer: "{{ env.ISSUER_URL }}" }], rules: [{ allow: private }]) {
#   query: Query
# }

# Define Data Models
# https://grafbase.com/docs/database
type Post @model @search {
  title: String!
  slug: String! @unique
  content: String
  publishedAt: DateTime
  comments: [Comment]
  likes: Int @default(value: 0)
  tags: [String] @length(max: 5)
  author: User
}

type Comment @model @search {
  post: Post!
  body: String!
  likes: Int @default(value: 0)
  author: User
}

type User @model {
  name: String!
  email: Email
  posts: [Post]
  comments: [Comment]

  # Extend models with resolvers
  # https://grafbase.com/docs/edge-gateway/resolvers
  # gravatar: URL @resolver(name: "user/gravatar")
}

# Start your backend
# https://grafbase.com/docs/cli
# npx grafbase dev
