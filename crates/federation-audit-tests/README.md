### Getting the gateway-audit-repo in place

If you don't already have it, or it's up to date:

```sh
cargo test -p federation-audit-tests --test checkout
```

### Making sure the cached list of tests is up to date

```sh
cargo test -p federation-audit-tests --test cache_freshness
```

### Running tests

1. Run the audit server in the background: `cd gateway-audit-repo && npm start serve`
2. `cargo nextest run -p federation-audit-tests --test audit_tests`

All the usual nextest options should work. If you want more logs we support
`RUST_LOG` to control them.
