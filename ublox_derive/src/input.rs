use crate::types::{
    BitFlagsMacro, BitFlagsMacroItem, PackDesc, PackField, PackFieldMapDesc, PackHeader,
    PacketFlag, PayloadLen, RecvPackets, UbxEnumRestHandling, UbxExtendEnum, UbxTypeFromFn,
    UbxTypeIntoFn,
};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use std::num::NonZeroUsize;
use syn::{
    braced, parse::Parse, punctuated::Punctuated, spanned::Spanned, Attribute, Error, Fields,
    Ident, Token, Type,
};

pub fn parse_packet_description(
    struct_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> syn::Result<PackDesc> {
    let main_sp = struct_name.span();

    let header = parse_ubx_attr(&attrs, &struct_name)?;
    let struct_comment = extract_item_comment(&attrs)?;

    let name = struct_name.to_string();
    let fields = parse_fields(fields)?;

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
        parse_ubx_extend_attrs("#[ubx_extend]", enum_name.span(), &attrs)?;

    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("repr"))
        .ok_or_else(|| {
            Error::new(
                enum_name.span(),
                format!("No repr attribute for ubx_type enum {}", enum_name),
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

pub fn parse_bitflags(mac: syn::ItemMacro) -> syn::Result<BitFlagsMacro> {
    let (from_fn, into_fn, rest_handling) =
        parse_ubx_extend_attrs("#[ubx_extend_bitflags]", mac.span(), &mac.attrs)?;

    let ast: BitFlagsAst = syn::parse2(mac.mac.tokens)?;

    let valid_types: [(Type, u32); 3] = [
        (syn::parse_quote!(u8), 1),
        (syn::parse_quote!(u16), 2),
        (syn::parse_quote!(u32), 4),
    ];
    let nbits = if let Some((_ty, size)) = valid_types.iter().find(|x| x.0 == ast.repr_ty) {
        size * 8
    } else {
        let mut valid_type_names = String::with_capacity(200);
        for (t, _) in &valid_types {
            if !valid_type_names.is_empty() {
                valid_type_names.push_str(", ");
            }
            valid_type_names.push_str(&t.into_token_stream().to_string());
        }
        return Err(Error::new(
            ast.repr_ty.span(),
            format!("Not supported type, expect one of {:?}", valid_type_names),
        ));
    };

    let mut consts = Vec::with_capacity(ast.items.len());
    for item in ast.items {
        consts.push(BitFlagsMacroItem {
            attrs: item.attrs,
            name: item.name,
            value: item.value.base10_parse()?,
        });
    }

    Ok(BitFlagsMacro {
        nbits,
        vis: ast.vis,
        attrs: ast.attrs,
        name: ast.ident,
        repr_ty: ast.repr_ty,
        consts,
        from_fn,
        into_fn,
        rest_handling,
    })
}

pub fn parse_idents_list(input: proc_macro2::TokenStream) -> syn::Result<RecvPackets> {
    syn::parse2(input)
}

fn parse_ubx_extend_attrs(
    ubx_extend_name: &str,
    item_sp: Span,
    attrs: &[Attribute],
) -> syn::Result<(
    Option<UbxTypeFromFn>,
    Option<UbxTypeIntoFn>,
    Option<UbxEnumRestHandling>,
)> {
    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("ubx"))
        .ok_or_else(|| Error::new(item_sp, format!("No ubx attribute for {}", ubx_extend_name)))?;
    let meta = attr.parse_meta()?;
    let mut from_fn = None;
    let mut rest_handling = None;
    let mut into_fn = None;
    let meta_sp = meta.span();
    match meta {
        syn::Meta::List(list) => {
            for item in list.nested {
                if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = item {
                    if p.is_ident("from") {
                        from_fn = Some(UbxTypeFromFn::From);
                    } else if p.is_ident("into_raw") {
                        into_fn = Some(UbxTypeIntoFn::Raw);
                    } else if p.is_ident("from_unchecked") {
                        from_fn = Some(UbxTypeFromFn::FromUnchecked);
                    } else if p.is_ident("rest_reserved") || p.is_ident("rest_error") {
                        if rest_handling.is_some() {
                            return Err(Error::new(
                                p.span(),
                                "rest_reserved or rest_error already defined",
                            ));
                        }

                        rest_handling = Some(if p.is_ident("rest_reserved") {
                            UbxEnumRestHandling::Reserved
                        } else {
                            UbxEnumRestHandling::ErrorProne
                        });
                    } else {
                        return Err(Error::new(p.span(), "Invalid ubx attribute"));
                    }
                } else {
                    return Err(Error::new(item.span(), "Invalid ubx attribute"));
                }
            }
        },
        _ => return Err(Error::new(attr.span(), "Invalid ubx attributes")),
    }

    if from_fn == Some(UbxTypeFromFn::From)
        && rest_handling == Some(UbxEnumRestHandling::ErrorProne)
    {
        return Err(Error::new(
            meta_sp,
            "you should use rest_error with from_unchecked",
        ));
    }

    Ok((from_fn, into_fn, rest_handling))
}

fn parse_ubx_attr(attrs: &[Attribute], struct_name: &Ident) -> syn::Result<PackHeader> {
    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("ubx"))
        .ok_or_else(|| {
            Error::new(
                struct_name.span(),
                format!("No ubx attribute for struct {}", struct_name),
            )
        })?;
    let meta = attr.parse_meta()?;
    let meta = match meta {
        syn::Meta::List(x) => x,
        _ => return Err(Error::new(meta.span(), "Invalid ubx attribute syntax")),
    };

    let mut class = None;
    let mut id = None;
    let mut fixed_payload_len = None;
    let mut flags = Vec::new();
    let mut max_payload_len = None;

    for e in &meta.nested {
        match e {
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path, lit, ..
            })) => {
                if path.is_ident("class") {
                    if class.is_some() {
                        return Err(Error::new(e.span(), "Duplicate \"class\" attribute"));
                    }
                    class = match lit {
                        syn::Lit::Int(x) => Some(x.base10_parse::<u8>()?),
                        _ => return Err(Error::new(lit.span(), "Should be integer literal")),
                    };
                } else if path.is_ident("id") {
                    if id.is_some() {
                        return Err(Error::new(e.span(), "Duplicate \"id\" attribute"));
                    }
                    id = match lit {
                        syn::Lit::Int(x) => Some(x.base10_parse::<u8>()?),
                        _ => return Err(Error::new(lit.span(), "Should be integer literal")),
                    };
                } else if path.is_ident("fixed_payload_len") {
                    if fixed_payload_len.is_some() {
                        return Err(Error::new(
                            e.span(),
                            "Duplicate \"fixed_payload_len\" attribute",
                        ));
                    }
                    fixed_payload_len = match lit {
                        syn::Lit::Int(x) => Some(x.base10_parse::<u16>()?),
                        _ => return Err(Error::new(lit.span(), "Should be integer literal")),
                    };
                } else if path.is_ident("max_payload_len") {
                    if max_payload_len.is_some() {
                        return Err(Error::new(
                            e.span(),
                            "Duplicate \"max_payload_len\" attribute",
                        ));
                    }
                    max_payload_len = match lit {
                        syn::Lit::Int(x) => Some(x.base10_parse::<u16>()?),
                        _ => return Err(Error::new(lit.span(), "Should be integer literal")),
                    };
                } else if path.is_ident("flags") {
                    if !flags.is_empty() {
                        return Err(Error::new(path.span(), "Duplicate flags"));
                    }
                    let my_flags = match lit {
                        syn::Lit::Str(x) => x.parse::<StructFlags>()?,
                        _ => return Err(Error::new(lit.span(), "Should be string literal")),
                    };
                    flags = my_flags.0.into_iter().collect();
                } else {
                    return Err(Error::new(path.span(), "Unsupported attribute"));
                }
            },
            _ => return Err(Error::new(e.span(), "Unsupported attribute")),
        }
    }
    let class = class.ok_or_else(|| Error::new(meta.span(), "No \"class\" attribute"))?;
    let id = id.ok_or_else(|| Error::new(meta.span(), "No \"id\" attribute"))?;

    let payload_len = match (max_payload_len, fixed_payload_len) {
        (Some(x), None) => PayloadLen::Max(x),
        (None, Some(x)) => PayloadLen::Fixed(x),
        (Some(_), Some(_)) => {
            return Err(Error::new(
                meta.span(),
                "You should not note max_payload_len AND fixed_payload_len",
            ))
        },
        (None, None) => {
            return Err(Error::new(
                meta.span(),
                "You should note max_payload_len or fixed_payload_len",
            ))
        },
    };

    Ok(PackHeader {
        class,
        id,
        payload_len,
        flags,
    })
}

