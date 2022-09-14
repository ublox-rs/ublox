use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
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

    pub fn packet_payload_size_except_size_fn(&self) -> Option<usize> {
        PackDesc::fields_size(self.fields.iter().filter(|f| f.size_fn().is_none()))
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
    pub payload_len: PayloadLen,
    pub flags: Vec<PacketFlag>,
}

#[derive(Debug, Clone, Copy)]
pub enum PayloadLen {
    Fixed(u16),
    Max(u16),
}

impl PayloadLen {
    pub fn fixed(&self) -> Option<u16> {
        if let PayloadLen::Fixed(len) = self {
            Some(*len)
        } else {
            None
        }
    }
}

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
            .map_or(false, |m| crate::type_is_option(&m.ty))
    }
}

#[derive(Debug)]
pub struct PackFieldMapDesc {
    pub map_type: Option<MapTypeDesc>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
    pub convert_may_fail: bool,
    pub get_as_ref: bool,
}

#[derive(Debug)]
pub struct MapTypeDesc {
    pub ty: Type,
    pub from_fn: TokenStream,
    pub is_valid_fn: TokenStream,
    pub into_fn: TokenStream,
    pub size_fn: Option<TokenStream>,
}

impl PackFieldMapDesc {
    pub fn new(x: crate::input::PackFieldMap, raw_ty: &Type) -> Self {
        let convert_may_fail = x.convert_may_fail;
        let scale_back = x.scale.as_ref().map(|x| quote! { 1. / #x });
        let map_type = x.map_type.map(|map_type| {
            let ty = map_type.ty;
            let from_fn = map_type.from_fn.unwrap_or_else(|| {
                if !convert_may_fail {
                    quote! { <#ty>::from }
                } else {
                    quote! { <#ty>::from_unchecked }
                }
            });

            let is_valid_fn = map_type.is_valid_fn.unwrap_or_else(|| {
                quote! { <#ty>::is_valid }
            });

            let into_fn = map_type.into_fn.unwrap_or_else(|| {
                if ty == syn::parse_quote! {f32} || ty == syn::parse_quote! {f64} {
                    if let Some(scale_back) = scale_back {
                        let conv_method =
                            quote::format_ident!("as_{}", raw_ty.into_token_stream().to_string());

                        return quote! {
                            ScaleBack::<#ty>(#scale_back).#conv_method
                        };
                    }
                }

                quote! { <#ty>::into_raw }
            });

            MapTypeDesc {
                ty,
                from_fn,
                is_valid_fn,
                into_fn,
                size_fn: map_type.size_fn,
            }
        });
        Self {
            map_type,
            scale: x.scale,
            alias: x.alias,
            convert_may_fail: x.convert_may_fail,
            get_as_ref: x.get_as_ref,
        }
    }
}

impl PackField {
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

pub struct UbxExtendEnum {
    pub name: Ident,
    pub repr: Type,
    pub from_fn: Option<UbxTypeFromFn>,
    pub rest_handling: Option<UbxEnumRestHandling>,
    pub into_fn: Option<UbxTypeIntoFn>,
    pub variants: Vec<(Ident, u8)>,
    pub attrs: Vec<syn::Attribute>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UbxTypeFromFn {
    From,
    FromUnchecked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UbxTypeIntoFn {
    Raw,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PacketFlag {
    DefaultForBuilder,
}

pub struct RecvPackets {
    pub union_enum_name: Ident,
    pub unknown_ty: Ident,
    pub all_packets: Vec<Ident>,
}
