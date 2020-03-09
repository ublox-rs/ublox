use proc_macro2::TokenStream;
use quote::quote;
use std::num::NonZeroUsize;
use syn::{Attribute, Ident, Type};

pub struct PackDesc {
    pub name: String,
    pub header: PackHeader,
    pub comment: String,
    pub fields: Vec<PackField>,
}

impl PackDesc {
    /// if packet has variable size, then `None`
    pub fn packet_payload_size(&self) -> Option<usize> {
        PackDesc::fields_size(self.fields.iter())
    }

    pub fn packet_payload_size_except_last_field(&self) -> Option<usize> {
        PackDesc::fields_size(self.fields.iter().rev().skip(1))
    }

    fn fields_size<'a, I: Iterator<Item = &'a PackField>>(iter: I) -> Option<usize> {
        let mut ret: usize = 0;
        for f in iter {
            let size = f.size_bytes?;
            ret = ret
                .checked_add(size.get())
                .expect("overflow during packet size calculation");
        }
        Some(ret)
    }
}

pub struct PackHeader {
    pub class: u8,
    pub id: u8,
    pub fixed_payload_len: Option<u16>,
    pub flags: Vec<PacketFlag>,
}

pub struct PackField {
    pub name: Ident,
    pub ty: Type,
    pub map: PackFieldMap,
    pub comment: String,
    pub size_bytes: Option<NonZeroUsize>,
}

impl PackField {
    pub fn has_intermidiate_type(&self) -> bool {
        self.map.map_type.is_some()
    }
    pub fn intermidiate_type(&self) -> &Type {
        self.map
            .map_type
            .as_ref()
            .map(|x| &x.ty)
            .unwrap_or(&self.ty)
    }
    pub fn intermidiate_field_name(&self) -> &Ident {
        self.map.alias.as_ref().unwrap_or(&self.name)
    }
    pub fn is_field_raw_ty_byte_array(&self) -> bool {
        if let syn::Type::Array(ref fixed_array) = self.ty {
            if *fixed_array.elem == syn::parse_quote!(u8) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub struct PackFieldMap {
    pub map_type: Option<MapType>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
    pub convert_may_fail: bool,
    pub get_as_ref: bool,
}

impl PackFieldMap {
    pub fn is_none(&self) -> bool {
        self.map_type.is_none() && self.scale.is_none() && self.alias.is_none()
    }
    pub fn none() -> Self {
        Self {
            map_type: None,
            scale: None,
            alias: None,
            convert_may_fail: false,
            get_as_ref: false,
        }
    }
}

pub struct MapType {
    pub ty: Type,
    pub from_fn: TokenStream,
    pub is_valid_fn: TokenStream,
}

impl MapType {
    pub fn new(ty: Type, convert_may_fail: bool) -> Self {
        let from_fn = if !convert_may_fail {
            quote! { <#ty>::from }
        } else {
            quote! { <#ty>::from_unchecked }
        };
        let is_valid_fn = quote! { <#ty>::is_valid };
        Self {
            ty,
            from_fn,
            is_valid_fn,
        }
    }
}

pub struct UbxExtendEnum {
    pub name: Ident,
    pub repr: Type,
    pub from_fn: Option<UbxTypeFromFn>,
    pub rest_handling: Option<UbxEnumRestHandling>,
    pub into_fn: Option<UbxTypeIntoFn>,
    pub variants: Vec<(Ident, u8)>,
    pub attrs: Vec<syn::Attribute>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UbxTypeFromFn {
    From,
    FromUnchecked,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UbxTypeIntoFn {
    Raw,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UbxEnumRestHandling {
    Reserved,
    ErrorProne,
}

pub struct BitFlagsMacro {
    pub nbits: u32,
    pub vis: syn::Visibility,
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub repr_ty: Type,
    pub consts: Vec<BitFlagsMacroItem>,
    pub from_fn: Option<UbxTypeFromFn>,
    pub into_fn: Option<UbxTypeIntoFn>,
    pub rest_handling: Option<UbxEnumRestHandling>,
}

pub struct BitFlagsMacroItem {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub value: u64,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PacketFlag {
    DefaultForBuilder,
}

pub struct RecvPackets {
    pub union_enum_name: Ident,
    pub unknown_ty: Ident,
    pub all_packets: Vec<Ident>,
}