fn extract_item_comment(attrs: &[Attribute]) -> syn::Result<String> {
    let mut doc_comments = String::new();
    for a in attrs {
        if a.path.is_ident("doc") {
            let meta = a.parse_meta()?;
            match meta {
                syn::Meta::NameValue(syn::MetaNameValue { lit, .. }) => {
                    let lit = match lit {
                        syn::Lit::Str(s) => s,
                        _ => return Err(Error::new(lit.span(), "Invalid comment")),
                    };
                    doc_comments.push_str(&lit.value());
                },
                _ => return Err(Error::new(a.span(), "Invalid comments")),
            }
        }
    }
    Ok(doc_comments)
}

fn parse_fields(fields: Fields) -> syn::Result<Vec<PackField>> {
    let fields = match fields {
        syn::Fields::Named(x) => x,
        _ => {
            return Err(Error::new(fields.span(), "Unsupported fields format"));
        },
    };
    let mut ret = Vec::with_capacity(fields.named.len());
    for f in fields.named {
        let f_sp = f.span();
        let syn::Field {
            ident: name,
            attrs,
            ty,
            ..
        } = f;
        let size_bytes = field_size_bytes(&ty)?;

        let name = name.ok_or_else(|| Error::new(f_sp, "No field name"))?;
        let comment = extract_item_comment(&attrs)?;
        let mut map = PackFieldMap::default();
        for a in attrs {
            if !a.path.is_ident("doc") {
                if !map.is_none() {
                    return Err(Error::new(
                        a.span(),
                        "Two map attributes for the same field",
                    ));
                }
                map = a.parse_args::<PackFieldMap>()?;
            }
        }

        if let Some(ref map_ty) = map.map_type {
            if map_ty.ty == ty {
                return Err(Error::new(
                    map_ty.ty.span(),
                    "You map type to the same type",
                ));
            }
        }

        let map = PackFieldMapDesc::new(map, &ty);

        ret.push(PackField {
            name,
            ty,
            map,
            comment,
            size_bytes,
        });
    }

    Ok(ret)
}

