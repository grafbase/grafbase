# Unreleased

## Features

### Trusted documents incremental adoption

Until now, trusted documents would be either disabled, or fully enforced. However in practice, organizations follow a more incremental migration path. The following options are now available in `grafbase.toml`:

```toml
[trusted_documents]
enabled = true # default: false
enforced = true # default: false
bypass_header_name = "my-header-name" # default null
bypass_header_value = "my-secret-is-{{ env.SECRET_HEADER_VALUE }}" # default null
document_id_unknown_log_level = "error" # default: info
document_id_and_query_mismatch_log_level = "off" # default: info
inline_document_unknown_log_level = "warn" # default: info
```

The `bypass_header_name` and `bypass_header_value` settings are unchanged.

The `enabled` setting is now much weaker. When trusted documents are only `enabled`, the gateway will not enforce trusted documents, but will still fetch and cache them. This is useful for organizations that want to gradually adopt trusted documents.

The `enforce` setting matches the previous behavior: when `true`, the gateway will enforce trusted documents. Additionally, we now allow trusted document requests with only an inline document, without document id, as long as the query is a trusted document. This is useful for organizations that want to enforce trusted documents, but have not yet migrated all their clients to send the document id.

The three new log options allow monitoring proper usage of trusted documents before you enforce them:

- `document_id_unknown_log_level`: logged when a document id is present in the request, but the corresponding trusted document cannot be found.
- `document_id_and_query_mismatch_log_level`: logged when both a query document and a document id are sent, when the retrieved trusted document does not match the inline query document.
- `inline_document_unknown_log_level`: logged when an inline document (a string in the `"query"` key of a the request) is sent, but no trusted document is found with the id matching the sha256 hash of the inline document.

They can all take `off`, `error`, `warn`, `info`, or `debug` as values. The default is `info`.
