use macros::rpc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "::trait_link::serde")]
#[allow(unused)]
struct Todo {
    name: String,
    description: String,
}

#[rpc]
trait TodoService {
    fn get_todos(&self) -> Vec<Todo>;
    fn get_todo(&self, name: String) -> Option<Todo>;
    fn new_todo(&self, todo: Todo);
}