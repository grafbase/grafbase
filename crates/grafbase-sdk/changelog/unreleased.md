## Features

- You can now set response headers on `ErrorResponse`. These extra headers will be merged with the response headers issued by the Gateway. (https://github.com/grafbase/grafbase/pull/3236)
- Authentication extensions now have an optional `public_metadata()` method for public metadata endpoints, to implement specs like the [OAuth protected resource metadata RFC](https://datatracker.ietf.org/doc/html/rfc9728). The endpoints are available on the gateway for GET requests at a custom path, and they return a static payload with custom headers. (https://github.com/grafbase/grafbase/pull/3235)
