use crate::{ReturnType, Rpc};
use convert_case::ccase;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Field, FieldMutability, Visibility, parse_quote};

macro_rules! ident_ccase {
    ($case:ident, $ident:expr) => {
        Ident::new(&ccase!($case, $ident.to_string()), $ident.span())
    };
}

impl ToTokens for Rpc {
    fn to_token_stream(&self) -> TokenStream {
        let service = &self.name;
        let module = ident_ccase!(snake, service);

        let generics = &self.generics;
        let gen_params: Vec<_> = generics.params.iter().collect();
        let phantom_data = if generics.params.is_empty() {
            TokenStream::new()
        } else {
            let params = generics.params.iter();
            quote!((PhantomData<(#(#params),*)>))
        };
        let docs = if self.docs.is_empty() {
            None
        } else {
            Some(&self.docs)
        }.into_iter().collect::<Vec<_>>();

        let imports = {
            let vis = &self.vis;
            let server = format_ident!("{}Server", service);
            let async_client = format_ident!("{}AsyncClient", service);
            let blocking_client = format_ident!("{}BlockingClient", service);
            quote!(
                #vis use #module::{
                    AsyncClient as #async_client,
                    BlockingClient as #blocking_client,
                    Server as #server,
                    Service as #service
                };
            )
        };

        let request_variants = self.methods.iter().map(|method| {
            let snake_name = method.name.to_string();
            let name = ident_ccase!(pascal, method.name);
            let mut fields: Vec<_> = method
                .args
                .iter()
                .map(|pat| Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    mutability: FieldMutability::None,
                    ident: None,
                    colon_token: None,
                    ty: *pat.ty.clone(),
                })
                .collect();
            if let ReturnType::Nested {
                service: ret,
            } = &method.ret
            {
                fields.push(parse_quote! {
                    <#ret as Rpc>::Request
                })
            }
            quote!(
                #[serde(rename = #snake_name)]
                #name(#(#fields),*)
            )
        });

        let response_variants = self.methods.iter().map(|method| {
            let snake_name = method.name.to_string();
            let name = ident_ccase!(pascal, method.name);
            let ret = match &method.ret {
                ReturnType::Simple(ty) => ty.clone(),
                ReturnType::Nested {
                    service: path,
                } => {
                    parse_quote!(<#path as Rpc>::Response)
                }
            };
            quote!(
                #[serde(rename = #snake_name)]
                #name(#ret)
            )
        });

        let server_fns = self.methods.iter().map(|method| {
            let name = &method.name;
            let params = &method.args;
            let docs = &method.docs;
            let docs = quote! {
                #(#[doc = #docs])*
            };
            match &method.ret {
                ReturnType::Simple(ret) => {
                    quote! {
                        #docs
                        fn #name(self #(,#params)*) -> impl Future<Output=#ret> + Send;
                    }
                }
                ReturnType::Nested { service: path } => {
                    quote! {
                        #docs
                        fn #name(self #(,#params)*) -> impl Future<Output = impl ::trait_link::Handler<Service = #path>> + Send;
                    }
                }
            }
        });
        let handle_arms = self.methods.iter().map(|method| {
            let name = &method.name;
            let variant = ident_ccase!(pascal, method.name);
            let params = method.args.iter().map(|pat| &pat.pat).collect::<Vec<_>>();
            match &method.ret {
                ReturnType::Nested { service: _ } => {
                    quote! {
                        Request::#variant(#(#params, )*request) => {
                            let response = self.0.#name(#(#params),*).await.handle(request).await;
                            Response::#variant(response)
                        },
                    }
                }
                _ => {
                    quote! {
                    Request::#variant(#(#params),*) => Response::#variant(self.0.#name(#(#params),*).await),
                }
                }
            }
        });

        let async_client_fns = self.client_fns(true);
        let blocking_client_fns = self.client_fns(false);

        quote! {
            #imports

            mod #module {
                use super::*;
                use ::trait_link::{
                    AsyncTransport, BlockingTransport, LinkError, MappedTransport, Rpc ,
                    serde::{Deserialize, Serialize},
                };
                use std::marker::PhantomData;

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                /// This is the [Rpc](::trait_link::Rpc) definition for this service
                pub struct Service #generics #phantom_data;

                impl #generics Rpc for Service #generics {
                    type AsyncClient<T: AsyncTransport<Self::Request, Self::Response>> = AsyncClient<T>;
                    type BlockingClient<T: BlockingTransport<Self::Request, Self::Response>> = BlockingClient<T>;
                    type Request = Request #generics;
                    type Response = Response #generics;
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
                pub enum Request #generics {
                    #(#request_variants,)*
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(crate = "::trait_link::serde")]
                #[serde(tag = "method", content = "result")]
                pub enum Response #generics {
                    #(#response_variants,)*
                }

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                /// This is the trait which is used by the server side in order to serve the client
                pub trait Server #generics {
                    #(#server_fns)*
                }

