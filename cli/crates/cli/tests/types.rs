#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Node<T> {
    pub node: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeList<T> {
    pub edges: Vec<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoListCollection<T> {
    pub todo_list_collection: T,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct TodoList {
    pub id: String,
    pub title: String,
    pub todos: Vec<Todo>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorCollection<T> {
    pub author_collection: T,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
}

pub type TodoListCollectionResponse = GraphQLResponse<TodoListCollection<EdgeList<Node<TodoList>>>>;
pub type AuthorCollectionResponse = GraphQLResponse<AuthorCollection<EdgeList<Node<Author>>>>;
