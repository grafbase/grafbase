// Welcome to Grafbase!
// Instant GraphQL APIs for your data
// https://grafbase.com

import { g, auth, connector, config } from '@grafbase/sdk';

// TODO
// MongoDB Connector (2 models)
// OpenAPI Connector (header forwarding)
// Extended OpenAPI Connector (+ resolver)
// Custom query resolver
// Caching config
// Public auth rule

export default config({
  schema: g,
});
