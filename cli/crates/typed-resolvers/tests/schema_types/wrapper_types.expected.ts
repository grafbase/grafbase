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

export type Query = {
  __typename?: 'Query';
  episode?: Episode | null;
  episodesBySeason?: Array<Episode>;
  character?: Character | null;
  searchCharacters?: Array<Character>;
  locations?: Array<Location>;
};

export type Mutation = {
  __typename?: 'Mutation';
  createEpisode?: Episode;
  updateEpisode?: Episode | null;
};

export type CreateEpisodeInput = {
  title: string;
  season: number;
  episodeNumber: number;
  description: string | null;
  characters: Array<string>;
};

export type UpdateEpisodeInput = {
  title: string | null;
  season: number | null;
  episodeNumber: number | null;
  description: string | null;
  characters: Array<string> | null;
};

export type Episode = {
  __typename?: 'Episode';
  id: string;
  title: string;
  season: number;
  episodeNumber: number;
  description: string | null;
  characters?: Array<Character>;
  nestedTrivia?: Array<Array<Array<TriviaItem> | null> | null> | null;
};

export type Character = {
  __typename?: 'Character';
  id: string;
  name: string;
  occupation: string | null;
  episodes?: Array<Episode>;
  friends?: Array<Character> | null;
  favoriteLocations?: Array<Location | null> | null;
  deepRelations?: Array<Array<Array<Array<Relation | null> | null>> | null> | null;
};

export type Location = {
  __typename?: 'Location';
  id: string;
  name: string;
  type: string;
  frequentVisitors?: Array<Character | null> | null;
};

export type TriviaItem = {
  __typename?: 'TriviaItem';
  fact: string;
  episode?: Episode;
};

export type Relation = {
  __typename?: 'Relation';
  relationType: string;
  character?: Character;
};
