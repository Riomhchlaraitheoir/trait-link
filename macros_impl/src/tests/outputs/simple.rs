pub use todo_service::{
    Client as TodoServiceClient, Server as TodoServiceServer, Service as TodoService,
};

mod todo_service {
    use super::*;
    use ::trait_link::{
        LinkError, MappedTransport, Rpc, Transport,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;

    pub struct Service;

    impl Rpc for Service {
        type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
        type Request = Request;
        type Response = Response;
    }

    impl Service {
        pub fn client<_Transport: Transport<Request, Response>>(
            transport: _Transport,
        ) -> Client<_Transport> {
            Client(transport)
        }

        pub fn server<S: Server>(server: S) -> Handler<S> {
            Handler(server)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "args")]
    pub enum Request {
        #[serde(rename = "get_todos")]
        GetTodos(),
        #[serde(rename = "get_todo")]
        GetTodo(String),
        #[serde(rename = "new_todo")]
        NewTodo(Todo),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "get_todos")]
        GetTodos(Vec<Todo>),
        #[serde(rename = "get_todo")]
        GetTodo(Option<Todo>),
        #[serde(rename = "new_todo")]
        NewTodo(()),
    }

    pub trait Server {
        fn get_todos(self) -> impl Future<Output = Vec<Todo>> + Send;
        fn get_todo(self, name: String) -> impl Future<Output = Option<Todo>> + Send;
        fn new_todo(self, todo: Todo) -> impl Future<Output = ()> + Send;
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Handler<_Server: Server>(_Server);

    impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
        type Service = Service;

        async fn handle(self, request: Request) -> Response {
            match request {
                Request::GetTodos() => Response::GetTodos(self.0.get_todos().await),
                Request::GetTodo(name) => Response::GetTodo(self.0.get_todo(name).await),
                Request::NewTodo(todo) => Response::NewTodo(self.0.new_todo(todo).await),
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Client<_Transport>(_Transport);

    impl<_Transport: Transport<Request, Response>> Client<_Transport> {
        pub async fn get_todos(self) -> Result<Vec<Todo>, LinkError<_Transport::Error>> {
            if let Response::GetTodos(value) = self.0.send(Request::GetTodos()).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn get_todo(
            self,
            name: String,
        ) -> Result<Option<Todo>, LinkError<_Transport::Error>> {
            if let Response::GetTodo(value) = self.0.send(Request::GetTodo(name)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn new_todo(self, todo: Todo) -> Result<(), LinkError<_Transport::Error>> {
            if let Response::NewTodo(value) = self.0.send(Request::NewTodo(todo)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }
}
