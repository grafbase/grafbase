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
  character?: Character | null;
};

export type Mutation = {
  __typename?: 'Mutation';
  createEpisode?: Episode;
};

export type Episode = {
  __typename?: 'Episode';
  id: string;
  title: string;
  season: number;
  episodeNumber: number;
  description: string | null;
  characters?: Array<Character>;
};

export type Character = {
  __typename?: 'Character';
  id: string;
  name: string;
  occupation: string | null;
  episodes?: Array<Episode>;
  friends?: Array<Character> | null;
};

export type CreateEpisodeInput = {
  title: string;
  season: number;
  episodeNumber: number;
  description: string | null;
  characters: Array<string> | null;
};
