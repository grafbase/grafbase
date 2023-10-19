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

export type User = {
  __typename?: 'User';
  id: string;
  name: string;
  account?: Account;
};

export type Account = {
  __typename?: 'Account';
  id: string;
  email: string;
};

export type Other = {
  __typename?: 'Other';
  id: string;
};

export type UserFilter = {
  name_eq: string | null;
};

export type Query = {
  __typename?: 'Query';
  user?: User | null;
  users?: Array<User | null> | null;
  other?: Other | null;
};

import * as sdk from '@grafbase/sdk'
type ResolverFn<Parent, Args, Return> = ((parent: Parent, args: Args, context: sdk.Context, info: sdk.Info) => Return) |
    ((parent: Parent, args: Args, context: sdk.Context, pageInfo: sdk.Info) => Promise<Return>)

export type Resolver = {
  'Query.user': ResolverFn<Query, { anonymize: boolean | null,  }, User | null>
  'Query.users': ResolverFn<Query, { filter: UserFilter | null, take: number,  }, Array<User | null> | null>
  'Query.other': ResolverFn<Query, {  }, Other | null>
}

