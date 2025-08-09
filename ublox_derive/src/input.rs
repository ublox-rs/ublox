use crate::types::packetflag::PacketFlag;
use crate::types::recvpackets::RecvPackets;
use crate::types::{PackDesc, UbxExtendEnum};
use proc_macro2::TokenStream;

use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, Attribute, Error, Fields, Generics,
    Ident, Token, Type,
};

pub(crate) mod bitflags;
pub(crate) mod keyword;
pub(crate) mod packfieldmap;
mod util;

pub fn parse_packet_description(
    struct_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
    generics: Generics,
) -> syn::Result<PackDesc> {
    let main_sp = struct_name.span();

    let header = util::parse_ubx_attr(&attrs, &struct_name)?;
    let struct_comment = util::extract_item_comment(&attrs)?;

    let name = struct_name.to_string();
    let fields = util::parse_fields(fields)?;

    if let Some(field) = fields
        .iter()
        .rev()
        .skip(1)
        .find(|f| f.size_bytes.is_none() && f.size_fn().is_none())
    {
        return Err(Error::new(
            field.name.span(),
            "Non-finite size for field which is not the last field",
        ));
    }

    let ret = PackDesc {
        name,
        header,
        comment: struct_comment,
        fields,
        generics,
    };

    if ret.header.payload_len.fixed().map(usize::from) == ret.packet_payload_size() {
        Ok(ret)
    } else {
        Err(Error::new(
            main_sp,
            format!(
                "Calculated packet size ({:?}) doesn't match specified ({:?})",
                ret.packet_payload_size(),
                ret.header.payload_len
            ),
        ))
    }
}

pub fn parse_ubx_enum_type(
    enum_name: Ident,
    attrs: Vec<Attribute>,
    in_variants: Punctuated<syn::Variant, syn::token::Comma>,
) -> syn::Result<UbxExtendEnum> {
    let (from_fn, into_fn, rest_handling) =
        util::parse_ubx_extend_attrs("#[ubx_extend]", enum_name.span(), &attrs)?;

    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("repr"))
        .ok_or_else(|| {
            Error::new(
                enum_name.span(),
                format!("No repr attribute for ubx_type enum {enum_name}"),
            )
        })?;
    let meta = attr.parse_meta()?;
    let repr: Type = match meta {
        syn::Meta::List(list) if list.nested.len() == 1 => {
            if let syn::NestedMeta::Meta(syn::Meta::Path(ref p)) = list.nested[0] {
                if !p.is_ident("u8") {
                    unimplemented!();
                }
            } else {
                return Err(Error::new(
                    list.nested[0].span(),
                    "Invalid repr attribute for ubx_type enum",
                ));
            }
            syn::parse_quote! { u8 }
        },
        _ => {
            return Err(Error::new(
                attr.span(),
                "Invalid repr attribute for ubx_type enum",
            ))
        },
    };
    let mut variants = Vec::with_capacity(in_variants.len());
    for var in in_variants {
        if syn::Fields::Unit != var.fields {
            return Err(Error::new(
                var.fields.span(),
                "Invalid variant for ubx_type enum",
            ));
        }
        let var_sp = var.ident.span();
        let (_, expr) = var
            .discriminant
            .ok_or_else(|| Error::new(var_sp, "ubx_type enum variant should has value"))?;
        let variant_value = if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(litint),
            ..
        }) = expr
        {
            litint.base10_parse::<u8>()?
        } else {
            return Err(Error::new(
                expr.span(),
                "Invalid variant value for ubx_type enum",
            ));
        };
        variants.push((var.ident, variant_value));
    }

    let attrs = attrs
        .into_iter()
        .filter(|x| !x.path.is_ident("ubx") && !x.path.is_ident("ubx_extend"))
        .collect();

    Ok(UbxExtendEnum {
        attrs,
        name: enum_name,
        repr,
        from_fn,
        into_fn,
        rest_handling,
        variants,
    })
}

pub fn parse_idents_list(input: proc_macro2::TokenStream) -> syn::Result<RecvPackets> {
    syn::parse2(input)
}

pub struct MapType {
    pub ty: Type,
    pub from_fn: Option<TokenStream>,
    pub is_valid_fn: Option<TokenStream>,
    pub into_fn: Option<TokenStream>,
    pub size_fn: Option<TokenStream>,
}

#[allow(dead_code)]
struct Comment(String);

impl Parse for Comment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![#]) && input.peek2(syn::token::Bracket) && input.peek3(Ident) {
            let attrs = input.call(Attribute::parse_outer)?;

            Ok(Comment(util::extract_item_comment(&attrs)?))
        } else {
            Ok(Comment(String::new()))
        }
    }
}

struct StructFlags(Punctuated<PacketFlag, Token![,]>);

impl Parse for StructFlags {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let flags = input.parse_terminated(PacketFlag::parse)?;
        Ok(Self(flags))
    }
}
