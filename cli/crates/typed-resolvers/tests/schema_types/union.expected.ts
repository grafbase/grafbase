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
  'Dog': {
    __typename?: 'Dog';
    id: string;
    barkVolume: number | null;
  };
  'Cat': {
    __typename?: 'Cat';
    id: string;
    meowVolume: number | null;
  };
  'Pet': | Schema['Cat'] | Schema['Dog'];
};
