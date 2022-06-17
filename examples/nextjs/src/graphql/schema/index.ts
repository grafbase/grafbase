import gql from 'graphql-tag';
import * as Urql from 'urql';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
};

export type Mutation = {
  __typename?: 'Mutation';
  /** Create a Todo */
  todoCreate?: Maybe<TodoCreatePayload>;
  /** Delete a Todo by ID */
  todoDelete?: Maybe<TodoDeletePayload>;
  /** Create a TodoList */
  todoListCreate?: Maybe<TodoListCreatePayload>;
  /** Delete a TodoList by ID */
  todoListDelete?: Maybe<TodoListDeletePayload>;
};


export type MutationTodoCreateArgs = {
  input: TodoCreationInput;
};


export type MutationTodoDeleteArgs = {
  id: Scalars['ID'];
};


export type MutationTodoListCreateArgs = {
  input: TodoListCreationInput;
};


export type MutationTodoListDeleteArgs = {
  id: Scalars['ID'];
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
  /** Get a Todo by ID */
  todo?: Maybe<Todo>;
  /** Paginated query to fetch the whole list of `Todo`. */
  todoCollection?: Maybe<TodoConnection>;
  /** Get a TodoList by ID */
  todoList?: Maybe<TodoList>;
  /** Paginated query to fetch the whole list of `TodoList`. */
  todoListCollection?: Maybe<TodoListConnection>;
};


export type QueryTodoArgs = {
  id: Scalars['ID'];
};


export type QueryTodoCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};


export type QueryTodoListArgs = {
  id: Scalars['ID'];
};


export type QueryTodoListCollectionArgs = {
  after?: InputMaybe<Scalars['String']>;
  before?: InputMaybe<Scalars['String']>;
  first?: InputMaybe<Scalars['Int']>;
  last?: InputMaybe<Scalars['Int']>;
};

export type Todo = {
  __typename?: 'Todo';
  complete?: Maybe<Scalars['Boolean']>;
  id: Scalars['ID'];
  list: TodoList;
  title: Scalars['String'];
};

export type TodoConnection = {
  __typename?: 'TodoConnection';
  edges?: Maybe<Array<Maybe<TodoEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

export type TodoCreatePayload = {
  __typename?: 'TodoCreatePayload';
  todo?: Maybe<Todo>;
};

/** Input to create a new Todo */
export type TodoCreationInput = {
  complete?: InputMaybe<Scalars['Boolean']>;
  list: TodoTodoRelateTodoListTodoListCreateInput;
  title: Scalars['String'];
};

export type TodoDeletePayload = {
  __typename?: 'TodoDeletePayload';
  deletedId: Scalars['ID'];
};

export type TodoEdge = {
  __typename?: 'TodoEdge';
  cursor: Scalars['String'];
  node: Todo;
};

export type TodoList = {
  __typename?: 'TodoList';
  id: Scalars['ID'];
  title: Scalars['String'];
  todos?: Maybe<Array<Maybe<Todo>>>;
};

export type TodoListConnection = {
  __typename?: 'TodoListConnection';
  edges?: Maybe<Array<Maybe<TodoListEdge>>>;
  /** Information to aid in pagination */
  pageInfo: PageInfo;
};

export type TodoListCreatePayload = {
  __typename?: 'TodoListCreatePayload';
  todoList?: Maybe<TodoList>;
};

/** Input to create a new TodoList */
export type TodoListCreationInput = {
  title: Scalars['String'];
  todos?: InputMaybe<Array<InputMaybe<TodoListTodoRelateTodoListTodoCreateInput>>>;
};

export type TodoListDeletePayload = {
  __typename?: 'TodoListDeletePayload';
  deletedId: Scalars['ID'];
};

export type TodoListEdge = {
  __typename?: 'TodoListEdge';
  cursor: Scalars['String'];
  node: TodoList;
};

/** Input to create a new TodoListTodoRelateTodoListTodoCreateInput */
export type TodoListTodoRelateTodoListTodoCreateInput = {
  create?: InputMaybe<TodoListTodoRelateTodoListTodoCreationInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to create a new TodoListTodoRelateTodoListTodoCreationInput */
export type TodoListTodoRelateTodoListTodoCreationInput = {
  complete?: InputMaybe<Scalars['Boolean']>;
  title: Scalars['String'];
};

/** Input to create a new TodoTodoRelateTodoListTodoListCreateInput */
export type TodoTodoRelateTodoListTodoListCreateInput = {
  create?: InputMaybe<TodoTodoRelateTodoListTodoListCreationInput>;
  link?: InputMaybe<Scalars['ID']>;
};

/** Input to create a new TodoTodoRelateTodoListTodoListCreationInput */
export type TodoTodoRelateTodoListTodoListCreationInput = {
  title: Scalars['String'];
};

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
export const TodosDocument = gql`
    query Todos {
  todoListCollection(first: 99) {
    edges {
      node {
        ...TodoList
      }
    }
  }
}
    ${TodoListFragmentDoc}`;

export function useTodosQuery(options?: Omit<Urql.UseQueryArgs<TodosQueryVariables>, 'query'>) {
  return Urql.useQuery<TodosQuery>({ query: TodosDocument, ...options });
};
export const TodoListCreateDocument = gql`
    mutation TodoListCreate($title: String!) {
  todoListCreate(input: {title: $title}) {
    todoList {
      id
    }
  }
}
    `;

export function useTodoListCreateMutation() {
  return Urql.useMutation<TodoListCreateMutation, TodoListCreateMutationVariables>(TodoListCreateDocument);
};
export const TodoCreateDocument = gql`
    mutation TodoCreate($title: String!, $todoListId: ID!) {
  todoCreate(input: {title: $title, complete: false, list: {link: $todoListId}}) {
    todo {
      id
    }
  }
}
    `;

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
export type TodoListFragment = { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null };

export type TodoFragment = { __typename?: 'Todo', id: string, title: string, complete?: boolean | null };

export type TodosQueryVariables = Exact<{ [key: string]: never; }>;


export type TodosQuery = { __typename?: 'Query', todoListCollection?: { __typename?: 'TodoListConnection', edges?: Array<{ __typename?: 'TodoListEdge', node: { __typename?: 'TodoList', id: string, title: string, todos?: Array<{ __typename?: 'Todo', id: string, title: string, complete?: boolean | null } | null> | null } } | null> | null } | null };

export type TodoListCreateMutationVariables = Exact<{
  title: Scalars['String'];
}>;


export type TodoListCreateMutation = { __typename?: 'Mutation', todoListCreate?: { __typename?: 'TodoListCreatePayload', todoList?: { __typename?: 'TodoList', id: string } | null } | null };

export type TodoCreateMutationVariables = Exact<{
  title: Scalars['String'];
  todoListId: Scalars['ID'];
}>;


export type TodoCreateMutation = { __typename?: 'Mutation', todoCreate?: { __typename?: 'TodoCreatePayload', todo?: { __typename?: 'Todo', id: string } | null } | null };

export type TodoListDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type TodoListDeleteMutation = { __typename?: 'Mutation', todoListDelete?: { __typename?: 'TodoListDeletePayload', deletedId: string } | null };

export type TodoDeleteMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type TodoDeleteMutation = { __typename?: 'Mutation', todoDelete?: { __typename?: 'TodoDeletePayload', deletedId: string } | null };
