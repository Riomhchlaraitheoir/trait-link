pub use api_service::{
    Client as ApiServiceClient, Server as ApiServiceServer, Service as ApiService,
};
mod api_service {
    use super::*;
    use ::trait_link::{
        LinkError, MappedTransport, Rpc, Transport,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;
    /// This is the [Rpc](::trait_link::Rpc) definition for this service
    pub struct Service;
    impl Rpc for Service {
        type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
        type Request = Request;
        type Response = Response;
    }
    impl Service {
        /// Create a new client, using the given underlying transport, if you wish to re-use the
        /// client for multiple calls, ensure you pass a copyable transport (eg: a reference)
        pub fn client<_Transport: Transport<Request, Response>>(
            transport: _Transport,
        ) -> Client<_Transport> {
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
    pub enum Request {
        #[serde(rename = "users")]
        Users(<UsersService as Rpc>::Request),
        #[serde(rename = "login")]
        Login(String, String),
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "users")]
        Users(<UsersService as Rpc>::Response),
        #[serde(rename = "login")]
        Login(LoginToken),
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait Server {
        fn users(
            self,
        ) -> impl Future<Output = impl ::trait_link::Handler<Service = UsersService>> + Send;
        fn login(
            self,
            username: String,
            password: String,
        ) -> impl Future<Output = LoginToken> + Send;
    }
    /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
    #[derive(Debug, Copy, Clone)]
    pub struct Handler<_Server: Server>(_Server);
    impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
        type Service = Service;
        async fn handle(self, request: Request) -> Response {
            match request {
                Request::Users(request) => {
                    let response = self.0.users().await.handle(request).await;
                    Response::Users(response)
                }
                Request::Login(username, password) => {
                    Response::Login(self.0.login(username, password).await)
                }
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
    impl<_Transport: Transport<Request, Response>> Client<_Transport> {
        pub fn users(
            self,
        ) -> <UsersService as Rpc>::Client<
            MappedTransport<_Transport, <UsersService as Rpc>::Request, Request, <UsersService as Rpc>::Response, Response, ()>,
        > {
            UsersService::client(MappedTransport::new(
                self.0,
                (),
                Self::users_to_inner,
                Self::users_to_outer,
            ))
        }
        fn users_to_inner(outer: Response) -> Option<<UsersService as Rpc>::Response> {
            if let Response::Users(inner) = outer {
                Some(inner)
            } else {
                None
            }
        }
        fn users_to_outer((): (), inner: <UsersService as Rpc>::Request) -> Request {
            Request::Users(inner)
        }
        pub async fn login(
            self,
            username: String,
            password: String,
        ) -> Result<LoginToken, LinkError<_Transport::Error>> {
            if let Response::Login(value) = self.0.send(Request::Login(username, password)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }
}
pub use users_service::{
    Client as UsersServiceClient, Server as UsersServiceServer, Service as UsersService,
};
mod users_service {
    use super::*;
    use ::trait_link::{
        LinkError, MappedTransport, Rpc, Transport,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;
    /// This is the [Rpc](::trait_link::Rpc) definition for this service
    pub struct Service;
    impl Rpc for Service {
        type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
        type Request = Request;
        type Response = Response;
    }
    impl Service {
        /// Create a new client, using the given underlying transport, if you wish to re-use the
        /// client for multiple calls, ensure you pass a copyable transport (eg: a reference)
        pub fn client<_Transport: Transport<Request, Response>>(
            transport: _Transport,
        ) -> Client<_Transport> {
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
    pub enum Request {
        #[serde(rename = "new")]
        New(User),
        #[serde(rename = "list")]
        List(),
        #[serde(rename = "by_id")]
        ById(u64, <UserService as Rpc>::Request),
        #[serde(rename = "current")]
        Current(LoginToken, <UserService as Rpc>::Request),
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "new")]
        New(User),
        #[serde(rename = "list")]
        List(Vec<User>),
        #[serde(rename = "by_id")]
        ById(<UserService as Rpc>::Response),
        #[serde(rename = "current")]
        Current(<UserService as Rpc>::Response),
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait Server {
        fn new(self, user: User) -> impl Future<Output = User> + Send;
        fn list(self) -> impl Future<Output = Vec<User>> + Send;
        fn by_id(
            self,
            id: u64,
        ) -> impl Future<Output = impl ::trait_link::Handler<Service = UserService>> + Send;
        fn current(
            self,
            token: LoginToken,
        ) -> impl Future<Output = impl ::trait_link::Handler<Service = UserService>> + Send;
    }
    /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
    #[derive(Debug, Copy, Clone)]
    pub struct Handler<_Server: Server>(_Server);
    impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
        type Service = Service;
        async fn handle(self, request: Request) -> Response {
            match request {
                Request::New(user) => Response::New(self.0.new(user).await),
                Request::List() => Response::List(self.0.list().await),
                Request::ById(id, request) => {
                    let response = self.0.by_id(id).await.handle(request).await;
                    Response::ById(response)
                }
                Request::Current(token, request) => {
                    let response = self.0.current(token).await.handle(request).await;
                    Response::Current(response)
                }
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
    impl<_Transport: Transport<Request, Response>> Client<_Transport> {
        pub async fn new(self, user: User) -> Result<User, LinkError<_Transport::Error>> {
            if let Response::New(value) = self.0.send(Request::New(user)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn list(self) -> Result<Vec<User>, LinkError<_Transport::Error>> {
            if let Response::List(value) = self.0.send(Request::List()).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub fn by_id(
            self,
            id: u64,
        ) -> <UserService as Rpc>::Client<
            MappedTransport<_Transport, <UserService as Rpc>::Request, Request, <UserService as Rpc>::Response, Response, (u64,)>
        > {
            UserService::client(MappedTransport::new(
                self.0,
                (id,),
                Self::by_id_to_inner,
                Self::by_id_to_outer,
            ))
        }
        fn by_id_to_inner(outer: Response) -> Option<<UserService as Rpc>::Response> {
            if let Response::ById(inner) = outer {
                Some(inner)
            } else {
                None
            }
        }
        fn by_id_to_outer((id,): (u64,), inner: <UserService as Rpc>::Request) -> Request {
            Request::ById(id, inner)
        }
        pub fn current(
            self,
            token: LoginToken,
        ) -> <UserService as Rpc>::Client<
            MappedTransport<_Transport, <UserService as Rpc>::Request, Request, <UserService as Rpc>::Response, Response, (LoginToken,)>
        > {
            UserService::client(MappedTransport::new(
                self.0,
                (token,),
                Self::current_to_inner,
                Self::current_to_outer,
            ))
        }
        fn current_to_inner(outer: Response) -> Option<<UserService as Rpc>::Response> {
            if let Response::Current(inner) = outer {
                Some(inner)
            } else {
                None
            }
        }
        fn current_to_outer(
            (token,): (LoginToken,),
            inner: <UserService as Rpc>::Request,
        ) -> Request {
            Request::Current(token, inner)
        }
    }
}
pub use user_service::{
    Client as UserServiceClient, Server as UserServiceServer, Service as UserService,
};
mod user_service {
    use super::*;
    use ::trait_link::{
        LinkError, MappedTransport, Rpc, Transport,
        serde::{Deserialize, Serialize},
    };
    use std::marker::PhantomData;
    /// This is the [Rpc](::trait_link::Rpc) definition for this service
    pub struct Service;
    impl Rpc for Service {
        type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
        type Request = Request;
        type Response = Response;
    }
    impl Service {
        /// Create a new client, using the given underlying transport, if you wish to re-use the
        /// client for multiple calls, ensure you pass a copyable transport (eg: a reference)
        pub fn client<_Transport: Transport<Request, Response>>(
            transport: _Transport,
        ) -> Client<_Transport> {
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
    pub enum Request {
        #[serde(rename = "get")]
        Get(),
        #[serde(rename = "update")]
        Update(User),
        #[serde(rename = "delete")]
        Delete(User),
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_link::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "get")]
        Get(Result<User, UserNotFound>),
        #[serde(rename = "update")]
        Update(Result<User, UserNotFound>),
        #[serde(rename = "delete")]
        Delete(Result<User, UserNotFound>),
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait Server {
        fn get(self) -> impl Future<Output = Result<User, UserNotFound>> + Send;
        fn update(self, user: User) -> impl Future<Output = Result<User, UserNotFound>> + Send;
        fn delete(self, user: User) -> impl Future<Output = Result<User, UserNotFound>> + Send;
    }
    /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
    #[derive(Debug, Copy, Clone)]
    pub struct Handler<_Server: Server>(_Server);
    impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
        type Service = Service;
        async fn handle(self, request: Request) -> Response {
            match request {
                Request::Get() => Response::Get(self.0.get().await),
                Request::Update(user) => Response::Update(self.0.update(user).await),
                Request::Delete(user) => Response::Delete(self.0.delete(user).await),
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
    impl<_Transport: Transport<Request, Response>> Client<_Transport> {
        pub async fn get(
            self,
        ) -> Result<Result<User, UserNotFound>, LinkError<_Transport::Error>> {
            if let Response::Get(value) = self.0.send(Request::Get()).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn update(
            self,
            user: User,
        ) -> Result<Result<User, UserNotFound>, LinkError<_Transport::Error>> {
            if let Response::Update(value) = self.0.send(Request::Update(user)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
        pub async fn delete(
            self,
            user: User,
        ) -> Result<Result<User, UserNotFound>, LinkError<_Transport::Error>> {
            if let Response::Delete(value) = self.0.send(Request::Delete(user)).await? {
                Ok(value)
            } else {
                Err(LinkError::WrongResponseType)
            }
        }
    }
}
