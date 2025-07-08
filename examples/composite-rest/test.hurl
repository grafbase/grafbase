POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp"},{"summary":"Follow-up call with TechStart Inc"},{"summary":"Product demo for Enterprise Solutions"}]}}

# With deals through derive & lookup
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals {
      name
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Acme Corp Support Package"}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan"}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Enterprise Solutions Premium"}]}]}}

POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals {
      name
      orders {
        createdAt
        lineItems {
          product {
            name
          }
        }
      }
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Acme Corp Support Package","orders":[{"createdAt":"2024-01-15T10:30:00Z","lineItems":[{"product":{"name":"Support Package Gold"}},{"product":{"name":"Training Package"}}]}]}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan","orders":[{"createdAt":"2024-01-01T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Enterprise Solutions Premium","orders":[]}]}]}}

# With deals through require
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals: deals2 {
      name
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Acme Corp Support Package"}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan"}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License"},{"name":"Enterprise Solutions Premium"}]}]}}

POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query Calls {
  calls {
    summary
    deals: deals2 {
      name
      orders {
        createdAt
        lineItems {
          product {
            name
          }
        }
      }
    }
  }
}
```

HTTP 200
{"data":{"calls":[{"summary":"Initial sales call with Acme Corp","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Acme Corp Support Package","orders":[{"createdAt":"2024-01-15T10:30:00Z","lineItems":[{"product":{"name":"Support Package Gold"}},{"product":{"name":"Training Package"}}]}]}]},{"summary":"Follow-up call with TechStart Inc","deals":[{"name":"TechStart Inc Starter Plan","orders":[{"createdAt":"2024-01-01T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]}]},{"summary":"Product demo for Enterprise Solutions","deals":[{"name":"Acme Corp Enterprise License","orders":[{"createdAt":"2023-12-26T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}},{"product":{"name":"Integration Module"}}]},{"createdAt":"2024-01-05T10:30:00Z","lineItems":[{"product":{"name":"CRM Pro License"}}]}]},{"name":"Enterprise Solutions Premium","orders":[]}]}]}}

# Get a specific commune
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query GetCommune {
  commune(code: "75056") {
    code
    nom
    region {
      code
      nom
    }
    departement {
      code
      nom
    }
  }
}
```

HTTP 200
{"data":{"commune":{"code":"75056","nom":"Paris","region":{"code":"11","nom":"Île-de-France"},"departement":{"code":"75","nom":"Paris"}}}}
