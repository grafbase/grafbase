### Run

    grafbase start
    curl -v -X POST localhost:4000/graphql --data '{ "query": "query { instruct(prompt: \"hello\") }" }' -H 'x-api-key: dummy'