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
