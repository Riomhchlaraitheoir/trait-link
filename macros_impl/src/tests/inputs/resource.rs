#[rpc]
pub trait Resources<T> {
    fn list(&self) -> Vec<T>;
    fn get(&self, id: usize) -> Option<T>;
    fn new(&self, value: T);
}