//! Code generation for the `match_packet` functions

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_fn_match_packet(
    union_enum_name_ref: &Ident,
    matches_ref: &[TokenStream],
    unknown_var_ref: &Ident,
) -> TokenStream {
    quote! {
        pub(crate) fn match_packet(class: u8, msg_id: u8, payload: &[u8]) -> Result<#union_enum_name_ref, ParserError> {
            match (class, msg_id) {
                #(#matches_ref)*
                _ => Ok(#union_enum_name_ref::Unknown(#unknown_var_ref {
                    payload,
                    class,
                    msg_id
                })),
            }
        }
    }
}

pub(super) fn generate_fn_match_packet_owned(
    union_enum_name_owned: &Ident,
    matches_owned: &[TokenStream],
    unknown_var_owned: &Ident,
) -> TokenStream {
    quote! {
        pub(crate) fn match_packet_owned(class: u8, msg_id: u8, payload: &[u8]) -> Result<#union_enum_name_owned, ParserError> {
            match (class, msg_id) {
                #(#matches_owned)*
                _ => {
                    let mut payload = [0u8; MAX_PAYLOAD_LEN as usize];
                    let payload_len = core::cmp::min(payload.len(), MAX_PAYLOAD_LEN as usize);
                    payload[..payload_len].copy_from_slice(&payload[..payload_len]);

                    Ok(#union_enum_name_owned::Unknown(#unknown_var_owned {
                        payload,
                        payload_len,
                        class,
                        msg_id
                    }))
                }
            }
        }
    }
}