mod kw {
    syn::custom_keyword!(map_type);
    syn::custom_keyword!(scale);
    syn::custom_keyword!(alias);
    syn::custom_keyword!(default_for_builder);
    syn::custom_keyword!(may_fail);
    syn::custom_keyword!(from);
    syn::custom_keyword!(is_valid);
    syn::custom_keyword!(get_as_ref);
    syn::custom_keyword!(into);
    syn::custom_keyword!(size_fn);
}

#[derive(Default)]
pub struct PackFieldMap {
    pub map_type: Option<MapType>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
    pub convert_may_fail: bool,
    pub get_as_ref: bool,
}

impl PackFieldMap {
    fn is_none(&self) -> bool {
        self.map_type.is_none() && self.scale.is_none() && self.alias.is_none()
    }
}

pub struct MapType {
    pub ty: Type,
    pub from_fn: Option<TokenStream>,
    pub is_valid_fn: Option<TokenStream>,
    pub into_fn: Option<TokenStream>,
    pub size_fn: Option<TokenStream>,
}

impl Parse for PackFieldMap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut map = PackFieldMap::default();
        let mut map_ty = None;
        let mut custom_from_fn: Option<syn::Path> = None;
        let mut custom_into_fn: Option<syn::Expr> = None;
        let mut custom_is_valid_fn: Option<syn::Path> = None;
        let mut custom_size_fn: Option<syn::Path> = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::map_type) {
                input.parse::<kw::map_type>()?;
                input.parse::<Token![=]>()?;
                map_ty = Some(input.parse()?);
            } else if lookahead.peek(kw::scale) {
                input.parse::<kw::scale>()?;
                input.parse::<Token![=]>()?;
                map.scale = Some(input.parse()?);
            } else if lookahead.peek(kw::alias) {
                input.parse::<kw::alias>()?;
                input.parse::<Token![=]>()?;
                map.alias = Some(input.parse()?);
            } else if lookahead.peek(kw::may_fail) {
                input.parse::<kw::may_fail>()?;
                map.convert_may_fail = true;
            } else if lookahead.peek(kw::from) {
                input.parse::<kw::from>()?;
                input.parse::<Token![=]>()?;
                custom_from_fn = Some(input.parse()?);
            } else if lookahead.peek(kw::is_valid) {
                input.parse::<kw::is_valid>()?;
                input.parse::<Token![=]>()?;
                custom_is_valid_fn = Some(input.parse()?);
            } else if lookahead.peek(kw::size_fn) {
                input.parse::<kw::size_fn>()?;
                input.parse::<Token![=]>()?;
                custom_size_fn = Some(input.parse()?);
            } else if lookahead.peek(kw::get_as_ref) {
                input.parse::<kw::get_as_ref>()?;
                map.get_as_ref = true;
            } else if lookahead.peek(kw::into) {
                input.parse::<kw::into>()?;
                input.parse::<Token![=]>()?;
                custom_into_fn = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if let Some(map_ty) = map_ty {
            map.map_type = Some(MapType {
                ty: map_ty,
                from_fn: custom_from_fn.map(ToTokens::into_token_stream),
                is_valid_fn: custom_is_valid_fn.map(ToTokens::into_token_stream),
                into_fn: custom_into_fn.map(ToTokens::into_token_stream),
                size_fn: custom_size_fn.map(ToTokens::into_token_stream),
            });
        }

        Ok(map)
    }
}

