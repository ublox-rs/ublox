use crate::types::{BitFlagsMacro, BitFlagsMacroItem};
use quote::ToTokens;

use syn::{
    braced, parse::Parse, punctuated::Punctuated, spanned::Spanned, Attribute, Error, Ident, Token,
    Type,
};

pub fn parse_bitflags(mac: syn::ItemMacro) -> syn::Result<BitFlagsMacro> {
    let (from_fn, into_fn, rest_handling) =
        super::parse_ubx_extend_attrs("#[ubx_extend_bitflags]", mac.span(), &mac.attrs)?;

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
