use super::{keyword, MapType};
use quote::ToTokens;
use syn::{parse::Parse, Ident, Token};

#[derive(Default)]
pub struct PackFieldMap {
    pub map_type: Option<MapType>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
    pub convert_may_fail: bool,
    pub get_as_ref: bool,
}

impl PackFieldMap {
    pub(crate) fn is_none(&self) -> bool {
        self.map_type.is_none() && self.scale.is_none() && self.alias.is_none()
    }
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

            if lookahead.peek(keyword::map_type) {
                input.parse::<keyword::map_type>()?;
                input.parse::<Token![=]>()?;
                map_ty = Some(input.parse()?);
            } else if lookahead.peek(keyword::scale) {
                input.parse::<keyword::scale>()?;
                input.parse::<Token![=]>()?;
                map.scale = Some(input.parse()?);
            } else if lookahead.peek(keyword::alias) {
                input.parse::<keyword::alias>()?;
                input.parse::<Token![=]>()?;
                map.alias = Some(input.parse()?);
            } else if lookahead.peek(keyword::may_fail) {
                input.parse::<keyword::may_fail>()?;
                map.convert_may_fail = true;
            } else if lookahead.peek(keyword::from) {
                input.parse::<keyword::from>()?;
                input.parse::<Token![=]>()?;
                custom_from_fn = Some(input.parse()?);
            } else if lookahead.peek(keyword::is_valid) {
                input.parse::<keyword::is_valid>()?;
                input.parse::<Token![=]>()?;
                custom_is_valid_fn = Some(input.parse()?);
            } else if lookahead.peek(keyword::size_fn) {
                input.parse::<keyword::size_fn>()?;
                input.parse::<Token![=]>()?;
                custom_size_fn = Some(input.parse()?);
            } else if lookahead.peek(keyword::get_as_ref) {
                input.parse::<keyword::get_as_ref>()?;
                map.get_as_ref = true;
            } else if lookahead.peek(keyword::into) {
                input.parse::<keyword::into>()?;
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
