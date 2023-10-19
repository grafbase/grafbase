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

export type TreeNode = {
  __typename?: 'TreeNode';
  value: number;
  children?: Array<TreeNode | null> | null;
};

export type TreeNodeInput = {
  value: number;
  children: Array<TreeNodeInput | null> | null;
};

export type Query = {
  __typename?: 'Query';
  getTree?: TreeNode | null;
};

export type Mutation = {
  __typename?: 'Mutation';
  createTree?: TreeNode | null;
};
