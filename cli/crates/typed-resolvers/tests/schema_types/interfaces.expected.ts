export type Human = {
  __typename?: 'Human';
  id: string;
  firstName: string;
  lastName: string;
};

export type Dog = {
  __typename?: 'Dog';
  id: string;
  name: string;
};

export type LivingThing = {
  metabolicRate: number;
  age: number;
};

export type Animal = {
  pettable: boolean;
};