struct Comment(String);

impl Parse for Comment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![#]) && input.peek2(syn::token::Bracket) && input.peek3(Ident) {
            let attrs = input.call(Attribute::parse_outer)?;

            Ok(Comment(extract_item_comment(&attrs)?))
        } else {
            Ok(Comment(String::new()))
        }
    }
}

fn field_size_bytes(ty: &Type) -> syn::Result<Option<NonZeroUsize>> {
    let valid_types: [(Type, NonZeroUsize); 8] = [
        (syn::parse_quote!(u8), NonZeroUsize::new(1).unwrap()),
        (syn::parse_quote!(i8), NonZeroUsize::new(1).unwrap()),
        (syn::parse_quote!(u16), NonZeroUsize::new(2).unwrap()),
        (syn::parse_quote!(i16), NonZeroUsize::new(2).unwrap()),
        (syn::parse_quote!(u32), NonZeroUsize::new(4).unwrap()),
        (syn::parse_quote!(i32), NonZeroUsize::new(4).unwrap()),
        (syn::parse_quote!(f32), NonZeroUsize::new(4).unwrap()),
        (syn::parse_quote!(f64), NonZeroUsize::new(8).unwrap()),
    ];
    if let Some((_ty, size)) = valid_types.iter().find(|x| x.0 == *ty) {
        Ok(Some(*size))
    } else if let syn::Type::Array(ref fixed_array) = ty {
        if *fixed_array.elem != syn::parse_quote!(u8) {
            return Err(Error::new(fixed_array.elem.span(), "Only u8 supported"));
        }
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(ref len),
            ..
        }) = fixed_array.len
        {
            let len_val: usize = len.base10_parse()?;
            Ok(NonZeroUsize::new(len_val))
        } else {
            Err(Error::new(
                fixed_array.len.span(),
                "Can not interpret array length",
            ))
        }
    } else if let syn::Type::Reference(_) = ty {
        Ok(None)
    } else {
        let mut valid_type_names = String::with_capacity(200);
        for (t, _) in &valid_types {
            if !valid_type_names.is_empty() {
                valid_type_names.push_str(", ");
            }
            valid_type_names.push_str(&t.into_token_stream().to_string());
        }
        Err(Error::new(
            ty.span(),
            format!("Not supported type, expect one of {:?}", valid_type_names),
        ))
    }
}

struct BitFlagsAst {
    attrs: Vec<Attribute>,
    vis: syn::Visibility,
    _struct_token: Token![struct],
    ident: Ident,
    _colon_token: Token![:],
    repr_ty: Type,
    _brace_token: syn::token::Brace,
    items: Punctuated<BitFlagsAstConst, Token![;]>,
}

struct BitFlagsAstConst {
    attrs: Vec<Attribute>,
    _const_token: Token![const],
    name: Ident,
    _eq_token: Token![=],
    value: syn::LitInt,
}

impl Parse for BitFlagsAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let ident = input.parse()?;
        let colon_token = input.parse()?;
        let repr_ty = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let items = content.parse_terminated(BitFlagsAstConst::parse)?;
        Ok(Self {
            attrs,
            vis,
            _struct_token: struct_token,
            ident,
            _colon_token: colon_token,
            repr_ty,
            _brace_token: brace_token,
            items,
        })
    }
}

impl Parse for BitFlagsAstConst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            _const_token: input.parse()?,
            name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for PacketFlag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::default_for_builder) {
            input.parse::<kw::default_for_builder>()?;
            Ok(PacketFlag::DefaultForBuilder)
        } else {
            Err(lookahead.error())
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

impl Parse for RecvPackets {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let union_enum_name: Ident = input.parse()?;
        let content;
        let _brace_token: syn::token::Brace = braced!(content in input);
        content.parse::<Token![_]>()?;
        content.parse::<Token![=]>()?;
        let unknown_ty: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let packs: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse)?;
        Ok(Self {
            union_enum_name,
            unknown_ty,
            all_packets: packs.into_iter().collect(),
        })
    }
}
