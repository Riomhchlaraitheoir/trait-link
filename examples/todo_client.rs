use trait_link::reqwest::Client;
use trait_link::*;

include!("traits/todo.rs");

#[tokio::main]
async fn main() {
    let client = Client::new("http://localhost:8080/api/todo");
    let client = TodoService::client(&client);
    for todo in client.get_todos().await.expect("get_todos failed") {
        println!("{todo:?}")
    }
    if let Some(todo) = client.get_todo("next".to_string()).await.expect("get_todo failed") {
        println!("{todo:?}")
    }
    client.new_todo(Todo {
        name: "Some task".to_string(),
        description: "A description of the task".to_string(),
    }).await.expect("new_todo failed");
}
