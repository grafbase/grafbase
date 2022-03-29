oha https://static_schema.griffon.workers.dev/graphql -n 5000 -m POST -d '{"query":"query {\n\tproductById(id: 1) {\n\t\t__typename\n\t\tid\n\t}\n}"}' -T application/json