                /// A [Handler](::trait_link::Handler) which handles requests/responses for a given service
                #[derive(Debug, Copy, Clone)]
                pub struct Handler<_Server: Server>(_Server);
                impl<_Server: Server + Send> ::trait_link::Handler for Handler<_Server> {
                    type Service = Service;
                    async fn handle(self, request: Request) -> Response {
                        match request {
                            #(#handle_arms)*
                        }
                    }
                }

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                /// This is the async client for the service, it produces requests from method calls
                /// (including chained method calls) and sends the requests with the given
                /// [transport](::trait_link::AsyncTransport) before returning the response
                ///
                /// The return value is always wrapped in a result: `Result<T, LinkError<_Transport::Error>>` where `T` is the service return value
                #[derive(Debug, Copy, Clone)]
                pub struct AsyncClient<_Transport>(_Transport);
                impl<_Transport: AsyncTransport<Request, Response> #(, #gen_params)*> AsyncClient<_Transport> {
                    #(#async_client_fns)*
                }

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                /// This is the blocking client for the service, it produces requests from method calls
                /// (including chained method calls) and sends the requests with the given
                /// [transport](::trait_link::AsyncTransport) before returning the response
                ///
                /// The return value is always wrapped in a result: `Result<T, LinkError<_Transport::Error>>` where `T` is the service return value
                #[derive(Debug, Copy, Clone)]
                pub struct BlockingClient<_Transport>(_Transport);
                impl<_Transport: BlockingTransport<Request, Response> #(, #gen_params)*> BlockingClient<_Transport> {
                    #(#blocking_client_fns)*
                }
            }
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }
}

impl Rpc {
    fn client_fns(&self, is_async: bool) -> impl Iterator<Item=TokenStream> {
        let await_ = if is_async {
            vec![quote!(.await)]
        } else {
            vec![]
        };
        let async_ = if is_async {
            vec![quote!(async)]
        } else {
            vec![]
        };
        self.methods.iter().map(move |method| {
            let name = &method.name;
            let params = &method.args;
            let args = method.args.iter().map(|pat| &pat.pat);
            let variant = ident_ccase!(pascal, name);
            let docs = &method.docs;
            let docs = quote! {
                #(#[doc = #docs])*
            };
            let client = if is_async {
                format_ident!("AsyncClient")
            } else {
                format_ident!("BlockingClient")
            };
            let new_client = ident_ccase!(snake, client);
            match &method.ret {
                ReturnType::Simple(ret) => {
                    quote! {
                        #docs
                        pub #(#async_)* fn #name(self #(, #params)*) -> Result<#ret, LinkError<_Transport::Error>> {
                            if let Response::#variant(value) = self.0
                                    .send(Request::#variant(#(#args),*))
                                    #(#await_)*? {
                                Ok(value)
                            } else {
                                Err(LinkError::WrongResponseType)
                            }
                        }
                    }
                }
                ReturnType::Nested { service: nested } => { // TODO account for sub-service error
                    let to_inner = format_ident!("{name}_to_inner");
                    let to_outer = format_ident!("{name}_to_outer");
                    let variant = ident_ccase!(pascal, name);
                    let args = method.args.iter().map(|pat| &pat.pat).collect::<Vec<_>>();
                    let types = method.args.iter().map(|pat| &pat.ty).collect::<Vec<_>>();
                    quote! {
                        #docs
                        pub fn #name(self #(, #params)*) -> <#nested as Rpc>::#client<MappedTransport<_Transport, <#nested as Rpc>::Request, Request, <#nested as Rpc>::Response, Response, (#(#types,)*)>> {
                            #nested::#new_client(MappedTransport::new(self.0, (#(#args,)*), Self::#to_inner, Self::#to_outer))
                        }

                        fn #to_inner(outer: Response) -> Option<<#nested as Rpc>::Response> {
                            if let Response::#variant(inner) = outer {
                                Some(inner)
                            } else {
                                None
                            }
                        }

                        fn #to_outer((#(#args,)*): (#(#types,)*), inner: <#nested as Rpc>::Request) -> Request {
                            Request::#variant(#(#args,)*inner)
                        }
                    }
                }
            }
        })
    }
}
