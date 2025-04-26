use packetflag::PacketFlag;
use packfield::PackField;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Generics, Ident, Lifetime, Type};

pub(crate) mod packetflag;
pub(crate) mod packfield;
pub(crate) mod packfieldmapdesc;
pub(crate) mod recvpackets;

pub struct PackDesc {
    pub name: String,
    pub header: PackHeader,
    pub comment: String,
    pub fields: Vec<PackField>,
    pub generics: Generics,
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

    /// Returns lifetimes if the packet has any on the form `<'a, 'b, 'c>`
    pub(crate) fn lifetime_tokens(&self) -> Option<TokenStream> {
        let lifetimes: Vec<Lifetime> = self
            .generics
            .lifetimes()
            .map(|ldef| ldef.lifetime.clone())
            .collect();

        if lifetimes.is_empty() {
            None
        } else {
            // Create a TokenStream with the lifetimes in angle brackets
            let tokens = quote! { <#(#lifetimes),*> };
            Some(tokens)
        }
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
    /// If the payload length is fixed, returns `Some(len)` else `None`
    pub fn fixed(&self) -> Option<u16> {
        if let PayloadLen::Fixed(len) = self {
            Some(*len)
        } else {
            None
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
