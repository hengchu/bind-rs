use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::VecDeque;
use syn::{
    parse::Parse, parse_macro_input, AngleBracketedGenericArguments, Attribute, DeriveInput, Error,
    GenericArgument, Ident, Lifetime, Token, Type,
};

use syn::parse::ParseStream;

#[derive(Debug)]
struct Repr {
    type_name: Ident,
    lifetime: Lifetime,
    namespace_params: Vec<Type>,
    index: Type,
}

impl Parse for Repr {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let type_name = input.parse::<Ident>()?;
        let mut generic_args: VecDeque<GenericArgument> = input
            .parse::<AngleBracketedGenericArguments>()?
            .args
            .iter()
            .cloned()
            .collect::<VecDeque<_>>();

        let expect_lifetime = generic_args.pop_front().ok_or_else(|| {
            Error::new(
                Span::call_site(),
                "expecting a lifetime argument at the front",
            )
        })?;

        let lifetime = match expect_lifetime {
            GenericArgument::Lifetime(lt) => lt,
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "expecting a lifetime argument at the front",
                ))
            }
        };

        let expect_type = generic_args.pop_back().ok_or_else(|| {
            Error::new(Span::call_site(), "expecting a type argument at the back")
        })?;

        let index = match expect_type {
            GenericArgument::Type(ty) => ty,
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "expecting a type argument at the back",
                ))
            }
        };

        let namespace_params = generic_args
            .into_iter()
            .map(|arg| match arg {
                GenericArgument::Type(ty) => Ok(ty),
                _ => Err(Error::new(
                    Span::call_site(),
                    "expecting type arguments in the middle",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Repr {
            type_name,
            lifetime,
            namespace_params,
            index,
        })
    }
}

#[derive(Debug)]
struct Via {
    type_name: Ident,
    namespace_params: Vec<Type>,
}

impl Parse for Via {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let type_name = input.parse::<Ident>()?;
        let generic_args: Vec<GenericArgument> = input
            .parse::<AngleBracketedGenericArguments>()?
            .args
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let namespace_params = generic_args
            .into_iter()
            .map(|arg| match arg {
                GenericArgument::Type(ty) => Ok(ty),
                _ => Err(Error::new(Span::call_site(), "expecting type arguments")),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Via {
            type_name,
            namespace_params,
        })
    }
}

#[derive(Debug)]
struct DeriveAttrs {
    repr: Repr,
    via: Via,
}

impl Parse for DeriveAttrs {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut repr: Option<Repr> = None;
        let mut via: Option<Via> = None;

        {
            let attr_keyword = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;

            match attr_keyword.to_string().as_str() {
                "repr" => repr = Some(input.parse::<Repr>()?),
                "via" => via = Some(input.parse::<Via>()?),
                _ => return Err(Error::new(Span::call_site(), "expected either repr or via")),
            }
        }

        let _comma = input.parse::<Token![,]>()?;

        {
            let attr_keyword = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;

            match attr_keyword.to_string().as_str() {
                "repr" => repr = Some(input.parse::<Repr>()?),
                "via" => via = Some(input.parse::<Via>()?),
                _ => return Err(Error::new(Span::call_site(), "expected either repr or via")),
            }
        }

        match (repr, via) {
            (Some(repr), Some(via)) => Ok(DeriveAttrs { repr, via }),
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "expected both repr and via directive",
                ))
            }
        }
    }
}

fn parse_derive_attributes(attrs: &[Attribute]) -> Result<DeriveAttrs, Error> {
    for attr in attrs.iter() {
        match attr.parse_args::<DeriveAttrs>() {
            Ok(attr) => return Ok(attr),
            Err(err) => println!("{:#?}", err),
        }
    }
    Err(Error::new(
        Span::call_site(),
        "expecting #[monad(repr = ..., via = ...)] directives",
    ))
}

#[proc_macro_derive(Monad, attributes(monad))]
pub fn derive_monad(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let derive_attrs = match parse_derive_attributes(input.attrs.as_slice()) {
        Ok(attrs) => attrs,
        Err(err) => return err.to_compile_error().into(),
    };
    //println!("{:#?}", derive_attrs);

    let namespace = input.ident;
    let lifetime = derive_attrs.repr.lifetime;
    let repr_index = derive_attrs.repr.index;
    let repr_ident = derive_attrs.repr.type_name;
    let repr_type_args = derive_attrs.repr.namespace_params;
    let lifetime_bounded_repr_type_args: Vec<proc_macro2::TokenStream> = repr_type_args
        .iter()
        .cloned()
        .map(|type_arg| {
            quote! {
                #type_arg: #lifetime
            }
        })
        .collect();

    let via_namespace = derive_attrs.via.type_name;
    let via_type_args = derive_attrs.via.namespace_params;
    let via_quoted = quote! { #via_namespace::< #(#via_type_args),* > };

    let expanded = quote! {
        impl<#lifetime> Monad<#lifetime> for #namespace {
            type Repr<#repr_index : #lifetime> = #repr_ident < #lifetime, #(#repr_type_args ,)*  #repr_index >;

            fn bind_impl<A: 'a, B: 'a, F: 'a>(v: Self::Repr<A>, f: F) -> Self::Repr<B>
            where
                F: FnOnce(A) -> Self::Repr<B> + Send {
                #repr_ident(<#via_quoted as Monad<#lifetime>>::bind_impl(v.0, |a| f(a).0))
            }

            fn ret<A: 'a + Send>(v: A) -> Self::Repr<A>
            {
                #repr_ident(<#via_quoted as Monad<#lifetime>>::ret(v))
            }
        }

        impl<#lifetime, #(#lifetime_bounded_repr_type_args ,)*  #repr_index: #lifetime >
            MonadRepr<#lifetime, #namespace> for #repr_ident < #lifetime, #(#repr_type_args ,)*  #repr_index > {
                type Index = #repr_index;

                fn bind<B: 'a, F: 'a>(self, f: F) -> <#namespace as Monad<#lifetime>>::Repr<B>
                where
                    F: FnOnce(Self::Index) -> <#namespace as Monad<#lifetime>>::Repr<B> + Send {
                    <#namespace as Monad<#lifetime>>::bind_impl(self, f)
                }
        }
    };

    expanded.into()
}
