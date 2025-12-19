use crate::output::Names;
use crate::{Link, Method};
use convert_case::ccase;
use proc_macro2::Ident;
use quote::format_ident;
use syn::{parse_quote, Field, FieldMutability, Fields, FieldsUnnamed, ItemEnum, Variant, Visibility};

impl Link {
    pub fn response_name(&self) -> Ident {
        format_ident!("{}Response", self.name)
    }

    pub fn response_enum(&self, names: &Names) -> ItemEnum {
        let serde = names.serde();
        let serde_str = serde.segments
            .iter()
            .map(|s| {
                format!("::{}", s.ident)
            })
            .collect::<String>();
        let name = self.response_name();
        let variants = self.methods.iter().map(Method::response_variant);
        let vis = &self.vis;
        parse_quote!(
            #[derive(Debug, #serde::Serialize, #serde::Deserialize)]
            #[serde(crate = #serde_str)]
            #vis enum #name {
                #(#variants),*
            }
        )
    }
}

impl Method {
    fn response_variant(&self) -> Variant {
        let name = Ident::new(&ccase!(pascal, self.name.to_string()), self.name.span());
        Variant {
            attrs: vec![],
            ident: name,
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: Default::default(),
                unnamed: [
                    Field {
                        attrs: vec![],
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: self.ret.clone(),
                    }
                ].into_iter().collect(),
            }),
            discriminant: None,
        }
    }
}