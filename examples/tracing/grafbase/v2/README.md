### Run

    grafbase start --search --config gateway-config.toml
    grafbase introspect http://localhost:4000/graphql -H 'x-api-key: dummy' | grafbase publish --name v1 --url http://localhost:4000/graphql --dev-api-port 4001 --dev -H 'x-api-key: dummy'
