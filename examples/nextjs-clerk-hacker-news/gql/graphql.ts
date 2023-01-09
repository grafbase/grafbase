/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /**
   * A date-time string at UTC, such as 2007-12-03T10:15:30Z, is compliant with the date-time format outlined in section 5.6 of the RFC 3339
   * profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.
   *
   * This scalar is a description of an exact instant on the timeline such as the instant that a user account was created.
   *
   * # Input Coercion
   *
   * When expected as an input type, only RFC 3339 compliant date-time strings are accepted. All other input values raise a query error indicating an incorrect type.
   *
   * # Result Coercion
   *
   * Where an RFC 3339 compliant date-time string has a time-zone other than UTC, it is shifted to UTC.
   * For example, the date-time string 2016-01-01T14:10:20+01:00 is shifted to 2016-01-01T13:10:20Z.
   */
  DateTime: any;
  /** A scalar to validate the email as it is defined in the HTML specification. */
  Email: any;
  /** An URL as defined by RFC1738. For example, `https://grafbase.com/foo/` or `mailto:example@grafbase.com`. */
  URL: any;
};

export type Comment = {
  __typename?: 'Comment';
  author: User;
  content: Scalars['String'];
  /** when the model was created */
  createdAt: Scalars['DateTime'];
  /** Unique identifier */
  id: Scalars['ID'];
  item: Item;
  /** when the model was updated */
  updatedAt: Scalars['DateTime'];
};

export type CommentByInput = {
  id?: InputMaybe<Scalars['ID']>;
};

/** Input to create a Item for the CommentRelateItem relation of Comment */
export type CommentCommentRelateItemItemCreateInput = {
  author: ItemItemRelateUserUserCreateRelationInput;
  comments?: InputMaybe<Array<InputMaybe<ItemCommentRelateItemCommentCreateRelationInput>>>;
  title: Scalars['String'];
  url: Scalars['URL'];
  votes?: InputMaybe<Array<InputMaybe<ItemItemRelateVoteVoteCreateRelationInput>>>;
};

