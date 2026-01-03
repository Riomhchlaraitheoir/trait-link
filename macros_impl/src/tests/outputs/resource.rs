pub use resources::{Client as ResourcesClient, Server as ResourcesServer, Service as Resources};
mod resources {
    use super::*;
    use ::trait_link::{
        LinkError, MappedTransport, Rpc, Transport,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;
    /// This is the [Rpc](::trait_link::Rpc) definition for this service
    pub struct Service<T>(PhantomData<(T)>);
    impl<T> Rpc for Service<T> {
        type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
        type Request = Request<T>;
        type Response = Response<T>;
    }
    impl Service {
        /// Create a new client, using the given underlying transport, if you wish to re-use the
        /// client for multiple calls, ensure you pass a copyable transport (eg: a reference)
        pub fn client<_Transport: Transport<Request, Response>>(transport: _Transport) -> Client<_Transport> {
            Client(transport)
        }
        /// Create a new [Handler](trait_link::Handler) for the service
        pub fn server<S: Server>(server: S) -> Handler<S> {
            Handler(server)
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "args")]
    pub enum Request<T> {
        #[serde(rename = "list")]
        List(),
        #[serde(rename = "get")]
        Get(usize),
        #[serde(rename = "new")]
        New(T),
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response<T> {
        #[serde(rename = "list")]
        List(Vec<T>),
        #[serde(rename = "get")]
        Get(Option<T>),
        #[serde(rename = "new")]
        New(()),
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait Server<T> {
        fn list(self) -> impl Future<Output = Vec<T>> + Send;
        fn get(self, id: usize) -> impl Future<Output = Option<T>> + Send;
        fn new(self, value: T) -> impl Future<Output = ()> + Send;
    }
    /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
    #[derive(Debug, Copy, Clone)]
    pub struct Handler<_Server: Server>(_Server);
    impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
        type Service = Service;
        async fn handle(self, request: Request) -> Response {
            match request {
                Request::List() => Response::List(self.0.list().await),
                Request::Get(id) => Response::Get(self.0.get(id).await),
                Request::New(value) => Response::New(self.0.new(value).await),
            }
        }
    }
    /// This is the client for the service, it produces requests from method calls
    /// (including chained method calls) and sends the requests with the given
    /// [transport](::trait_link::Transport) before returning the response
    ///
    /// The return value is always wrapped in a result: `Result<T, LinkError<_Transport::Error>>` where `T` is the service return value
    #[derive(Debug, Copy, Clone)]
    pub struct Client<_Transport>(_Transport);
    impl<_Transport: Transport<Request, Response>, T> Client<_Transport> {
        pub async fn list(self) -> Result<Vec<T>, LinkError<_Transport::Error>> {
            if let Response::List(value) = self
                .0
                .send(Request::List())
                .await?
            {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn get(self, id: usize) -> Result<Option<T>, LinkError<_Transport::Error>> {
            if let Response::Get(value) = self
                .0
                .send(Request::Get(id))
                .await?
            {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn new(self, value: T) -> Result<(), LinkError<_Transport::Error>> {
            if let Response::New(value) = self
                .0
                .send(Request::New(value))
                .await?
            {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }
}
