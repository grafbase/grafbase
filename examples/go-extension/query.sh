#!/bin/bash

# Check if authorization header is provided
if [ -z "$1" ]; then
  echo "Error: Authorization header value is required"
  echo "Usage: $0 <authorization_header_value>"
  exit 1
fi

# Store the authorization header value
AUTH_VALUE="$1"

# Send GraphQL request using curl
curl -v \
  -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: $AUTH_VALUE" \
  -d '{"query":"query GetGreeting { greet(name: \"Alice\") }"}' \
  http://localhost:5000/graphql
