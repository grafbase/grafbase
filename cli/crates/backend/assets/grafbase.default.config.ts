// Welcome to Grafbase!
// Instant GraphQL APIs for your data
// https://grafbase.com

import { g, auth, connector, config } from '@grafbase/sdk';

// TODO TIDY COMMENTS
// MongoDB Connector (2 models)

// const mongo = connector.MongoDB('MongoDB', {
//   apiKey: g.env('MONGODB_API_KEY'),
//   url: g.env('MONGODB_API_URL'),
//   dataSource: g.env('MONGODB_DATASOURCE'),
//   database: g.env('MONGODB_DATABASE'),
// });

// const address = g
//   .type('Address', {
//     street: g.string(),
//     city: g.string(),
//     country: g.string(),
//   })
//   .collection('addresses');

// mongo
//   .model('User', {
//     name: g.string(),
//     age: g.int().optional(),
//     address: g.ref(address).optional(),
//     metadata: g.json().optional(),
//   })
//   .collection('users');

// g.datasource(mongo);

// OpenAPI Connector (header forwarding)
// const stripe = connector.OpenAPI('Stripe', {
//   schema: 'https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json',
//   headers: (headers) => {
//     headers.set('Authorization', `Bearer ${g.env('STRIPE_SECRET_KEY')}`);
//   },
//   transforms: schema => {
//     schema.
//   }
// });

// Extended OpenAPI Connector (+ resolver)
// const gravatarRating = g.enum("GravatarRating", ["g", "pg", "r", "x"]);

// g.extend("StripeCustomer", {
//   gravatar: {
//     args: {
//       size: g.int().optional(),
//       defaultImage: g.url().optional(),
//       rating: g.enumRef(gravatarRating).optional(),
//       secure: g.boolean().optional(),
//     },
//     returns: g.url().optional(),
//     resolver: "gravatar",
//   },
// });

// g.datasource(stripe);

// Custom query resolver
g.query('hello', {
  returns: g.string(),
  resolver: 'hello-world',
});

export default config({
  schema: g,
  // Public auth rule
  auth: {
    rules: (rules) => {
      rules.public();
    },
  },
  // Caching config
  cache: {
    rules: [
      {
        types: ['Query'],
        maxAge: 60,
      },
    ],
  },
});
