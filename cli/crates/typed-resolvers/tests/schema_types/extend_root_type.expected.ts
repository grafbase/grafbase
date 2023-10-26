// This is a generated file. It should not be edited manually.
//
// You can decide to commit this file or add it to your `.gitignore`.
//
// By convention, this module is imported as `@grafbase/generated`. To make this syntax possible,
// add a `paths` entry to your `tsconfig.json`.
//
//  "compilerOptions": {
//    "paths": {
//      "@grafbase/generated": ["./grafbase/generated"]
//    }
//  }

export type Schema = {
  'Query': {
    __typename?: 'Query';
    ping?: string;
  };
  'Mutation': {
    __typename?: 'Mutation';
    pong?: string;
  };
  'Subscription': {
    __typename?: 'Subscription';
    pingPongs: Array<string>;
  };
};

import { ResolverFn } from '@grafbase/sdk'

export type Resolver = {
  'Query.ping': ResolverFn<Schema['Query'], { name: string | null,  }, string>
  'Mutation.pong': ResolverFn<Schema['Mutation'], {  }, string>
}

