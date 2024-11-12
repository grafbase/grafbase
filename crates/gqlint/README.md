# `gqlint`

A GraphQL SDL linting CLI

## Install

```sh
cargo install gqlint
```

## Usage

```sh
$ gqlint schema.graphql

⚠️ [Warning]: directive 'WithDeprecatedArgs' should be renamed to 'withDeprecatedArgs'
⚠️ [Warning]: argument 'ARG' on directive 'WithDeprecatedArgs' should be renamed to 'arg'
⚠️ [Warning]: enum 'Enum_lowercase' should be renamed to 'EnumLowercase'
⚠️ [Warning]: enum 'Enum_lowercase' has a forbidden prefix: 'Enum'
⚠️ [Warning]: usage of directive 'deprecated' on enum 'Enum_lowercase' does not populate the 'reason' argument
⚠️ [Warning]: value 'an_enum_member' on enum 'Enum_lowercase' should be renamed to 'AN_ENUM_MEMBER'
⚠️ [Warning]: usage of directive 'deprecated' on enum value 'an_enum_member' on enum 'Enum_lowercase' does not populate the 'reason' argument
⚠️ [Warning]: enum 'lowercase_Enum' should be renamed to 'LowercaseEnum'
⚠️ [Warning]: enum 'lowercase_Enum' has a forbidden suffix: 'Enum'
⚠️ [Warning]: value 'an_enum_member' on enum 'lowercase_Enum' should be renamed to 'AN_ENUM_MEMBER'
⚠️ [Warning]: usage of directive 'deprecated' on enum value 'an_enum_member' on enum 'lowercase_Enum' does not populate the 'reason' argument
⚠️ [Warning]: field 'getHello' on type 'Query' has a forbidden prefix: 'get'
⚠️ [Warning]: field 'queryHello' on type 'Query' has a forbidden prefix: 'query'
⚠️ [Warning]: field 'listHello' on type 'Query' has a forbidden prefix: 'list'
⚠️ [Warning]: field 'helloQuery' on type 'Query' has a forbidden suffix: 'Query'
⚠️ [Warning]: field 'putHello' on type 'Mutation' has a forbidden prefix: 'put'
⚠️ [Warning]: field 'mutationHello' on type 'Mutation' has a forbidden prefix: 'mutation'
⚠️ [Warning]: field 'postHello' on type 'Mutation' has a forbidden prefix: 'post'
⚠️ [Warning]: field 'patchHello' on type 'Mutation' has a forbidden prefix: 'patch'
⚠️ [Warning]: field 'helloMutation' on type 'Mutation' has a forbidden suffix: 'Mutation'
⚠️ [Warning]: field 'subscriptionHello' on type 'Subscription' has a forbidden prefix: 'subscription'
⚠️ [Warning]: field 'helloSubscription' on type 'Subscription' has a forbidden suffix: 'Subscription'
⚠️ [Warning]: type 'TypeTest' has a forbidden prefix: 'Type'
⚠️ [Warning]: usage of directive 'deprecated' on field 'name' on type 'TypeTest' does not populate the 'reason' argument
⚠️ [Warning]: type 'TestType' has a forbidden suffix: 'Type'
⚠️ [Warning]: type 'other' should be renamed to 'Other'
⚠️ [Warning]: usage of directive 'deprecated' on scalar 'CustomScalar' does not populate the 'reason' argument
⚠️ [Warning]: union 'UnionTest' has a forbidden prefix: 'Union'
⚠️ [Warning]: usage of directive 'deprecated' on union 'UnionTest' does not populate the 'reason' argument
⚠️ [Warning]: union 'TestUnion' has a forbidden suffix: 'Union'
⚠️ [Warning]: interface 'GameInterface' has a forbidden suffix: 'Interface'
⚠️ [Warning]: usage of directive 'deprecated' on field 'publisher' on interface 'GameInterface' does not populate the 'reason' argument
⚠️ [Warning]: interface 'InterfaceGame' has a forbidden prefix: 'Interface'
⚠️ [Warning]: usage of directive 'deprecated' on interface 'InterfaceGame' does not populate the 'reason' argument
⚠️ [Warning]: usage of directive 'deprecated' on input 'TEST' does not populate the 'reason' argument
⚠️ [Warning]: input value 'OTHER' on input 'TEST' should be renamed to 'other'
⚠️ [Warning]: usage of directive 'deprecated' on input value 'OTHER' on input 'TEST' does not populate the 'reason' argument
⚠️ [Warning]: type 'hello' should be renamed to 'Hello'
⚠️ [Warning]: usage of directive 'deprecated' on type 'hello' does not populate the 'reason' argument
⚠️ [Warning]: field 'Test' on type 'hello' should be renamed to 'test'
⚠️ [Warning]: argument 'NAME' on field 'Test' on type 'hello' should be renamed to 'name'
⚠️ [Warning]: type 'hello' should be renamed to 'Hello'
⚠️ [Warning]: field 'GOODBYE' on type 'hello' should be renamed to 'goodbye'
```

## Rules

See [`graphql-lint`](https://crates.io/crates/graphql-lint)
