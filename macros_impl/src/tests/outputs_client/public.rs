pub trait TodoServer {
    type Error: ::core::error::Error;
    async fn get_todos(&self) -> Result<Vec<Todo>, ::trait_link::LinkError<Self::Error>>;
    async fn get_todo(&self, name: String) -> Result<Option<Todo>, ::trait_link::LinkError<Self::Error>>;
    async fn new_todo(&self, todo: Todo) -> Result<(), ::trait_link::LinkError<Self::Error>>;
}

#[derive(Debug, ::trait_link::serde::Serialize, ::trait_link::serde::Deserialize)]
#[serde(crate = "::trait_link::serde")]
pub enum TodoServerRequest {
    GetTodos(),
    GetTodo(String),
    NewTodo(Todo),
}

#[derive(Debug, ::trait_link::serde::Serialize, ::trait_link::serde::Deserialize)]
#[serde(crate = "::trait_link::serde")]
pub enum TodoServerResponse {
    GetTodos(Vec<Todo>),
    GetTodo(Option<Todo>),
    NewTodo(()),
}

pub struct TodoServerClient<T: ::trait_link::Transport>(T);

impl<T: ::trait_link::Transport> TodoServerClient<T> {
    pub fn new(transport: T) -> Self {
        Self(transport)
    }
}

impl<T: ::trait_link::Transport> TodoServer for TodoServerClient<T> {
    type Error = <T as ::trait_link::Transport>::Error;
    async fn get_todos(&self) -> Result<Vec<Todo>, ::trait_link::LinkError<Self::Error>> {
        if let TodoServerResponse::GetTodos(value) =
            self.0.send(TodoServerRequest::GetTodos()).await?
        {
            Ok(value)
        } else {
            Err(::trait_link::LinkError::<Self::Error>::WrongResponseType)
        }
    }
    async fn get_todo(&self, name: String) -> Result<Option<Todo>, ::trait_link::LinkError<Self::Error>> {
        if let TodoServerResponse::GetTodo(value) =
            self.0.send(TodoServerRequest::GetTodo(name)).await?
        {
            Ok(value)
        } else {
            Err(::trait_link::LinkError::<Self::Error>::WrongResponseType)
        }
    }
    async fn new_todo(&self, todo: Todo) -> Result<(), ::trait_link::LinkError<Self::Error>> {
        if let TodoServerResponse::NewTodo(value) =
            self.0.send(TodoServerRequest::NewTodo(todo)).await?
        {
            Ok(value)
        } else {
            Err(::trait_link::LinkError::<Self::Error>::WrongResponseType)
        }
    }
}
