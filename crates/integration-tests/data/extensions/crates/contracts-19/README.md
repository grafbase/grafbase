# Tag extension

Build a contract with the `@tag` extension defined as:

```graphql
directive @tag(
  name: String!
) repeatable on FIELD_DEFINITION | INTERFACE | OBJECT | UNION | ARGUMENT_DEFINITION | SCALAR | ENUM | ENUM_VALUE | INPUT_OBJECT | INPUT_FIELD_DEFINITION
```

The contract key is a JSON with two optional keys `includedTags` and `excludedTags`:

```json
{
  "includedTags": ["a"],
  "excludedTags": ["b"]
}
```

If the `includedTags` list is empty, the contract schema includes each type and object/interface field unless it's tagged with an excluded tag.

If the `includedTags` list is non-empty, the contract schema excludes each union type and object/interface field unless it's tagged with an included tag.

- Each object and interface type is included as long as at least one of its fields is included
  (unless the type is explicitly excluded)
- The contract schema excludes a type or field if it's tagged with both an included tag and an excluded tag.

If you enable the option to hide unreachable types, the contract schema excludes each unreachable object, interface, union, input, enum, and scalar unless it's tagged with an included tag.

If a contract defines a list of included `@tag`s, any object or interface type without an included tag is still included in the contract schema if at least one of its fields is included:

```graphql
# This type definition is included because one if its fields is included.
type User {
  id: ID! @tag(name: "includeMe")
}
```

If a contract excludes every field of an object or interface type, the entire type definition is excluded from the contract schema:

```graphql
# This object type is excluded because all its fields are excluded.
type User {
  id: ID! @tag(name: "excludeMe")
}
```

If a contract excludes every object type that's part of a union type, the entire union type definition is excluded from the contract schema:

```graphql
# This union type is excluded because all its possible types are excluded.
union Media = Book | Movie

type Book @tag(name: "excludeMe") {
  title: String!
}

type Movie @tag(name: "excludeMe") {
  title: String!
}
```

A contract cannot exclude any of the following, even if tagged:

- Built-in scalars (Int, Float, etc.)
- Built-in directives (@skip, @include, etc.)
