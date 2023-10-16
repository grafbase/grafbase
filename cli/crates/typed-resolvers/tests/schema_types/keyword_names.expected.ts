export type holycow = type | _interface | _object | union;

/**
 * GraphQL field names can be anything.
 */
export type Cursed = {
  __typename?: 'Cursed';
  self: boolean | null;
  this: string | null;
  let: number | null;
  type: number | null;
  number: number | null;
  super: boolean | null;
  const: boolean | null;
  /**
   * lol
   */
  async: boolean | null;
  _: boolean | null;
};

/**
 * Look, this enum is called undefined!
 */
export enum _undefined {
  void,
  string,
}

export type type = {
  __typename?: 'type';
  type?: type;
  interface?: _interface;
};

export type _object = {
  __typename?: 'object';
  id: string;
};

export type union = {
  id: string;
};

export type _interface = {
  __typename?: 'interface';
  type?: type;
  interface?: _interface | null;
};

export type schema = {
  id: string;
};

export type query = {
  fragment?: type | null;
};
