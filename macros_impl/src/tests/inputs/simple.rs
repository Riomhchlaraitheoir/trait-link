#[rpc]
pub trait TodoService {
    fn get_todos(&self) -> Vec<Todo>;
    fn get_todo(&self, name: String) -> Option<Todo>;
    fn new_todo(&self, todo: Todo);
}