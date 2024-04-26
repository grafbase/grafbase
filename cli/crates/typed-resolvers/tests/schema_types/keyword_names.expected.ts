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
  'holycow': | Schema['type'] | Schema['interface'] | Schema['object'];
  /**
   * GraphQL field names can be anything.
   */
  'Cursed': {
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
  'undefined': | 'void'| 'string';
  'type': {
    __typename?: 'type';
    type?: Schema['type'];
    interface?: Schema['interface'];
  };
  'object': {
    __typename?: 'object';
    id: string;
  };
  'interface': {
    __typename?: 'interface';
    type?: Schema['type'];
    interface?: Schema['interface'] | null;
  };
  'query': | Schema['interface'];
};
