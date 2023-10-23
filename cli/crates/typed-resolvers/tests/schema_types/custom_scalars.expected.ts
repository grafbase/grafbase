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

/**
 * An IPv4 address
 */
export type NetworkAddress = any;

export type QueryRoot = {
  __typename?: 'QueryRoot';
  address: NetworkAddress | null;
};
