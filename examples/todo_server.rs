use std::ops::Deref;
use std::sync::{Arc};
use tokio::sync::RwLock;
use axum::routing::post;

include!("traits/todo.rs");

#[derive(Default)]
struct Todos {
    todos: RwLock<Vec<Todo>>
}

impl TodoServiceServer for &Todos {
    async fn get_todos(self) -> Vec<Todo> {
        self.todos.read().await.deref().clone()
    }

    async fn get_todo(self, name: String) -> Option<Todo> {
        self.todos.read().await.iter().find(|todo| todo.name == name).cloned()
    }

    async fn new_todo(self, todo: Todo) -> () {
        self.todos.write().await.push(todo)
    }
}

#[tokio::main]
async fn main() {
    let server: &'static _ = Box::leak(Box::new(Todos::default()));
    let app = axum::Router::new()
        .route("/api/todo", post(trait_link::server::axum::json))
        .with_state(Arc::new(TodoService::server(server)));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve::serve(listener, app).await.unwrap()
}
