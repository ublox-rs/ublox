extern crate proc_macro;

mod input;
mod output;
#[cfg(test)]
mod tests;
mod types;

use proc_macro2::TokenStream;
use quote::ToTokens;

use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DeriveInput,
    Fields, Ident, Type, Variant,
};

#[proc_macro_attribute]
pub fn ubx_packet_recv(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ret = if let Data::Struct(data) = input.data {
        generate_code_for_recv_packet(input.ident, input.attrs, data.fields)
    } else {
        Err(syn::Error::new(
            input.ident.span(),
            "This attribute can only be used for struct",
        ))
    };

    ret.map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn ubx_packet_send(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ret = if let Data::Struct(data) = input.data {
        generate_code_for_send_packet(input.ident, input.attrs, data.fields)
    } else {
        Err(syn::Error::new(
            input.ident.span(),
            "This attribute can only be used for struct",
        ))
    };

    ret.map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn ubx_packet_recv_send(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ret = if let Data::Struct(data) = input.data {
        generate_code_for_recv_send_packet(input.ident, input.attrs, data.fields)
    } else {
        Err(syn::Error::new(
            input.ident.span(),
            "This attribute can only be used for struct",
        ))
    };

    ret.map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn ubx_extend(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ret = if let Data::Enum(data) = input.data {
        extend_enum(input.ident, input.attrs, data.variants)
    } else {
        Err(syn::Error::new(
            input.ident.span(),
            "This attribute can only be used for enum",
        ))
    };

    ret.map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn ubx_extend_bitflags(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::ItemMacro);

    extend_bitflags(input)
        .map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro]
pub fn define_recv_packets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_define_recv_packets(input.into())
        .map(|x| x.into())
        .unwrap_or_else(|err| err.to_compile_error().into())
}

fn generate_code_for_recv_packet(
    pack_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> syn::Result<TokenStream> {
    let pack_desc = input::parse_packet_description(pack_name, attrs, fields)?;

    let mut code = output::generate_types_for_packet(&pack_desc);
    let recv_code = output::generate_recv_code_for_packet(&pack_desc);
    code.extend(recv_code);
    Ok(code)
}

fn generate_code_for_send_packet(
    pack_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> syn::Result<TokenStream> {
    let pack_desc = input::parse_packet_description(pack_name, attrs, fields)?;

    let mut code = output::generate_types_for_packet(&pack_desc);
    let send_code = output::generate_send_code_for_packet(&pack_desc);
    code.extend(send_code);
    Ok(code)
}

fn generate_code_for_recv_send_packet(
    pack_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> syn::Result<TokenStream> {
    let pack_desc = input::parse_packet_description(pack_name, attrs, fields)?;

    let mut code = output::generate_types_for_packet(&pack_desc);
    let send_code = output::generate_send_code_for_packet(&pack_desc);
    code.extend(send_code);
    let recv_code = output::generate_recv_code_for_packet(&pack_desc);
    code.extend(recv_code);
    Ok(code)
}

fn extend_enum(
    name: Ident,
    attrs: Vec<Attribute>,
    variants: Punctuated<Variant, syn::token::Comma>,
) -> syn::Result<TokenStream> {
    let ext_enum = input::parse_ubx_enum_type(name, attrs, variants)?;
    let code = output::generate_code_to_extend_enum(&ext_enum);
    Ok(code)
}

fn extend_bitflags(mac: syn::ItemMacro) -> syn::Result<TokenStream> {
    if !mac.mac.path.is_ident("bitflags") {
        return Err(syn::Error::new(
            mac.ident
                .as_ref()
                .map(|x| x.span())
                .unwrap_or_else(|| mac.span()),
            format!(
                "Expect bitflags invocation here, instead got '{}'",
                mac.mac.path.into_token_stream()
            ),
        ));
    }
    let bitflags = input::parse_bitflags(mac)?;
    output::generate_code_to_extend_bitflags(bitflags)
}

fn do_define_recv_packets(input: TokenStream) -> syn::Result<TokenStream> {
    let recv_packs = input::parse_idents_list(input)?;
    Ok(output::generate_code_for_parse(&recv_packs))
}

fn type_is_option(ty: &Type) -> bool {
    matches!(ty, Type::Path(ref typepath) if typepath.qself.is_none() && path_is_option(&typepath.path))
}

fn path_is_option(path: &syn::Path) -> bool {
    path.segments.len() == 1 && path.segments.iter().next().unwrap().ident == "Option"
}
