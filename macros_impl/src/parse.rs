use crate::{Method, Rpc};
use syn::{
    FnArg, ItemTrait, ReturnType, TraitItem, TraitItemFn, Type, TypeParamBound, parse_quote,
};

pub struct Parser;

impl Parser {
    pub fn rpc(&self, input: ItemTrait) -> syn::Result<Rpc> {
        let mut methods = vec![];
        for item in input.items {
            match item {
                TraitItem::Fn(item) => methods.push(self.method(item)?),
                TraitItem::Type(_) => {}
                _ => {}
            }
        }
        if !input.supertraits.is_empty() {
            return Err(syn::Error::new_spanned(
                input.supertraits,
                "supertraits are not supported",
            ));
        }
        Ok(Rpc {
            vis: input.vis,
            generics: input.generics,
            name: input.ident,
            methods,
        })
    }

    fn method(&self, item: TraitItemFn) -> syn::Result<Method> {
        if let Some(default) = item.default {
            return Err(syn::Error::new_spanned(
                default,
                "default fn is not supported",
            ));
        }
        if let Some(con) = item.sig.constness {
            return Err(syn::Error::new_spanned(con, "const fn is not supported"));
        }
        if let Some(unsafety) = item.sig.unsafety {
            return Err(syn::Error::new_spanned(
                unsafety,
                "unsafe fn is not supported",
            ));
        }
        let name = item.sig.ident.clone();
        let mut args = Vec::with_capacity(item.sig.inputs.len() - 1);
        let mut has_self = false;
        for arg in &item.sig.inputs {
            match arg {
                FnArg::Receiver(s) => {
                    if has_self {
                        return Err(syn::Error::new_spanned(s, "cannot have multiple receivers"));
                    }
                    if s.reference.is_none() {
                        return Err(syn::Error::new_spanned(s, "cannot take owned self value"));
                    }
                    if s.mutability.is_some() {
                        return Err(syn::Error::new_spanned(
                            s,
                            "cannot take a mutable self reference",
                        ));
                    }
                    if let Type::Reference(ty) = &*s.ty
                        && ty.mutability.is_none()
                        && let Type::Path(ty) = &*ty.elem
                        && ty.path.segments.len() == 1
                        && ty.path.segments[0].ident == "Self"
                    {
                    } else {
                        return Err(syn::Error::new_spanned(
                            s,
                            "cannot use a smart pointer for self type, must use &Self",
                        ));
                    }
                    has_self = true;
                }
                FnArg::Typed(arg) => args.push(arg.clone()),
            }
        }
        if !has_self {
            return Err(syn::Error::new_spanned(item, "missing self"));
        }
        let ret = self.return_type(item.sig.output)?;
        Ok(Method { name, args, ret })
    }

    fn return_type(&self, output: ReturnType) -> syn::Result<super::ReturnType> {
        match output {
            ReturnType::Default => Ok(super::ReturnType::Simple(parse_quote! {()})),
            ReturnType::Type(_, ty) => {
                if let Type::ImplTrait(ty) = &*ty {
                    if let Some(first) = ty.bounds.first() {
                        if ty.bounds.len() > 1 {
                            return Err(syn::Error::new_spanned(
                                &ty.bounds,
                                "cannot specify multiple bounds here",
                            ));
                        }
                        if let TypeParamBound::Trait(bound) = first {
                            if bound.lifetimes.is_some() {
                                return Err(syn::Error::new_spanned(
                                    &bound.lifetimes,
                                    "lifetimes not supported here",
                                ));
                            }
                            Ok(super::ReturnType::Nested {
                                service: bound.path.clone(),
                            })
                        } else {
                            Err(syn::Error::new_spanned(ty, "unsupported bound"))
                        }
                    } else {
                        Err(syn::Error::new_spanned(ty, "no bounds found"))
                    }
                } else {
                    Ok(super::ReturnType::Simple(*ty))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Parser;
    use syn::parse_quote;
    use syn::{ReturnType, Type, TypeTuple};

    macro_rules! return_type_tests {
        ($($name:ident: $output:expr => {$($input:tt)*}),*) => {
            $(
            #[test]
            fn $name() {
                test_return_type(parse_quote!($($input)*), $output);
            }
            )*
        };
    }

    return_type_tests![
        unit: crate::ReturnType::Simple(Type::Tuple(TypeTuple { paren_token: Default::default(),elems: Default::default(),})) => {},
        simple: crate::ReturnType::Simple(Type::Path(parse_quote!(String))) => {-> String},
        service: crate::ReturnType::Nested {  service: parse_quote!(SubService) } => { -> impl SubService }
    ];

    fn test_return_type(input: ReturnType, expected: crate::ReturnType) {
        let parser = Parser;
        let output = parser.return_type(input).expect("failed to parse input");
        assert_eq!(output, expected);
    }
}
