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
  'User': {
    __typename?: 'User';
    id: string;
    name: string;
    biography?: string;
    linkedInProfile?: string;
    account?: Schema['Account'];
  };
  'Account': {
    __typename?: 'Account';
    id: string;
    email: string;
  };
  'Other': {
    __typename?: 'Other';
    id: string;
  };
  'UserFilter': {
    name_eq: string | null;
  };
  'Query': {
    __typename?: 'Query';
    user?: Schema['User'] | null;
    users?: Array<Schema['User'] | null> | null;
    other?: Schema['Other'] | null;
  };
};

import { ResolverFn } from '@grafbase/sdk'

export type Resolver = {
  'User.linkedInProfile': ResolverFn<Schema['User'], {  }, string>
  'Query.user': ResolverFn<Schema['Query'], { anonymize: boolean | null,  }, Schema['User'] | null>
  'Query.users': ResolverFn<Schema['Query'], { filter: Schema['UserFilter'] | null, take: number,  }, Array<Schema['User'] | null> | null>
  'Query.other': ResolverFn<Schema['Query'], {  }, Schema['Other'] | null>
}

