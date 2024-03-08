### Run

    grafbase start --config gateway-config.toml --listen-address 127.0.0.1:4500
    curl -v -X POST localhost:4000/graphql --data '{ "query": "query { instruct(prompt: \"hello\") }" }' -H 'x-api-key: dummy'
    curl -v -X POST localhost:4500/graphql --data '{ "query": "query { instruct(prompt: \"hello\") }" }' -H 'x-api-key: dummy' -H 'Content-Type: application/json'