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
    episodesBySeason?: Array<Schema['Episode']>;
    character?: Schema['Character'] | null;
    searchCharacters?: Array<Schema['Character']>;
    locations?: Array<Schema['Location']>;
  };
  'Mutation': {
    __typename?: 'Mutation';
    createEpisode?: Schema['Episode'];
    updateEpisode?: Schema['Episode'] | null;
  };
  'CreateEpisodeInput': {
    title: string;
    season: number;
    episodeNumber: number;
    description: string | null;
    characters: Array<string>;
  };
  'UpdateEpisodeInput': {
    title: string | null;
    season: number | null;
    episodeNumber: number | null;
    description: string | null;
    characters: Array<string> | null;
  };
  'Episode': {
    __typename?: 'Episode';
    id: string;
    title: string;
    season: number;
    episodeNumber: number;
    description: string | null;
    characters?: Array<Schema['Character']>;
    nestedTrivia?: Array<Array<Array<Schema['TriviaItem']> | null> | null> | null;
  };
  'Character': {
    __typename?: 'Character';
    id: string;
    name: string;
    occupation: string | null;
    episodes?: Array<Schema['Episode']>;
    friends?: Array<Schema['Character']> | null;
    favoriteLocations?: Array<Schema['Location'] | null> | null;
    deepRelations?: Array<Array<Array<Array<Schema['Relation'] | null> | null>> | null> | null;
  };
  'Location': {
    __typename?: 'Location';
    id: string;
    name: string;
    type: string;
    frequentVisitors?: Array<Schema['Character'] | null> | null;
  };
  'TriviaItem': {
    __typename?: 'TriviaItem';
    fact: string;
    episode?: Schema['Episode'];
  };
  'Relation': {
    __typename?: 'Relation';
    relationType: string;
    character?: Schema['Character'];
  };
};
