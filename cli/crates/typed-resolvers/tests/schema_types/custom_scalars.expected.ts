/**
 * An IPv4 address
 */
export type NetworkAddress = any;

export type QueryRoot = {
  __typename?: 'QueryRoot';
  address: NetworkAddress | null;
};
