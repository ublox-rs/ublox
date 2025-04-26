use crate::types::{UbxEnumRestHandling, UbxExtendEnum, UbxTypeFromFn, UbxTypeIntoFn};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{parse_quote, Type};

pub(crate) fn generate_code_to_extend_enum(ubx_enum: &UbxExtendEnum) -> TokenStream {
    assert_eq!(ubx_enum.repr, {
        let ty: Type = parse_quote! { u8 };
        ty
    });
    let name = &ubx_enum.name;
    let mut variants = ubx_enum.variants.clone();
    let attrs = &ubx_enum.attrs;
    if let Some(UbxEnumRestHandling::Reserved) = ubx_enum.rest_handling {
        let defined: HashSet<u8> = ubx_enum.variants.iter().map(|x| x.1).collect();
        for i in 0..=u8::MAX {
            if !defined.contains(&i) {
                let name = format_ident!("Reserved{}", i);
                variants.push((name, i));
            }
        }
    }
    let repr_ty = &ubx_enum.repr;
    let from_code = match ubx_enum.from_fn {
        Some(UbxTypeFromFn::From) => {
            assert_ne!(
                Some(UbxEnumRestHandling::ErrorProne),
                ubx_enum.rest_handling
            );
            let mut match_branches = Vec::with_capacity(variants.len());
            for (id, val) in &variants {
                match_branches.push(quote! { #val => #name :: #id });
            }

            quote! {
                impl #name {
                    fn from(x: #repr_ty) -> Self {
                        match x {
                            #(#match_branches),*
                        }
                    }
                }
            }
        },
        Some(UbxTypeFromFn::FromUnchecked) => {
            assert_ne!(Some(UbxEnumRestHandling::Reserved), ubx_enum.rest_handling);
            let mut match_branches = Vec::with_capacity(variants.len());
            for (id, val) in &variants {
                match_branches.push(quote! { #val => #name :: #id });
            }

            let mut values = Vec::with_capacity(variants.len());
            for (i, (_, val)) in variants.iter().enumerate() {
                if i != 0 {
                    values.push(quote! { | #val });
                } else {
                    values.push(quote! { #val });
                }
            }

            quote! {
                impl #name {
                    fn from_unchecked(x: #repr_ty) -> Self {
                        match x {
                            #(#match_branches),*,
                            _ => unreachable!(),
                        }
                    }
                    fn is_valid(x: #repr_ty) -> bool {
                        match x {
                            #(#values)* => true,
                            _ => false,
                        }
                    }
                }
            }
        },
        None => quote! {},
    };

    let to_code = match ubx_enum.into_fn {
        None => quote! {},
        Some(UbxTypeIntoFn::Raw) => quote! {
            impl #name {
                const fn into_raw(self) -> #repr_ty {
                    self as #repr_ty
                }
            }
        },
    };

    let mut enum_variants = Vec::with_capacity(variants.len());
    for (id, val) in &variants {
        enum_variants.push(quote! { #id = #val });
    }

    let code = quote! {
        #(#attrs)*
        pub enum #name {
            #(#enum_variants),*
        }

        #from_code
        #to_code

        #[cfg(feature = "serde")]
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_u8(*self as u8)
            }
        }
    };
    code
}
