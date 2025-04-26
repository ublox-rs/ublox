use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Type};

#[derive(Debug)]
pub struct PackFieldMapDesc {
    pub map_type: Option<MapTypeDesc>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
    pub convert_may_fail: bool,
    pub get_as_ref: bool,
}

#[derive(Debug)]
pub struct MapTypeDesc {
    pub ty: Type,
    pub from_fn: TokenStream,
    pub is_valid_fn: TokenStream,
    pub into_fn: TokenStream,
    pub size_fn: Option<TokenStream>,
}

impl PackFieldMapDesc {
    pub fn new(x: crate::input::PackFieldMap, raw_ty: &Type) -> Self {
        let convert_may_fail = x.convert_may_fail;
        let scale_back = x.scale.as_ref().map(|x| quote! { 1. / #x });
        let map_type = x.map_type.map(|map_type| {
            let ty = map_type.ty;
            let from_fn = map_type.from_fn.unwrap_or_else(|| {
                if !convert_may_fail {
                    quote! { <#ty>::from }
                } else {
                    quote! { <#ty>::from_unchecked }
                }
            });

            let is_valid_fn = map_type.is_valid_fn.unwrap_or_else(|| {
                quote! { <#ty>::is_valid }
            });

            let into_fn = map_type.into_fn.unwrap_or_else(|| {
                if ty == syn::parse_quote! {f32} || ty == syn::parse_quote! {f64} {
                    if let Some(scale_back) = scale_back {
                        let conv_method =
                            quote::format_ident!("as_{}", raw_ty.into_token_stream().to_string());

                        return quote! {
                            ScaleBack::<#ty>(#scale_back).#conv_method
                        };
                    }
                }

                quote! { <#ty>::into_raw }
            });

            MapTypeDesc {
                ty,
                from_fn,
                is_valid_fn,
                into_fn,
                size_fn: map_type.size_fn,
            }
        });
        Self {
            map_type,
            scale: x.scale,
            alias: x.alias,
            convert_may_fail: x.convert_may_fail,
            get_as_ref: x.get_as_ref,
        }
    }
}
