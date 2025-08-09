use std::num::NonZeroUsize;

use crate::types::{
    packfield::PackField, packfieldmapdesc::PackFieldMapDesc, PackHeader, PayloadLen,
    UbxEnumRestHandling, UbxTypeFromFn, UbxTypeIntoFn,
};
use proc_macro2::Span;

use quote::ToTokens as _;
use syn::{spanned::Spanned, Attribute, Error, Fields, Ident, Type};

use super::{packfieldmap::PackFieldMap, StructFlags};

pub(crate) fn parse_ubx_extend_attrs(
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
        .ok_or_else(|| Error::new(item_sp, format!("No ubx attribute for {ubx_extend_name}")))?;
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

pub(super) fn parse_ubx_attr(attrs: &[Attribute], struct_name: &Ident) -> syn::Result<PackHeader> {
    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("ubx"))
        .ok_or_else(|| {
            Error::new(
                struct_name.span(),
                format!("No ubx attribute for struct {struct_name}"),
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

pub(super) fn extract_item_comment(attrs: &[Attribute]) -> syn::Result<String> {
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

pub(super) fn parse_fields(fields: Fields) -> syn::Result<Vec<PackField>> {
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
                "Cannot interpret array length",
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
            format!("Unsupported type, expected one of {valid_type_names:?}"),
        ))
    }
}