/** Input to link to or create a Item for the CommentRelateItem relation of Comment */
export type CommentCommentRelateItemItemCreateRelationInput = {
  create?: InputMaybe<CommentCommentRelateItemItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Item for the CommentRelateItem relation of Comment */
export type CommentCommentRelateItemItemUpdateRelationInput = {
  create?: InputMaybe<CommentCommentRelateItemItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

/** Input to create a User for the CommentRelateUser relation of Comment */
export type CommentCommentRelateUserUserCreateInput = {
  comments?: InputMaybe<Array<InputMaybe<UserCommentRelateUserCommentCreateRelationInput>>>;
  email: Scalars['Email'];
  imageUrl?: InputMaybe<Scalars['String']>;
  items?: InputMaybe<Array<InputMaybe<UserItemRelateUserItemCreateRelationInput>>>;
  name: Scalars['String'];
};

/** Input to link to or create a User for the CommentRelateUser relation of Comment */
export type CommentCommentRelateUserUserCreateRelationInput = {
  create?: InputMaybe<CommentCommentRelateUserUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a User for the CommentRelateUser relation of Comment */
export type CommentCommentRelateUserUserUpdateRelationInput = {
  create?: InputMaybe<CommentCommentRelateUserUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

export type CommentConnection = {
  __typename?: 'CommentConnection';
  edges?: Maybe<Array<Maybe<CommentEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a Comment */
export type CommentCreateInput = {
  author: CommentCommentRelateUserUserCreateRelationInput;
  content: Scalars['String'];
  item: CommentCommentRelateItemItemCreateRelationInput;
};

export type CommentCreatePayload = {
  __typename?: 'CommentCreatePayload';
  comment?: Maybe<Comment>;
};

export type CommentDeletePayload = {
  __typename?: 'CommentDeletePayload';
  deletedId: Scalars['ID'];
};

export type CommentEdge = {
  __typename?: 'CommentEdge';
  cursor: Scalars['String'];
  node: Comment;
};

/** Input to update a Comment */
export type CommentUpdateInput = {
  author?: InputMaybe<CommentCommentRelateUserUserUpdateRelationInput>;
  content?: InputMaybe<Scalars['String']>;
  item?: InputMaybe<CommentCommentRelateItemItemUpdateRelationInput>;
};

export type CommentUpdatePayload = {
  __typename?: 'CommentUpdatePayload';
  comment?: Maybe<Comment>;
};

export type Item = {
  __typename?: 'Item';
  author: User;
  comments?: Maybe<CommentConnection>;
  /** when the model was created */
  createdAt: Scalars['DateTime'];
  /** Unique identifier */
  id: Scalars['ID'];
  title: Scalars['String'];
  /** when the model was updated */
  updatedAt: Scalars['DateTime'];
  url: Scalars['URL'];
  votes?: Maybe<VoteConnection>;
};


export type ItemCommentsArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type ItemVotesArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};

export type ItemByInput = {
  id?: InputMaybe<Scalars['ID']>;
};

/** Input to create a Comment for the CommentRelateItem relation of Item */
export type ItemCommentRelateItemCommentCreateInput = {
  author: CommentCommentRelateUserUserCreateRelationInput;
  content: Scalars['String'];
};

/** Input to link to or create a Comment for the CommentRelateItem relation of Item */
export type ItemCommentRelateItemCommentCreateRelationInput = {
  create?: InputMaybe<ItemCommentRelateItemCommentCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Comment for the CommentRelateItem relation of Item */
export type ItemCommentRelateItemCommentUpdateRelationInput = {
  create?: InputMaybe<ItemCommentRelateItemCommentCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

export type ItemConnection = {
  __typename?: 'ItemConnection';
  edges?: Maybe<Array<Maybe<ItemEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a Item */
export type ItemCreateInput = {
  author: ItemItemRelateUserUserCreateRelationInput;
  comments?: InputMaybe<Array<InputMaybe<ItemCommentRelateItemCommentCreateRelationInput>>>;
  title: Scalars['String'];
  url: Scalars['URL'];
  votes?: InputMaybe<Array<InputMaybe<ItemItemRelateVoteVoteCreateRelationInput>>>;
};

export type ItemCreatePayload = {
  __typename?: 'ItemCreatePayload';
  item?: Maybe<Item>;
};

export type ItemDeletePayload = {
  __typename?: 'ItemDeletePayload';
  deletedId: Scalars['ID'];
};

export type ItemEdge = {
  __typename?: 'ItemEdge';
  cursor: Scalars['String'];
  node: Item;
};

/** Input to create a User for the ItemRelateUser relation of Item */
export type ItemItemRelateUserUserCreateInput = {
  comments?: InputMaybe<Array<InputMaybe<UserCommentRelateUserCommentCreateRelationInput>>>;
  email: Scalars['Email'];
  imageUrl?: InputMaybe<Scalars['String']>;
  items?: InputMaybe<Array<InputMaybe<UserItemRelateUserItemCreateRelationInput>>>;
  name: Scalars['String'];
};

/** Input to link to or create a User for the ItemRelateUser relation of Item */
export type ItemItemRelateUserUserCreateRelationInput = {
  create?: InputMaybe<ItemItemRelateUserUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a User for the ItemRelateUser relation of Item */
export type ItemItemRelateUserUserUpdateRelationInput = {
  create?: InputMaybe<ItemItemRelateUserUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

/** Input to create a Vote for the ItemRelateVote relation of Item */
export type ItemItemRelateVoteVoteCreateInput = {
  positive: Scalars['Boolean'];
  user: VoteUserRelateVoteUserCreateRelationInput;
};

/** Input to link to or create a Vote for the ItemRelateVote relation of Item */
export type ItemItemRelateVoteVoteCreateRelationInput = {
  create?: InputMaybe<ItemItemRelateVoteVoteCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Vote for the ItemRelateVote relation of Item */
export type ItemItemRelateVoteVoteUpdateRelationInput = {
  create?: InputMaybe<ItemItemRelateVoteVoteCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

/** Input to update a Item */
export type ItemUpdateInput = {
  author?: InputMaybe<ItemItemRelateUserUserUpdateRelationInput>;
  comments?: InputMaybe<Array<InputMaybe<ItemCommentRelateItemCommentUpdateRelationInput>>>;
  title?: InputMaybe<Scalars['String']>;
  url?: InputMaybe<Scalars['URL']>;
  votes?: InputMaybe<Array<InputMaybe<ItemItemRelateVoteVoteUpdateRelationInput>>>;
};

export type ItemUpdatePayload = {
  __typename?: 'ItemUpdatePayload';
  item?: Maybe<Item>;
};

export type Mutation = {
  __typename?: 'Mutation';
  /** Create a Comment */
  commentCreate?: Maybe<CommentCreatePayload>;
  /** Delete a Comment by ID or unique field */
  commentDelete?: Maybe<CommentDeletePayload>;
  /** Update a Comment */
  commentUpdate?: Maybe<CommentUpdatePayload>;
  /** Create a Item */
  itemCreate?: Maybe<ItemCreatePayload>;
  /** Delete a Item by ID or unique field */
  itemDelete?: Maybe<ItemDeletePayload>;
  /** Update a Item */
  itemUpdate?: Maybe<ItemUpdatePayload>;
  /** Create a User */
  userCreate?: Maybe<UserCreatePayload>;
  /** Delete a User by ID or unique field */
  userDelete?: Maybe<UserDeletePayload>;
  /** Update a User */
  userUpdate?: Maybe<UserUpdatePayload>;
  /** Create a Vote */
  voteCreate?: Maybe<VoteCreatePayload>;
  /** Delete a Vote by ID or unique field */
  voteDelete?: Maybe<VoteDeletePayload>;
  /** Update a Vote */
  voteUpdate?: Maybe<VoteUpdatePayload>;
};


export type MutationCommentCreateArgs = {
  input: CommentCreateInput;
};


export type MutationCommentDeleteArgs = {
  by: CommentByInput;
};


export type MutationCommentUpdateArgs = {
  by: CommentByInput;
  input: CommentUpdateInput;
};


export type MutationItemCreateArgs = {
  input: ItemCreateInput;
};


export type MutationItemDeleteArgs = {
  by: ItemByInput;
};


export type MutationItemUpdateArgs = {
  by: ItemByInput;
  input: ItemUpdateInput;
};


export type MutationUserCreateArgs = {
  input: UserCreateInput;
};


export type MutationUserDeleteArgs = {
  by: UserByInput;
};


export type MutationUserUpdateArgs = {
  by: UserByInput;
  input: UserUpdateInput;
};


export type MutationVoteCreateArgs = {
  input: VoteCreateInput;
};


export type MutationVoteDeleteArgs = {
  by: VoteByInput;
};


export type MutationVoteUpdateArgs = {
  by: VoteByInput;
  input: VoteUpdateInput;
};

export type PageInfo = {
  __typename?: 'PageInfo';
  endCursor?: Maybe<Scalars['String']>;
  hasNextPage: Scalars['Boolean'];
  hasPreviousPage: Scalars['Boolean'];
  startCursor?: Maybe<Scalars['String']>;
};

export type Query = {
  __typename?: 'Query';
  /** Query a single Comment by an ID or a unique field */
  comment?: Maybe<Comment>;
  /** Paginated query to fetch the whole list of `Comment`. */
  commentCollection?: Maybe<CommentConnection>;
  /** Query a single Item by an ID or a unique field */
  item?: Maybe<Item>;
  /** Paginated query to fetch the whole list of `Item`. */
  itemCollection?: Maybe<ItemConnection>;
  /** Query a single User by an ID or a unique field */
  user?: Maybe<User>;
  /** Paginated query to fetch the whole list of `User`. */
  userCollection?: Maybe<UserConnection>;
  /** Query a single Vote by an ID or a unique field */
  vote?: Maybe<Vote>;
  /** Paginated query to fetch the whole list of `Vote`. */
  voteCollection?: Maybe<VoteConnection>;
};


export type QueryCommentArgs = {
  by: CommentByInput;
};


export type QueryCommentCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type QueryItemArgs = {
  by: ItemByInput;
};


export type QueryItemCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type QueryUserArgs = {
  by: UserByInput;
};


export type QueryUserCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type QueryVoteArgs = {
  by: VoteByInput;
};


export type QueryVoteCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};

export type User = {
  __typename?: 'User';
  comments?: Maybe<CommentConnection>;
  /** when the model was created */
  createdAt: Scalars['DateTime'];
  email: Scalars['Email'];
  /** Unique identifier */
  id: Scalars['ID'];
  imageUrl?: Maybe<Scalars['String']>;
  items?: Maybe<ItemConnection>;
  name: Scalars['String'];
  /** when the model was updated */
  updatedAt: Scalars['DateTime'];
};


export type UserCommentsArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type UserItemsArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};

export type UserByInput = {
  email?: InputMaybe<Scalars['Email']>;
  id?: InputMaybe<Scalars['ID']>;
};

/** Input to create a Comment for the CommentRelateUser relation of User */
export type UserCommentRelateUserCommentCreateInput = {
  content: Scalars['String'];
  item: CommentCommentRelateItemItemCreateRelationInput;
};

/** Input to link to or create a Comment for the CommentRelateUser relation of User */
export type UserCommentRelateUserCommentCreateRelationInput = {
  create?: InputMaybe<UserCommentRelateUserCommentCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Comment for the CommentRelateUser relation of User */
export type UserCommentRelateUserCommentUpdateRelationInput = {
  create?: InputMaybe<UserCommentRelateUserCommentCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

export type UserConnection = {
  __typename?: 'UserConnection';
  edges?: Maybe<Array<Maybe<UserEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a User */
export type UserCreateInput = {
  comments?: InputMaybe<Array<InputMaybe<UserCommentRelateUserCommentCreateRelationInput>>>;
  email: Scalars['Email'];
  imageUrl?: InputMaybe<Scalars['String']>;
  items?: InputMaybe<Array<InputMaybe<UserItemRelateUserItemCreateRelationInput>>>;
  name: Scalars['String'];
};

export type UserCreatePayload = {
  __typename?: 'UserCreatePayload';
  user?: Maybe<User>;
};

export type UserDeletePayload = {
  __typename?: 'UserDeletePayload';
  deletedId: Scalars['ID'];
};

export type UserEdge = {
  __typename?: 'UserEdge';
  cursor: Scalars['String'];
  node: User;
};

/** Input to create a Item for the ItemRelateUser relation of User */
export type UserItemRelateUserItemCreateInput = {
  comments?: InputMaybe<Array<InputMaybe<ItemCommentRelateItemCommentCreateRelationInput>>>;
  title: Scalars['String'];
  url: Scalars['URL'];
  votes?: InputMaybe<Array<InputMaybe<ItemItemRelateVoteVoteCreateRelationInput>>>;
};

/** Input to link to or create a Item for the ItemRelateUser relation of User */
export type UserItemRelateUserItemCreateRelationInput = {
  create?: InputMaybe<UserItemRelateUserItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Item for the ItemRelateUser relation of User */
export type UserItemRelateUserItemUpdateRelationInput = {
  create?: InputMaybe<UserItemRelateUserItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

/** Input to update a User */
export type UserUpdateInput = {
  comments?: InputMaybe<Array<InputMaybe<UserCommentRelateUserCommentUpdateRelationInput>>>;
  email?: InputMaybe<Scalars['Email']>;
  imageUrl?: InputMaybe<Scalars['String']>;
  items?: InputMaybe<Array<InputMaybe<UserItemRelateUserItemUpdateRelationInput>>>;
  name?: InputMaybe<Scalars['String']>;
};

export type UserUpdatePayload = {
  __typename?: 'UserUpdatePayload';
  user?: Maybe<User>;
};

export type Vote = {
  __typename?: 'Vote';
  /** when the model was created */
  createdAt: Scalars['DateTime'];
  /** Unique identifier */
  id: Scalars['ID'];
  item: Item;
  positive: Scalars['Boolean'];
  /** when the model was updated */
  updatedAt: Scalars['DateTime'];
  user: User;
};

export type VoteByInput = {
  id?: InputMaybe<Scalars['ID']>;
};

export type VoteConnection = {
  __typename?: 'VoteConnection';
  edges?: Maybe<Array<Maybe<VoteEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a Vote */
export type VoteCreateInput = {
  item: VoteItemRelateVoteItemCreateRelationInput;
  positive: Scalars['Boolean'];
  user: VoteUserRelateVoteUserCreateRelationInput;
};

export type VoteCreatePayload = {
  __typename?: 'VoteCreatePayload';
  vote?: Maybe<Vote>;
};

export type VoteDeletePayload = {
  __typename?: 'VoteDeletePayload';
  deletedId: Scalars['ID'];
};

export type VoteEdge = {
  __typename?: 'VoteEdge';
  cursor: Scalars['String'];
  node: Vote;
};

/** Input to create a Item for the ItemRelateVote relation of Vote */
export type VoteItemRelateVoteItemCreateInput = {
  author: ItemItemRelateUserUserCreateRelationInput;
  comments?: InputMaybe<Array<InputMaybe<ItemCommentRelateItemCommentCreateRelationInput>>>;
  title: Scalars['String'];
  url: Scalars['URL'];
  votes?: InputMaybe<Array<InputMaybe<ItemItemRelateVoteVoteCreateRelationInput>>>;
};

/** Input to link to or create a Item for the ItemRelateVote relation of Vote */
export type VoteItemRelateVoteItemCreateRelationInput = {
  create?: InputMaybe<VoteItemRelateVoteItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a Item for the ItemRelateVote relation of Vote */
export type VoteItemRelateVoteItemUpdateRelationInput = {
  create?: InputMaybe<VoteItemRelateVoteItemCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

/** Input to update a Vote */
export type VoteUpdateInput = {
  item?: InputMaybe<VoteItemRelateVoteItemUpdateRelationInput>;
  positive?: InputMaybe<Scalars['Boolean']>;
  user?: InputMaybe<VoteUserRelateVoteUserUpdateRelationInput>;
};

export type VoteUpdatePayload = {
  __typename?: 'VoteUpdatePayload';
  vote?: Maybe<Vote>;
};

/** Input to create a User for the UserRelateVote relation of Vote */
export type VoteUserRelateVoteUserCreateInput = {
  comments?: InputMaybe<Array<InputMaybe<UserCommentRelateUserCommentCreateRelationInput>>>;
  email: Scalars['Email'];
  imageUrl?: InputMaybe<Scalars['String']>;
  items?: InputMaybe<Array<InputMaybe<UserItemRelateUserItemCreateRelationInput>>>;
  name: Scalars['String'];
};

/** Input to link to or create a User for the UserRelateVote relation of Vote */
export type VoteUserRelateVoteUserCreateRelationInput = {
  create?: InputMaybe<VoteUserRelateVoteUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to link/unlink to or create a User for the UserRelateVote relation of Vote */
export type VoteUserRelateVoteUserUpdateRelationInput = {
  create?: InputMaybe<VoteUserRelateVoteUserCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
};

export type CommentAddMutationVariables = Exact<{
  content: Scalars['String'];
  authorId: Scalars['ID'];
  itemId: Scalars['ID'];
}>;


export type CommentAddMutation = { __typename?: 'Mutation', commentCreate?: { __typename?: 'CommentCreatePayload', comment?: { __typename: 'Comment' } | null } | null };

export type ItemCommentDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type ItemCommentDeleteMutation = { __typename?: 'Mutation', commentDelete?: { __typename?: 'CommentDeletePayload', deletedId: string } | null };

export type ItemVoteMutationVariables = Exact<{
  vote: Scalars['Boolean'];
  authorId: Scalars['ID'];
  itemId: Scalars['ID'];
}>;


export type ItemVoteMutation = { __typename?: 'Mutation', voteCreate?: { __typename?: 'VoteCreatePayload', vote?: { __typename: 'Vote' } | null } | null };

export type ItemVoteUpdateMutationVariables = Exact<{
  id: Scalars['ID'];
  vote: Scalars['Boolean'];
}>;


export type ItemVoteUpdateMutation = { __typename?: 'Mutation', voteUpdate?: { __typename?: 'VoteUpdatePayload', vote?: { __typename: 'Vote' } | null } | null };

export type ViewerQueryVariables = Exact<{
  email: Scalars['Email'];
}>;


export type ViewerQuery = { __typename?: 'Query', user?: { __typename?: 'User', id: string, name: string, email: any, imageUrl?: string | null, createdAt: any, items?: { __typename?: 'ItemConnection', edges?: Array<{ __typename: 'ItemEdge' } | null> | null } | null } | null };

export type UserCreateLoginMutationVariables = Exact<{
  name: Scalars['String'];
  email: Scalars['Email'];
  imageUrl: Scalars['String'];
}>;


export type UserCreateLoginMutation = { __typename?: 'Mutation', userCreate?: { __typename: 'UserCreatePayload' } | null };

export type UserUpdateLoginMutationVariables = Exact<{
  id: Scalars['ID'];
  imageUrl: Scalars['String'];
}>;


export type UserUpdateLoginMutation = { __typename?: 'Mutation', userUpdate?: { __typename: 'UserUpdatePayload' } | null };

export type ItemsListQueryVariables = Exact<{
  after?: InputMaybe<Scalars['String']>;
}>;


export type ItemsListQuery = { __typename?: 'Query', itemCollection?: { __typename?: 'ItemConnection', pageInfo: { __typename?: 'PageInfo', endCursor?: string | null, hasNextPage: boolean }, edges?: Array<{ __typename?: 'ItemEdge', node: { __typename?: 'Item', id: string, title: string, url: any, createdAt: any, comments?: { __typename?: 'CommentConnection', edges?: Array<{ __typename?: 'CommentEdge', node: { __typename: 'Comment' } } | null> | null } | null, votes?: { __typename?: 'VoteConnection', edges?: Array<{ __typename?: 'VoteEdge', node: { __typename?: 'Vote', id: string, positive: boolean, user: { __typename?: 'User', id: string } } } | null> | null } | null, author: { __typename?: 'User', id: string, name: string, imageUrl?: string | null } } } | null> | null } | null };

export type ItemOneQueryVariables = Exact<{
  id: Scalars['ID'];
  afterComments?: InputMaybe<Scalars['String']>;
}>;


export type ItemOneQuery = { __typename?: 'Query', item?: { __typename?: 'Item', id: string, title: string, url: any, createdAt: any, comments?: { __typename?: 'CommentConnection', pageInfo: { __typename?: 'PageInfo', endCursor?: string | null, hasNextPage: boolean }, edges?: Array<{ __typename?: 'CommentEdge', node: { __typename?: 'Comment', id: string, content: string, createdAt: any, author: { __typename?: 'User', id: string, name: string, imageUrl?: string | null } } } | null> | null } | null, votes?: { __typename?: 'VoteConnection', edges?: Array<{ __typename?: 'VoteEdge', node: { __typename?: 'Vote', id: string, positive: boolean, user: { __typename?: 'User', id: string } } } | null> | null } | null, author: { __typename?: 'User', id: string, name: string, imageUrl?: string | null } } | null };

export type ItemOneDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type ItemOneDeleteMutation = { __typename?: 'Mutation', itemDelete?: { __typename?: 'ItemDeletePayload', deletedId: string } | null };

export type ItemMutationVariables = Exact<{
  title: Scalars['String'];
  url: Scalars['URL'];
  authorId: Scalars['ID'];
}>;


export type ItemMutation = { __typename?: 'Mutation', itemCreate?: { __typename?: 'ItemCreatePayload', item?: { __typename?: 'Item', id: string } | null } | null };

export type UsersListQueryVariables = Exact<{
  after?: InputMaybe<Scalars['String']>;
}>;


export type UsersListQuery = { __typename?: 'Query', userCollection?: { __typename?: 'UserConnection', pageInfo: { __typename?: 'PageInfo', endCursor?: string | null, hasNextPage: boolean }, edges?: Array<{ __typename?: 'UserEdge', node: { __typename?: 'User', id: string, name: string, imageUrl?: string | null, createdAt: any } } | null> | null } | null };


export const CommentAddDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CommentAdd"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"content"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"commentCreate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"content"},"value":{"kind":"Variable","name":{"kind":"Name","value":"content"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"author"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"link"},"value":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}}}]}},{"kind":"ObjectField","name":{"kind":"Name","value":"item"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"link"},"value":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}}}]}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"comment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}}]} as unknown as DocumentNode<CommentAddMutation, CommentAddMutationVariables>;
export const ItemCommentDeleteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ItemCommentDelete"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"commentDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deletedId"}}]}}]}}]} as unknown as DocumentNode<ItemCommentDeleteMutation, ItemCommentDeleteMutationVariables>;
export const ItemVoteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ItemVote"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"vote"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"voteCreate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"positive"},"value":{"kind":"Variable","name":{"kind":"Name","value":"vote"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"user"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"link"},"value":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}}}]}},{"kind":"ObjectField","name":{"kind":"Name","value":"item"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"link"},"value":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}}}]}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"vote"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}}]} as unknown as DocumentNode<ItemVoteMutation, ItemVoteMutationVariables>;
export const ItemVoteUpdateDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ItemVoteUpdate"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"vote"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"voteUpdate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"positive"},"value":{"kind":"Variable","name":{"kind":"Name","value":"vote"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"vote"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}}]} as unknown as DocumentNode<ItemVoteUpdateMutation, ItemVoteUpdateMutationVariables>;
export const ViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Viewer"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"email"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Email"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"user"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"email"},"value":{"kind":"Variable","name":{"kind":"Name","value":"email"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"items"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"3"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}}]}}]} as unknown as DocumentNode<ViewerQuery, ViewerQueryVariables>;
export const UserCreateLoginDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UserCreateLogin"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"email"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Email"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"imageUrl"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userCreate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"email"},"value":{"kind":"Variable","name":{"kind":"Name","value":"email"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"imageUrl"},"value":{"kind":"Variable","name":{"kind":"Name","value":"imageUrl"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]} as unknown as DocumentNode<UserCreateLoginMutation, UserCreateLoginMutationVariables>;
export const UserUpdateLoginDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UserUpdateLogin"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"imageUrl"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userUpdate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}},{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"imageUrl"},"value":{"kind":"Variable","name":{"kind":"Name","value":"imageUrl"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]} as unknown as DocumentNode<UserUpdateLoginMutation, UserUpdateLoginMutationVariables>;
export const ItemsListDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ItemsList"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"itemCollection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"4"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}},{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"comments"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"100"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"votes"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"100"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"positive"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}}]}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]}}]}}]} as unknown as DocumentNode<ItemsListQuery, ItemsListQueryVariables>;
export const ItemOneDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ItemOne"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"afterComments"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"item"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"comments"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"6"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"afterComments"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}},{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"content"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"votes"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"100"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"positive"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"author"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}}]}},{"kind":"Field","name":{"kind":"Name","value":"url"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]} as unknown as DocumentNode<ItemOneQuery, ItemOneQueryVariables>;
export const ItemOneDeleteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ItemOneDelete"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"itemDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"by"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deletedId"}}]}}]}}]} as unknown as DocumentNode<ItemOneDeleteMutation, ItemOneDeleteMutationVariables>;
export const ItemDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Item"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"title"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"url"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"URL"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"itemCreate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"title"},"value":{"kind":"Variable","name":{"kind":"Name","value":"title"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"url"},"value":{"kind":"Variable","name":{"kind":"Name","value":"url"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"author"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"link"},"value":{"kind":"Variable","name":{"kind":"Name","value":"authorId"}}}]}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"item"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<ItemMutation, ItemMutationVariables>;
export const UsersListDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"UsersList"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userCollection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"10"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}},{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]}}]}}]} as unknown as DocumentNode<UsersListQuery, UsersListQueryVariables>;