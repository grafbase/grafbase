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
  'TreeNode': {
    __typename?: 'TreeNode';
    value: number;
    children?: Array<Schema['TreeNode'] | null> | null;
  };
  'TreeNodeInput': {
    value: number;
    children: Array<Schema['TreeNodeInput'] | null> | null;
  };
  'Query': {
    __typename?: 'Query';
    getTree?: Schema['TreeNode'] | null;
  };
  'Mutation': {
    __typename?: 'Mutation';
    createTree?: Schema['TreeNode'] | null;
  };
};
