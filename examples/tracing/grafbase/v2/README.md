### Run

    grafbase federated start --listen-address 127.0.0.1:4500 --config gateway-config.toml --federated-schema federated-schema.graphql
    curl -v -X POST localhost:4500/graphql --data '{ "query": "query { instruct(prompt: \"hello\") }" }' -H 'x-api-key: dummy' -H 'Content-Type: application/json'
