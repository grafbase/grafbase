import gql from 'graphql-tag';
import * as Urql from '@urql/vue';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>;
/** All built-in and custom scalars, mapped to their actual values */
export interface Scalars {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
}

export interface Mutation {
  __typename?: 'Mutation';
  /** Create a Todo */
  todoCreate?: Maybe<TodoCreatePayload>;
  /** Delete a Todo by ID */
  todoDelete?: Maybe<TodoDeletePayload>;
  /** Create a TodoList */
  todoListCreate?: Maybe<TodoListCreatePayload>;
  /** Delete a TodoList by ID */
  todoListDelete?: Maybe<TodoListDeletePayload>;
  /** Update a TodoList */
  todoListUpdate?: Maybe<TodoListUpdatePayload>;
  /** Update a Todo */
  todoUpdate?: Maybe<TodoUpdatePayload>;
}


export interface MutationTodoCreateArgs {
  input: TodoCreateInput;
}


export interface MutationTodoDeleteArgs {
  id: Scalars['ID'];
}


export interface MutationTodoListCreateArgs {
  input: TodoListCreateInput;
}


export interface MutationTodoListDeleteArgs {
  id: Scalars['ID'];
}


export interface MutationTodoListUpdateArgs {
  id: Scalars['ID'];
  input: TodoListUpdateInput;
}


export interface MutationTodoUpdateArgs {
  id: Scalars['ID'];
  input: TodoUpdateInput;
}

export interface PageInfo {
  __typename?: 'PageInfo';
  endCursor?: Maybe<Scalars['String']>;
  hasNextPage: Scalars['Boolean'];
  hasPreviousPage: Scalars['Boolean'];
  startCursor?: Maybe<Scalars['String']>;
}

export interface Query {
  __typename?: 'Query';
  /** Get a Todo by ID */
  todo?: Maybe<Todo>;
  /** Paginated query to fetch the whole list of `Todo`. */
  todoCollection?: Maybe<TodoConnection>;
  /** Get a TodoList by ID */
  todoList?: Maybe<TodoList>;
  /** Paginated query to fetch the whole list of `TodoList`. */
  todoListCollection?: Maybe<TodoListConnection>;
}


export interface QueryTodoArgs {
  id: Scalars['ID'];
}


export interface QueryTodoCollectionArgs {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
}


export interface QueryTodoListArgs {
  id: Scalars['ID'];
}


export interface QueryTodoListCollectionArgs {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
}

export interface Todo {
  __typename?: 'Todo';
  complete?: Maybe<Scalars['Boolean']>;
  id: Scalars['ID'];
  list?: Maybe<TodoList>;
  title: Scalars['String'];
}

export interface TodoConnection {
  __typename?: 'TodoConnection';
  edges?: Maybe<Array<Maybe<TodoEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
}

/** Input to create a new Todo */
export interface TodoCreateInput {
  complete?: InputMaybe<Scalars['Boolean']>;
  list?: InputMaybe<TodoTodoRelateTodoListTodoListCreateRelationInput>;
  title: Scalars['String'];
}

export interface TodoCreatePayload {
  __typename?: 'TodoCreatePayload';
  todo?: Maybe<Todo>;
}

export interface TodoDeletePayload {
  __typename?: 'TodoDeletePayload';
  deletedId: Scalars['ID'];
}

export interface TodoEdge {
  __typename?: 'TodoEdge';
  cursor: Scalars['String'];
  node: Todo;
}

export interface TodoList {
  __typename?: 'TodoList';
  id: Scalars['ID'];
  title: Scalars['String'];
  todos?: Maybe<Array<Maybe<Todo>>>;
}

export interface TodoListConnection {
  __typename?: 'TodoListConnection';
  edges?: Maybe<Array<Maybe<TodoListEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
}

/** Input to create a new TodoList */
export interface TodoListCreateInput {
  title: Scalars['String'];
  todos?: InputMaybe<Array<InputMaybe<TodoListTodoRelateTodoListTodoCreateRelationInput>>>;
}

