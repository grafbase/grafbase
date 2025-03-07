# Snowflake extension

This Grafbase gateway extension allows you to execute SQL statements against a Snowflake database over the [Snowflake SQL API](https://docs.snowflake.com/en/developer-guide/sql-api/intro).

## Example usage

The `customLimit` query demonstrates how to use the `@snowflakeQuery` directive to execute a SQL statement against a Snowflake database. Note that the `bindings` argument is a `JsonTemplate` custom scalar that can use JSON literals but also refer to the contents of field arguments.

```graphql
extend schema
  @link(url: "https://specs.apollo.dev/federation/v2.7")
  @link(url: "https://extensions.grafbase.com/extensions/snowflake/0.1.0", import: ["@snowflakeQuery"])

scalar JSON

type Query {
    customLimit(params: [JSON!]!): String! @snowflakeQuery(sql: "SELECT * FROM my_table LIMIT ?", bindings: "{{ args.params }}")
}
```

## Authentication

This extension supports [key-pair authentication](https://docs.snowflake.com/en/developer-guide/sql-api/authenticating#using-key-pair-authentication).

See the [key-pair authentication documentation](https://docs.snowflake.com/en/user-guide/key-pair-auth) for how to generate a key pair, and the following section to see how to pass the key pair to the extension.

The key pair must be [associated to the configured user for the extension](https://docs.snowflake.com/en/user-guide/key-pair-auth#assign-the-public-key-to-a-snowflake-user).

## Configuration

The following example details all existing options:

```toml
[extensions.snowflake.config]
# Required. The account identifier.
account = "cywxwdp-qv84952"
# Required. The user name.
user = "username"
# Optional role name to assume. Will default to the user's default role if not specified.
role = "custom-role"

# The following options are included in the body of statements requests (https://docs.snowflake.com/en/developer-guide/sql-api/reference#label-sql-api-reference-request-headers).
# They are optional.
warehouse = "COMPUTE_WH"
database = "SNOWFLAKE_SAMPLE_DATA"
schema = "TPCH_SF1"

[extensions.snowflake.config.authentication.key_pair_jwt]
public_key = "{{ env.SNOWFLAKE_PUBLIC_KEY }}"
private_key = "{{ env.SNOWFLAKE_PRIVATE_KEY }}"
```

## Limitations

This extension is a proof of concept, it is expected to change as use cases emerge and it is made production ready.

- `@snowflakeQuery` resolvers could return an array of rows with named columns. The Snowflake SQL API response contains metadata about the columns, so this should be relatively straightforward to implement.
- Statements are not batched. The Snowflake SQL API does support multi-statements, returning multiple handles to the results, to be fetched separately ([docs](https://docs.snowflake.com/en/developer-guide/sql-api/submitting-multiple-statements)). Support for this is not implemented in this extension, but it could be.
- OAuth authentication support is not implemented yet.
- JWTs expire after one hour. They should be refreshed.
