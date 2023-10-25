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
  'Query': {
    __typename?: 'Query';
    episode?: Schema['Episode'] | null;
    character?: Schema['Character'] | null;
  };
  'Mutation': {
    __typename?: 'Mutation';
    createEpisode?: Schema['Episode'];
  };
  'Episode': {
    __typename?: 'Episode';
    id: string;
    title: string;
    season: number;
    episodeNumber: number;
    description: string | null;
    characters?: Array<Schema['Character']>;
  };
  'Character': {
    __typename?: 'Character';
    id: string;
    name: string;
    occupation: string | null;
    episodes?: Array<Schema['Episode']>;
    friends?: Array<Schema['Character']> | null;
  };
  'CreateEpisodeInput': {
    title: string;
    season: number;
    episodeNumber: number;
    description: string | null;
    characters: Array<string> | null;
  };
};