export interface TodoListCreatePayload {
  __typename?: 'TodoListCreatePayload';
  todoList?: Maybe<TodoList>;
}

export interface TodoListDeletePayload {
  __typename?: 'TodoListDeletePayload';
  deletedId: Scalars['ID'];
}

export interface TodoListEdge {
  __typename?: 'TodoListEdge';
  cursor: Scalars['String'];
  node: TodoList;
}

/** Input to create a new TodoListTodoRelateTodoListTodo */
export interface TodoListTodoRelateTodoListTodoCreateInput {
  complete?: InputMaybe<Scalars['Boolean']>;
  title: Scalars['String'];
}

/** Input to create a new TodoListTodoRelateTodoListTodo relation */
export interface TodoListTodoRelateTodoListTodoCreateRelationInput {
  create?: InputMaybe<TodoListTodoRelateTodoListTodoCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
}

/** Input to update a TodoListTodoRelateTodoListTodo relation */
export interface TodoListTodoRelateTodoListTodoUpdateRelationInput {
  create?: InputMaybe<TodoListTodoRelateTodoListTodoCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
}

/** Input to create a new TodoList */
export interface TodoListUpdateInput {
  title?: InputMaybe<Scalars['String']>;
  todos?: InputMaybe<Array<InputMaybe<TodoListTodoRelateTodoListTodoUpdateRelationInput>>>;
}

export interface TodoListUpdatePayload {
  __typename?: 'TodoListUpdatePayload';
  todoList?: Maybe<TodoList>;
}

/** Input to create a new TodoTodoRelateTodoListTodoList */
export interface TodoTodoRelateTodoListTodoListCreateInput {
  title: Scalars['String'];
}

/** Input to create a new TodoTodoRelateTodoListTodoList relation */
export interface TodoTodoRelateTodoListTodoListCreateRelationInput {
  create?: InputMaybe<TodoTodoRelateTodoListTodoListCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
}

/** Input to update a TodoTodoRelateTodoListTodoList relation */
export interface TodoTodoRelateTodoListTodoListUpdateRelationInput {
  create?: InputMaybe<TodoTodoRelateTodoListTodoListCreateInput>;
  link?: InputMaybe<Scalars['ID']>;
  unlink?: InputMaybe<Scalars['ID']>;
}

/** Input to create a new Todo */
export interface TodoUpdateInput {
  complete?: InputMaybe<Scalars['Boolean']>;
  list?: InputMaybe<TodoTodoRelateTodoListTodoListUpdateRelationInput>;
  title?: InputMaybe<Scalars['String']>;
}

export interface TodoUpdatePayload {
  __typename?: 'TodoUpdatePayload';
  todo?: Maybe<Todo>;
}

export type TodoListFragment = { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null };

export type TodoFragment = { __typename?: 'Todo', id: string, title: string, complete?: boolean | null };

export type TodoListsQueryVariables = Exact<{ [key: string]: never; }>;


export type TodoListsQuery = { __typename?: 'Query', todoListCollection?: { __typename?: 'TodoListConnection', edges?: Array<{ __typename?: 'TodoListEdge', node: { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null } } | null> | null } | null };

export type TodoListCreateMutationVariables = Exact<{
  title: Scalars['String'];
}>;


export type TodoListCreateMutation = { __typename?: 'Mutation', todoListCreate?: { __typename?: 'TodoListCreatePayload', todoList?: { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null } | null } | null };

export type TodoCreateMutationVariables = Exact<{
  title: Scalars['String'];
  todoListId: Scalars['ID'];
}>;


