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

        let imports = {
            let vis = &self.vis;
            let server = format_ident!("{}Server", service);
            let client = format_ident!("{}Client", service);
            quote!(
                #vis use #module::{Client as #client, Server as #server, Service as #service};
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
            match &method.ret {
                ReturnType::Simple(ret) => {
                    quote! {
                        fn #name(self #(,#params)*) -> impl Future<Output=#ret> + Send;
                    }
                }
                ReturnType::Nested { service: path } => {
                    quote! {
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

        let client_fns = self.methods.iter().map(|method| {
            let name = &method.name;
            let params = &method.args;
            let args = method.args.iter().map(|pat| &pat.pat);
            let variant = ident_ccase!(pascal, name);
            match &method.ret {
                ReturnType::Simple(ret) => {
                    quote! {
                        pub async fn #name(self #(, #params)*) -> Result<#ret, LinkError<_Transport::Error>> {
                            if let Response::#variant(value) = self.0
                                    .send(Request::#variant(#(#args),*))
                                    .await? {
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
                        pub fn #name(self #(, #params)*) -> <#nested as Rpc>::Client<MappedTransport<_Transport, <#nested as Rpc>::Request, Request, <#nested as Rpc>::Response, Response, (#(#types,)*)>> {
                            #nested::client(MappedTransport::new(self.0, (#(#args,)*), Self::#to_inner, Self::#to_outer))
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
        });

        quote! {
            #imports

            mod #module {
                use super::*;
                use ::trait_link::{
                    LinkError, MappedTransport, Rpc, Transport,
                    serde::{Deserialize, Serialize},
                };
                use std::marker::PhantomData;

                pub struct Service #generics #phantom_data;

                impl #generics Rpc for Service #generics {
                    type Client<T: Transport<Self::Request, Self::Response>> = Client<T>;
                    type Request = Request #generics;
                    type Response = Response #generics;
                }

                impl Service {
                    pub fn client<_Transport: Transport<Request, Response>>(transport: _Transport) -> Client<_Transport> {
                        Client(transport)
                    }

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

                pub trait Server #generics {
                    #(#server_fns)*
                }

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

                #[derive(Debug, Copy, Clone)]
                pub struct Client<_Transport>(_Transport);
                impl<_Transport: Transport<Request, Response> #(, #gen_params)*> Client<_Transport> {
                    #(#client_fns)*
                }
            }
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }
}
