pub use todo_service::{
    AsyncClient as TodoServiceAsyncClient,
    BlockingClient as TodoServiceBlockingClient,
    Server as TodoServiceServer,
    Service as TodoService,
};

mod todo_service {
    use super::*;
    use ::trait_link::{
        AsyncTransport, BlockingTransport, LinkError, MappedTransport, Rpc,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;

    /// A service for managing to-do items
    ///
    /// This is the [Rpc](::trait_link::Rpc) definition for this service
    pub struct Service;

    impl Rpc for Service {
        type AsyncClient<T: AsyncTransport<Self::Request, Self::Response>> = AsyncClient<T>;
type BlockingClient<T: BlockingTransport<Self::Request, Self::Response>> = BlockingClient<T>;
        type Request = Request;
        type Response = Response;
        fn async_client<_Transport: AsyncTransport<Request, Response>>(transport: _Transport) -> AsyncClient<_Transport> {
            AsyncClient(transport)
        }
        fn blocking_client<_Transport: BlockingTransport<Request, Response>>(transport: _Transport) -> BlockingClient<_Transport> {
            BlockingClient(transport)
        }
    }

    impl Service {
        /// Create a new [Handler](trait_link::Handler) for the service
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

    /// A service for managing to-do items
    ///
    /// This is the trait which is used by the server side in order to serve the client
    pub trait Server {
        /// Get a list of to-do items
        fn get_todos(self) -> impl Future<Output = Vec<Todo>> + Send;
        /// Get a to-do item by name, returns None if no to-do item with the given name exists
        fn get_todo(self, name: String) -> impl Future<Output = Option<Todo>> + Send;
        /// Create a new to-do item
        fn new_todo(self, todo: Todo) -> impl Future<Output = ()> + Send;
    }

    /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
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

    /// A service for managing to-do items
    ///
    /// This is the async client for the service, it produces requests from method calls
    /// (including chained method calls) and sends the requests with the given
    /// [transport](::trait_link::AsyncTransport) before returning the response
    ///
    /// The return value is always wrapped in a result: `Result<T, LinkError<_Transport::Error>>` where `T` is the service return value
    #[derive(Debug, Copy, Clone)]
    pub struct AsyncClient<_Transport>(_Transport);

    impl<_Transport: AsyncTransport<Request, Response>> AsyncClient<_Transport> {
        /// Get a list of to-do items
        pub async fn get_todos(self) -> Result<Vec<Todo>, LinkError<_Transport::Error>> {
            if let Response::GetTodos(value) = self.0.send(Request::GetTodos()).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        /// Get a to-do item by name, returns None if no to-do item with the given name exists
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
        /// Create a new to-do item
        pub async fn new_todo(self, todo: Todo) -> Result<(), LinkError<_Transport::Error>> {
            if let Response::NewTodo(value) = self.0.send(Request::NewTodo(todo)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }

    /// A service for managing to-do items
    ///
    /// This is the blocking client for the service, it produces requests from method calls
    /// (including chained method calls) and sends the requests with the given
    /// [transport](::trait_link::AsyncTransport) before returning the response
    ///
    /// The return value is always wrapped in a result: `Result<T, LinkError<_Transport::Error>>` where `T` is the service return value
    #[derive(Debug, Copy, Clone)]
    pub struct BlockingClient<_Transport>(_Transport);

    impl<_Transport: BlockingTransport<Request, Response>> BlockingClient<_Transport> {
        /// Get a list of to-do items
        pub fn get_todos(self) -> Result<Vec<Todo>, LinkError<_Transport::Error>> {
            if let Response::GetTodos(value) = self.0.send(Request::GetTodos())? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        /// Get a to-do item by name, returns None if no to-do item with the given name exists
        pub fn get_todo(self, name: String) -> Result<Option<Todo>, LinkError<_Transport::Error>> {
            if let Response::GetTodo(value) = self.0.send(Request::GetTodo(name))? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        /// Create a new to-do item
        pub fn new_todo(self, todo: Todo) -> Result<(), LinkError<_Transport::Error>> {
            if let Response::NewTodo(value) = self.0.send(Request::NewTodo(todo))? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }
}