export type TodoCreateMutation = { __typename?: 'Mutation', todoCreate?: { __typename?: 'TodoCreatePayload', todo?: { __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null } | null };

export type TodoListDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type TodoListDeleteMutation = { __typename?: 'Mutation', todoListDelete?: { __typename?: 'TodoListDeletePayload', deletedId: string } | null };

export type TodoListUpdateMutationVariables = Exact<{
  id: Scalars['ID'];
  title?: InputMaybe<Scalars['String']>;
}>;


export type TodoListUpdateMutation = { __typename?: 'Mutation', todoListUpdate?: { __typename?: 'TodoListUpdatePayload', todoList?: { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null } | null } | null };

export type TodoDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type TodoDeleteMutation = { __typename?: 'Mutation', todoDelete?: { __typename?: 'TodoDeletePayload', deletedId: string } | null };

export type TodoUpdateMutationVariables = Exact<{
  id: Scalars['ID'];
  title?: InputMaybe<Scalars['String']>;
  complete?: InputMaybe<Scalars['Boolean']>;
}>;


export type TodoUpdateMutation = { __typename?: 'Mutation', todoUpdate?: { __typename?: 'TodoUpdatePayload', todo?: { __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null } | null };

export const TodoFragmentDoc = gql`
    fragment Todo on Todo {
  id
  title
  complete
}
    `;
export const TodoListFragmentDoc = gql`
    fragment TodoList on TodoList {
  id
  title
  todos {
    ...Todo
  }
}
    ${TodoFragmentDoc}`;
export const TodoListsDocument = gql`
    query TodoLists {
  todoListCollection(first: 99) {
    edges {
      node {
        ...TodoList
      }
    }
  }
}
    ${TodoListFragmentDoc}`;

export function useTodoListsQuery(options: Omit<Urql.UseQueryArgs<never, TodoListsQueryVariables>, 'query'> = {}) {
  return Urql.useQuery<TodoListsQuery>({ query: TodoListsDocument, ...options });
};
export const TodoListCreateDocument = gql`
    mutation TodoListCreate($title: String!) {
  todoListCreate(input: {title: $title}) {
    todoList {
      ...TodoList
    }
  }
}
    ${TodoListFragmentDoc}`;

export function useTodoListCreateMutation() {
  return Urql.useMutation<TodoListCreateMutation, TodoListCreateMutationVariables>(TodoListCreateDocument);
};
export const TodoCreateDocument = gql`
    mutation TodoCreate($title: String!, $todoListId: ID!) {
  todoCreate(input: {title: $title, complete: false, list: {link: $todoListId}}) {
    todo {
      ...Todo
    }
  }
}
    ${TodoFragmentDoc}`;

export function useTodoCreateMutation() {
  return Urql.useMutation<TodoCreateMutation, TodoCreateMutationVariables>(TodoCreateDocument);
};
export const TodoListDeleteDocument = gql`
    mutation TodoListDelete($id: ID!) {
  todoListDelete(id: $id) {
    deletedId
  }
}
    `;

export function useTodoListDeleteMutation() {
  return Urql.useMutation<TodoListDeleteMutation, TodoListDeleteMutationVariables>(TodoListDeleteDocument);
};
export const TodoListUpdateDocument = gql`
    mutation TodoListUpdate($id: ID!, $title: String) {
  todoListUpdate(id: $id, input: {title: $title}) {
    todoList {
      ...TodoList
    }
  }
}
    ${TodoListFragmentDoc}`;

export function useTodoListUpdateMutation() {
  return Urql.useMutation<TodoListUpdateMutation, TodoListUpdateMutationVariables>(TodoListUpdateDocument);
};
export const TodoDeleteDocument = gql`
    mutation TodoDelete($id: ID!) {
  todoDelete(id: $id) {
    deletedId
  }
}
    `;

export function useTodoDeleteMutation() {
  return Urql.useMutation<TodoDeleteMutation, TodoDeleteMutationVariables>(TodoDeleteDocument);
};
export const TodoUpdateDocument = gql`
    mutation TodoUpdate($id: ID!, $title: String, $complete: Boolean) {
  todoUpdate(id: $id, input: {title: $title, complete: $complete}) {
    todo {
      ...Todo
    }
  }
}
    ${TodoFragmentDoc}`;

export function useTodoUpdateMutation() {
  return Urql.useMutation<TodoUpdateMutation, TodoUpdateMutationVariables>(TodoUpdateDocument);
};