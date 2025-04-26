use proc_macro2::TokenStream;
use std::num::NonZeroUsize;
use syn::{Ident, Type};

use super::packfieldmapdesc::PackFieldMapDesc;

#[derive(Debug)]
pub struct PackField {
    pub name: Ident,
    pub ty: Type,
    pub map: PackFieldMapDesc,
    pub comment: String,
    pub size_bytes: Option<NonZeroUsize>,
}

impl PackField {
    pub fn size_fn(&self) -> Option<&TokenStream> {
        self.map.map_type.as_ref().and_then(|m| m.size_fn.as_ref())
    }

    pub fn is_optional(&self) -> bool {
        self.map
            .map_type
            .as_ref()
            .is_some_and(|m| crate::type_is_option(&m.ty))
    }

    pub fn has_intermediate_type(&self) -> bool {
        self.map.map_type.is_some()
    }
    pub fn intermediate_type(&self) -> &Type {
        self.map
            .map_type
            .as_ref()
            .map(|x| &x.ty)
            .unwrap_or(&self.ty)
    }
    pub fn intermediate_field_name(&self) -> &Ident {
        self.map.alias.as_ref().unwrap_or(&self.name)
    }
    pub fn is_field_raw_ty_byte_array(&self) -> bool {
        if let syn::Type::Array(ref fixed_array) = self.ty {
            *fixed_array.elem == syn::parse_quote!(u8)
        } else {
            false
        }
    }
}
