//! Code generation for the `match_packet` functions

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_fn_match_packet(
    union_enum_name: &Ident,
    matches: &[TokenStream],
    unknown_var: &Ident,
) -> TokenStream {
    quote! {
        pub(crate) fn match_packet(class: u8, msg_id: u8, payload: &[u8]) -> Result<#union_enum_name, ParserError> {
            match (class, msg_id) {
                #(#matches)*
                _ => Ok(#union_enum_name::Unknown(#unknown_var {
                    buffer: PacketBuffer::Borrowed(payload),
                    class,
                    msg_id,
                    payload_len: payload.len(),
                })),
            }
        }
    }
}
