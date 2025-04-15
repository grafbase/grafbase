# =============================
# ===== Sanity check ==========
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
mutation {
  routeguide_RouteGuide_GetFeature(input:  {
     latitude: 409146138
     longitude: -746188906
  }) {
    location {
      latitude
      longitude
    }
    name
  }
}
```

HTTP 200
{"data":{"routeguide_RouteGuide_GetFeature":{"location":{"latitude":409146138,"longitude":-746188906},"name":"Berkshire Valley Management Area Trail, Jefferson, NJ, USA"}}}
