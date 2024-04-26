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
  'Human': {
    __typename?: 'Human';
    id: string;
    firstName: string;
    lastName: string;
  };
  'Dog': {
    __typename?: 'Dog';
    id: string;
    name: string;
  };
  'Animal': | Schema['Dog'];
  'LivingThing': | Schema['Dog'] | Schema['Human'];
};
