use crate::types::{
    PackDesc, PackField, PackFieldMap, PackHeader, UbxEnum, UbxEnumRestHandling, UbxTypeFromFn,
};
use log::trace;
use std::num::NonZeroUsize;
use syn::{parse::Parse, spanned::Spanned, Attribute, Error, Ident, Token, Type};

pub fn parse_packet_description(input: syn::ItemStruct) -> syn::Result<PackDesc> {
    let struct_name = &input.ident;
    let main_sp = input.span();

    let header = parse_ubx_attr(&input.attrs, &struct_name)?;
    let struct_comment = extract_item_comment(&input.attrs)?;

    let name = struct_name.to_string();
    let fields = parse_fields(input)?;

    let ret = PackDesc {
        name,
        header,
        comment: struct_comment,
        fields,
    };

    if ret.header.fixed_payload_len.map(usize::from) == ret.packet_payload_size() {
        Ok(ret)
    } else {
        Err(Error::new(
            main_sp,
            format!(
                "Calculated packet size ({:?}) doesn't match specified ({:?})",
                ret.packet_payload_size(),
                ret.header.fixed_payload_len
            ),
        ))
    }
}

pub fn parse_ubx_enum_type(input: syn::ItemEnum) -> syn::Result<UbxEnum> {
    let enum_name = &input.ident;
    let attr = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("ubx"))
        .ok_or_else(|| {
            Error::new(
                enum_name.span(),
                format!("No ubx attribute for ubx_type enum {}", enum_name),
            )
        })?;
    let meta = attr.parse_meta()?;
    trace!("parse_ubx_enum_type: ubx_type meta {:?}", meta);
    let mut from_fn = None;
    let mut to_fn = false;
    let mut rest_handling = None;
    match meta {
        syn::Meta::List(list) => {
            for item in list.nested {
                if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = item {
                    if p.is_ident("from") {
                        from_fn = Some(UbxTypeFromFn::From);
                    } else if p.is_ident("to") {
                        to_fn = true;
                    } else if p.is_ident("from_unchecked") {
                        from_fn = Some(UbxTypeFromFn::FromUnchecked);
                    } else if p.is_ident("rest_reserved") {
                        rest_handling = Some(UbxEnumRestHandling::Reserved);
                    } else if p.is_ident("rest_error") {
                        rest_handling = Some(UbxEnumRestHandling::ErrorProne);
                    } else {
                        return Err(syn::Error::new(p.span(), "Invalid ubx attribute"));
                    }
                } else {
                    return Err(syn::Error::new(item.span(), "Invalid ubx attribute"));
                }
            }
        }
        _ => return Err(syn::Error::new(attr.span(), "Invalid ubx attributes")),
    }

    let attr = input
        .attrs
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
                return Err(syn::Error::new(
                    list.nested[0].span(),
                    "Invalid repr attribute for ubx_type enum",
                ));
            }
            syn::parse_quote! { u8 }
        }
        _ => {
            return Err(syn::Error::new(
                attr.span(),
                "Invalid repr attribute for ubx_type enum",
            ))
        }
    };
    let mut variants = Vec::with_capacity(input.variants.len());
    for var in input.variants {
        if syn::Fields::Unit != var.fields {
            return Err(syn::Error::new(
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
            return Err(syn::Error::new(
                expr.span(),
                "Invalid variant value for ubx_type enum",
            ));
        };
        variants.push((var.ident, variant_value));
    }

    let attrs = input
        .attrs
        .into_iter()
        .filter(|x| !x.path.is_ident("ubx") && !x.path.is_ident("ubx_type"))
        .collect();

    Ok(UbxEnum {
        attrs,
        name: input.ident,
        repr,
        from_fn,
        to_fn,
        rest_handling,
        variants,
    })
}

fn parse_ubx_attr(attrs: &[Attribute], struct_name: &Ident) -> syn::Result<PackHeader> {
    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("ubx"))
        .ok_or_else(|| {
            Error::new(
                struct_name.span(),
                format!("No ubx attribute for payload struct {}", struct_name),
            )
        })?;
    let meta = attr.parse_meta()?;
    trace!("parse_ubx_attr: ubx meta {:?}", meta);
    let meta = match meta {
        syn::Meta::List(x) => x,
        _ => return Err(Error::new(meta.span(), "Invalid ubx attribute syntax")),
    };

    let mut class = None;
    let mut id = None;
    let mut fixed_payload_len = None;

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
                } else {
                    return Err(Error::new(path.span(), "Unsupported attribute"));
                }
            }
            _ => return Err(Error::new(e.span(), "Unsupported attribute")),
        }
    }
    let class = class.ok_or_else(|| Error::new(meta.span(), "No \"class\" attribute"))?;
    let id = id.ok_or_else(|| Error::new(meta.span(), "No \"id\" attribute"))?;

    Ok(PackHeader {
        class,
        id,
        fixed_payload_len,
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
                }
                _ => return Err(Error::new(a.span(), "Invalid comments")),
            }
        }
    }
    Ok(doc_comments)
}

fn parse_fields(struct_data: syn::ItemStruct) -> syn::Result<Vec<PackField>> {
    let fields = match struct_data.fields {
        syn::Fields::Named(x) => x,
        _ => {
            return Err(Error::new(
                struct_data.fields.span(),
                "Unsupported fields format",
            ));
        }
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
        let mut map = PackFieldMap::none();
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
            if *map_ty == ty {
                return Err(Error::new(map_ty.span(), "You map type to the same type"));
            }
        }

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
}

impl Parse for PackFieldMap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut map = PackFieldMap::none();
        if input.peek(kw::map_type) {
            input.parse::<kw::map_type>()?;
            input.parse::<Token![=]>()?;
            map.map_type = Some(input.parse()?);
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if input.peek(kw::scale) {
            input.parse::<kw::scale>()?;
            input.parse::<Token![=]>()?;
            map.scale = Some(input.parse()?);
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if input.peek(kw::alias) {
            input.parse::<kw::alias>()?;
            input.parse::<Token![=]>()?;
            map.alias = Some(input.parse()?);
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
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
    //TODO: make this array static
    //TODO: support f32, f64
    let valid_types: [(Type, NonZeroUsize); 6] = [
        (syn::parse_quote!(u8), NonZeroUsize::new(1).unwrap()),
        (syn::parse_quote!(i8), NonZeroUsize::new(1).unwrap()),
        (syn::parse_quote!(u16), NonZeroUsize::new(2).unwrap()),
        (syn::parse_quote!(i16), NonZeroUsize::new(2).unwrap()),
        (syn::parse_quote!(u32), NonZeroUsize::new(4).unwrap()),
        (syn::parse_quote!(i32), NonZeroUsize::new(4).unwrap()),
    ];
    if let Some((_ty, size)) = valid_types.iter().find(|x| x.0 == *ty) {
        Ok(Some(*size))
    } else {
        Err(Error::new(
            ty.span(),
            format!("Not supported type, expect one of {:?}", valid_types),
        ))
    }
}
