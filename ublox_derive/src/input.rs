use crate::types::{
    PackDesc, PackField, PackFieldBitflagItemValue, PackFieldEnum, PackFieldEnumItemValue,
    PackFieldFlagValue, PackFieldFlags, PackFieldMap, PackFieldRepr, PackHeader,
};
use heck::CamelCase;
use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned, Attribute, Error, Ident, Token, Type};

pub fn parse_packet_description(input: syn::DeriveInput) -> syn::Result<PackDesc> {
    let struct_name = &input.ident;

    const REQUIRED_SUFFIX: &str = "Raw";
    let name = struct_name.to_string();
    if !name.ends_with(REQUIRED_SUFFIX) {
        return Err(Error::new(
            input.ident.span(),
            format!(
                "Invalid name \"{}\", should ends with \"{}\"",
                struct_name, REQUIRED_SUFFIX
            ),
        ));
    }
    let name = &name[0..(name.len() - REQUIRED_SUFFIX.len())];

    trace!("attrs: {:?}", input.attrs);
    validate_has_repr_packed(&input.attrs, &struct_name)?;
    let header = parse_ubx_attr(&input.attrs, &struct_name)?;
    let struct_comment = extract_item_comment(&input.attrs)?;

    let struct_data = match input.data {
        syn::Data::Struct(x) => x,
        _ => return Err(Error::new(input.span(), "Should be struct")),
    };
    let fields = parse_fields(struct_data)?;

    Ok(PackDesc {
        name: name.to_string(),
        header,
        comment: struct_comment,
        fields,
    })
}

fn validate_has_repr_packed(attrs: &[Attribute], struct_name: &Ident) -> syn::Result<()> {
    let attr = attrs
        .iter()
        .find(|a| a.path.is_ident("repr"))
        .ok_or_else(|| {
            Error::new(
                struct_name.span(),
                format!("No repr(packed) for payload struct {}", struct_name),
            )
        })?;
    if attr.into_token_stream().to_string() != "# [repr (packed)]" {
        return Err(Error::new(attr.span(), "Expect repr(packed) here"));
    }

    Ok(())
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
    let mut fixed_len = None;

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
                } else if path.is_ident("fixed_len") {
                    if fixed_len.is_some() {
                        return Err(Error::new(e.span(), "Duplicate \"fixed_len\" attribute"));
                    }
                    fixed_len = match lit {
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
        fixed_len,
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

fn parse_fields(struct_data: syn::DataStruct) -> syn::Result<Vec<PackField>> {
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
        let mut repr = PackFieldRepr::Plain;
        let comment = extract_item_comment(&attrs)?;
        for a in attrs {
            if !a.path.is_ident("doc") {
                match repr {
                    PackFieldRepr::Plain => (),
                    _ => return Err(Error::new(a.span(), "Two attributes for the same field")),
                }
                repr = a.parse_args()?;
            }
        }
        let name_camel_case = name.to_string().to_camel_case();
        let field_name_as_type: Type = syn::parse_str(&name_camel_case).map_err(|err| {
            Error::new(
                name.span(),
                format!("can not parse {} as type: {}", name_camel_case, err),
            )
        })?;

        ret.push(PackField {
            name,
            ty,
            repr,
            comment,
            field_name_as_type,
            size_bytes,
        });
    }

    Ok(ret)
}

mod kw {
    syn::custom_keyword!(bitflags);
    syn::custom_keyword!(map_type);
    syn::custom_keyword!(scale);
    syn::custom_keyword!(alias);
}

impl Parse for PackFieldRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::bitflags) {
            input.parse::<kw::bitflags>()?;
            let mut name = None;
            if input.peek(Ident) {
                name = Some(input.parse::<Type>()?);
            }
            let content;
            let _brace_token = syn::braced!(content in input);

            Ok(PackFieldRepr::Flags(PackFieldFlags {
                explicit_name: name,
                values: content.parse_terminated(PackFieldBitflagItemValue::parse)?,
            }))
        } else if lookahead.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            let mut name = None;
            if input.peek(Ident) {
                name = Some(input.parse::<Type>()?);
            }

            let content;
            let _brace_token = syn::braced!(content in input);
            Ok(PackFieldRepr::Enum(PackFieldEnum {
                explicit_name: name,
                values: content.parse_terminated(PackFieldEnumItemValue::parse)?,
            }))
        } else if lookahead.peek(kw::map_type)
            || lookahead.peek(kw::scale)
            || lookahead.peek(kw::alias)
        {
            let mut map = PackFieldMap::none();

            if input.peek(kw::map_type) {
                input.parse::<kw::map_type>()?;
                input.parse::<Token![=]>()?;
                map.out_type = Some(input.parse()?);
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

            assert!(!map.is_none());

            Ok(PackFieldRepr::Map(map))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for PackFieldEnumItemValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comment: Comment = input.parse()?;
        Ok(Self {
            comment: comment.0,
            name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for PackFieldBitflagItemValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comment: Comment = input.parse()?;
        Ok(Self {
            comment: comment.0,
            name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for PackFieldFlagValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let value = ident.to_string();
        if value.starts_with("bit") {
            let number_str = &value[3..];
            let n: u8 = number_str.parse().map_err(|err| {
                Error::new(
                    ident.span(),
                    format!("Can not parse {} as number: {}", number_str, err),
                )
            })?;
            Ok(PackFieldFlagValue::Bit(n))
        } else if value.starts_with("mask") {
            let number_str = &value[4..];
            let n: u64 = number_str.parse().map_err(|err| {
                Error::new(
                    ident.span(),
                    format!("Can not parse {} as number: {}", number_str, err),
                )
            })?;
            Ok(PackFieldFlagValue::Mask(n))
        } else {
            Err(Error::new(ident.span(), "Expect bitX or maskX here"))
        }
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

fn field_size_bytes(ty: &Type) -> syn::Result<Option<usize>> {
    //TODO: make this array static
    let valid_types: [(Type, usize); 8] = [
        (syn::parse_quote!(u8), 1),
        (syn::parse_quote!(i8), 1),
        (syn::parse_quote!(u16), 2),
        (syn::parse_quote!(i16), 2),
        (syn::parse_quote!(u32), 4),
        (syn::parse_quote!(i32), 4),
        (syn::parse_quote!(f32), 4),
        (syn::parse_quote!(f64), 8),
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
