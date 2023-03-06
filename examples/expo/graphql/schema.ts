import { GraphQLClient } from "graphql-request";
import * as Dom from "graphql-request/dist/types.dom";
import { print } from "graphql";
import gql from "graphql-tag";
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]?: Maybe<T[SubKey]>;
};
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]: Maybe<T[SubKey]>;
};
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  DateTime: any;
};

export type Emoji = {
  __typename?: "Emoji";
  char: Scalars["String"];
  /** when the model was created */
  createdAt: Scalars["DateTime"];
  /** Unique identifier */
  id: Scalars["ID"];
  tags?: Maybe<TagsConnection>;
  /** when the model was updated */
  updatedAt: Scalars["DateTime"];
};

export type EmojiTagsArgs = {
  after?: InputMaybe<Scalars["String"]>;
  before?: InputMaybe<Scalars["String"]>;
  first?: InputMaybe<Scalars["Int"]>;
  last?: InputMaybe<Scalars["Int"]>;
  orderBy?: InputMaybe<EmojiOrderByInput>;
};

export type EmojiByInput = {
  char?: InputMaybe<Scalars["String"]>;
  id?: InputMaybe<Scalars["ID"]>;
};

export type EmojiConnection = {
  __typename?: "EmojiConnection";
  edges?: Maybe<Array<Maybe<EmojiEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a Emoji */
export type EmojiCreateInput = {
  char: Scalars["String"];
  tags: Array<EmojiToTagsCreateTagsRelation>;
};

export type EmojiCreatePayload = {
  __typename?: "EmojiCreatePayload";
  emoji?: Maybe<Emoji>;
};

export type EmojiDeletePayload = {
  __typename?: "EmojiDeletePayload";
  deletedId: Scalars["ID"];
};

export type EmojiEdge = {
  __typename?: "EmojiEdge";
  cursor: Scalars["String"];
  node: Emoji;
};

export type EmojiOrderByInput = {
  createdAt?: InputMaybe<OrderByDirection>;
};

/** Input to create a Tags for the EmojiToTags relation of Emoji */
export type EmojiToTagsCreateTags = {
  text: Scalars["String"];
};

/** Input to link to or create a Tags for the EmojiToTags relation of Emoji */
export type EmojiToTagsCreateTagsRelation = {
  create?: InputMaybe<EmojiToTagsCreateTags>;
  link?: InputMaybe<Scalars["ID"]>;
};

/** Input to link/unlink to or create a Tags for the EmojiToTags relation of Emoji */
export type EmojiToTagsUpdateTagsRelation = {
  create?: InputMaybe<EmojiToTagsCreateTags>;
  link?: InputMaybe<Scalars["ID"]>;
  unlink?: InputMaybe<Scalars["ID"]>;
};

/** Input to update a Emoji */
export type EmojiUpdateInput = {
  char?: InputMaybe<Scalars["String"]>;
  tags?: InputMaybe<Array<EmojiToTagsUpdateTagsRelation>>;
};

export type EmojiUpdatePayload = {
  __typename?: "EmojiUpdatePayload";
  emoji?: Maybe<Emoji>;
};

export type Mutation = {
  __typename?: "Mutation";
  /** Create a Emoji */
  emojiCreate?: Maybe<EmojiCreatePayload>;
  /** Delete a Emoji by ID or unique field */
  emojiDelete?: Maybe<EmojiDeletePayload>;
  /** Update a Emoji */
  emojiUpdate?: Maybe<EmojiUpdatePayload>;
  /** Create a Tags */
  tagsCreate?: Maybe<TagsCreatePayload>;
  /** Delete a Tags by ID or unique field */
  tagsDelete?: Maybe<TagsDeletePayload>;
  /** Update a Tags */
  tagsUpdate?: Maybe<TagsUpdatePayload>;
};

export type MutationEmojiCreateArgs = {
  input: EmojiCreateInput;
};

export type MutationEmojiDeleteArgs = {
  by: EmojiByInput;
};

export type MutationEmojiUpdateArgs = {
  by: EmojiByInput;
  input: EmojiUpdateInput;
};

export type MutationTagsCreateArgs = {
  input: TagsCreateInput;
};

export type MutationTagsDeleteArgs = {
  by: TagsByInput;
};

export type MutationTagsUpdateArgs = {
  by: TagsByInput;
  input: TagsUpdateInput;
};

export enum OrderByDirection {
  Asc = "ASC",
  Desc = "DESC",
}

export type PageInfo = {
  __typename?: "PageInfo";
  endCursor?: Maybe<Scalars["String"]>;
  hasNextPage: Scalars["Boolean"];
  hasPreviousPage: Scalars["Boolean"];
  startCursor?: Maybe<Scalars["String"]>;
};

export type Query = {
  __typename?: "Query";
  /** Query a single Emoji by an ID or a unique field */
  emoji?: Maybe<Emoji>;
  /** Paginated query to fetch the whole list of `Emoji`. */
  emojiCollection?: Maybe<EmojiConnection>;
  /** Query a single Tags by an ID or a unique field */
  tags?: Maybe<Tags>;
  /** Paginated query to fetch the whole list of `Tags`. */
  tagsCollection?: Maybe<TagsConnection>;
};

export type QueryEmojiArgs = {
  by: EmojiByInput;
};

export type QueryEmojiCollectionArgs = {
  after?: InputMaybe<Scalars["String"]>;
  before?: InputMaybe<Scalars["String"]>;
  first?: InputMaybe<Scalars["Int"]>;
  last?: InputMaybe<Scalars["Int"]>;
  orderBy?: InputMaybe<EmojiOrderByInput>;
};

export type QueryTagsArgs = {
  by: TagsByInput;
};

export type QueryTagsCollectionArgs = {
  after?: InputMaybe<Scalars["String"]>;
  before?: InputMaybe<Scalars["String"]>;
  first?: InputMaybe<Scalars["Int"]>;
  last?: InputMaybe<Scalars["Int"]>;
  orderBy?: InputMaybe<TagsOrderByInput>;
};

export type Tags = {
  __typename?: "Tags";
  /** when the model was created */
  createdAt: Scalars["DateTime"];
  /** Unique identifier */
  id: Scalars["ID"];
  text: Scalars["String"];
  /** when the model was updated */
  updatedAt: Scalars["DateTime"];
};

export type TagsByInput = {
  id?: InputMaybe<Scalars["ID"]>;
};

export type TagsConnection = {
  __typename?: "TagsConnection";
  edges?: Maybe<Array<Maybe<TagsEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

/** Input to create a Tags */
export type TagsCreateInput = {
  text: Scalars["String"];
};

export type TagsCreatePayload = {
  __typename?: "TagsCreatePayload";
  tags?: Maybe<Tags>;
};

export type TagsDeletePayload = {
  __typename?: "TagsDeletePayload";
  deletedId: Scalars["ID"];
};

export type TagsEdge = {
  __typename?: "TagsEdge";
  cursor: Scalars["String"];
  node: Tags;
};

export type TagsOrderByInput = {
  createdAt?: InputMaybe<OrderByDirection>;
};

/** Input to update a Tags */
export type TagsUpdateInput = {
  text?: InputMaybe<Scalars["String"]>;
};

export type TagsUpdatePayload = {
  __typename?: "TagsUpdatePayload";
  tags?: Maybe<Tags>;
};

export type EmojisQueryVariables = Exact<{ [key: string]: never }>;

export type EmojisQuery = {
  __typename?: "Query";
  emojiCollection?: {
    __typename?: "EmojiConnection";
    edges?: Array<{
      __typename?: "EmojiEdge";
      node: {
        __typename?: "Emoji";
        id: string;
        char: string;
        tags?: {
          __typename?: "TagsConnection";
          edges?: Array<{
            __typename?: "TagsEdge";
            node: { __typename?: "Tags"; text: string };
          } | null> | null;
        } | null;
      };
    } | null> | null;
  } | null;
};

export const EmojisDocument = gql`
  query Emojis {
    emojiCollection(first: 99) {
      edges {
        node {
          id
          char
          tags(first: 99) {
            edges {
              node {
                text
              }
            }
          }
        }
      }
    }
  }
`;

export type SdkFunctionWrapper = <T>(
  action: (requestHeaders?: Record<string, string>) => Promise<T>,
  operationName: string,
  operationType?: string
) => Promise<T>;

const defaultWrapper: SdkFunctionWrapper = (
  action,
  _operationName,
  _operationType
) => action();
const EmojisDocumentString = print(EmojisDocument);
export function getSdk(
  client: GraphQLClient,
  withWrapper: SdkFunctionWrapper = defaultWrapper
) {
  return {
    Emojis(
      variables?: EmojisQueryVariables,
      requestHeaders?: Dom.RequestInit["headers"]
    ): Promise<{
      data: EmojisQuery;
      extensions?: any;
      headers: Dom.Headers;
      status: number;
    }> {
      return withWrapper(
        (wrappedRequestHeaders) =>
          client.rawRequest<EmojisQuery>(EmojisDocumentString, variables, {
            ...requestHeaders,
            ...wrappedRequestHeaders,
          }),
        "Emojis",
        "query"
      );
    },
  };
}
export type Sdk = ReturnType<typeof getSdk>;
