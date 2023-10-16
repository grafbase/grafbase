export type Dog = {
  __typename?: 'Dog';
  id: string;
  barkVolume: number | null;
};

export type Cat = {
  __typename?: 'Cat';
  id: string;
  meowVolume: number | null;
};

export type Pet = Cat | Dog;
