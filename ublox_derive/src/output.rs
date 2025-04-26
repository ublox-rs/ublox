use crate::types::BitFlagsMacro;
use crate::types::{PackDesc, PayloadLen, UbxEnumRestHandling, UbxTypeFromFn, UbxTypeIntoFn};
use crate::util::DebugContext;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{collections::HashSet, convert::TryFrom};
use syn::Ident;

pub(crate) mod extend_enum;
pub(crate) mod gen_code_for_parse;
pub(crate) mod gen_recv_code;
pub(crate) mod gen_send_code;
mod match_packet;
mod util;

pub fn generate_types_for_packet(_dbg_ctx: DebugContext, pack_descr: &PackDesc) -> TokenStream {
    let name = Ident::new(&pack_descr.name, Span::call_site());
    let class = pack_descr.header.class;
    let id = pack_descr.header.id;
    let fixed_payload_len = match pack_descr.header.payload_len.fixed() {
        Some(x) => quote! { Some(#x) },
        None => quote! { None },
    };
    let struct_comment = &pack_descr.comment;
    let max_payload_len = match pack_descr.header.payload_len {
        PayloadLen::Fixed(x) => x,
        PayloadLen::Max(x) => x,
    };
    quote! {

        #[doc = #struct_comment]
        pub struct #name;
        impl UbxPacketMeta for #name {
            const CLASS: u8 = #class;
            const ID: u8 = #id;
            const FIXED_PAYLOAD_LEN: Option<u16> = #fixed_payload_len;
            const MAX_PAYLOAD_LEN: u16 = #max_payload_len;
        }
    }
}

pub fn generate_code_to_extend_bitflags(bitflags: BitFlagsMacro) -> syn::Result<TokenStream> {
    match bitflags.rest_handling {
        Some(UbxEnumRestHandling::ErrorProne) | None => {
            return Err(syn::Error::new(
                bitflags.name.span(),
                "Only reserved supported",
            ))
        },
        Some(UbxEnumRestHandling::Reserved) => (),
    }

    let mut known_flags = HashSet::new();
    let mut items = Vec::with_capacity(usize::try_from(bitflags.nbits).unwrap());
    let repr_ty = &bitflags.repr_ty;

    for bit in 0..bitflags.nbits {
        let flag_bit = 1u64.checked_shl(bit).unwrap();
        if let Some(item) = bitflags.consts.iter().find(|x| x.value == flag_bit) {
            known_flags.insert(flag_bit);
            let name = &item.name;
            let attrs = &item.attrs;
            if bit != 0 {
                items.push(quote! {
                    #(#attrs)*
                    const #name  = ((1 as #repr_ty) << #bit)
                });
            } else {
                items.push(quote! {
                    #(#attrs)*
                    const #name  = (1 as #repr_ty)
                });
            }
        } else {
            let name = format_ident!("RESERVED{}", bit);
            if bit != 0 {
                items.push(quote! { const #name = ((1 as #repr_ty) << #bit)});
            } else {
                items.push(quote! { const #name = (1 as #repr_ty) });
            }
        }
    }

    if known_flags.len() != bitflags.consts.len() {
        let user_flags: HashSet<_> = bitflags.consts.iter().map(|x| x.value).collect();
        let set = user_flags.difference(&known_flags);
        return Err(syn::Error::new(
            bitflags.name.span(),
            format!("Strange flags, not power of 2?: {:?}", set),
        ));
    }

    let vis = &bitflags.vis;
    let attrs = &bitflags.attrs;
    let name = &bitflags.name;

    let from = match bitflags.from_fn {
        None => quote! {},
        Some(UbxTypeFromFn::From) => quote! {
            impl #name {
                const fn from(x: #repr_ty) -> Self {
                    Self::from_bits_truncate(x)
                }
            }
        },
        Some(UbxTypeFromFn::FromUnchecked) => unimplemented!(),
    };

    let into = match bitflags.into_fn {
        None => quote! {},
        Some(UbxTypeIntoFn::Raw) => quote! {
            impl #name {
                const fn into_raw(self) -> #repr_ty {
                    self.bits()
                }
            }
        },
    };

    let serialize_fn = format_ident!("serialize_{}", repr_ty.to_token_stream().to_string());
    let serde = quote! {
        #[cfg(feature = "serde")]
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.#serialize_fn(self.bits())
            }
        }
    };

    Ok(quote! {
        bitflags! {
            #(#attrs)*
            #vis struct #name : #repr_ty {
                #(#items);*;
            }
        }
        #from
        #into
        #serde
    })
}
