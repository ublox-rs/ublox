use std::num::NonZeroUsize;
use syn::{Ident, Type};

pub struct PackDesc {
    pub name: String,
    pub header: PackHeader,
    pub comment: String,
    pub fields: Vec<PackField>,
}

impl PackDesc {
    /// if packet has variable size, then `None`
    pub fn packet_payload_size(&self) -> Option<usize> {
        let mut ret: usize = 0;
        for f in &self.fields {
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
        self.map.map_type.as_ref().unwrap_or(&self.ty)
    }
    pub fn intermidiate_field_name(&self) -> &Ident {
        self.map.alias.as_ref().unwrap_or(&self.name)
    }
}

pub struct PackFieldMap {
    pub map_type: Option<Type>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
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
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum HowCodeForPackage {
    SendOnly,
    RecvOnly,
    SendRecv,
}

pub struct UbxEnum {
    pub name: Ident,
    pub repr: Type,
    pub from_fn: Option<UbxTypeFromFn>,
    pub to_fn: bool,
    pub rest_handling: Option<UbxEnumRestHandling>,
    pub variants: Vec<(Ident, u8)>,
    pub attrs: Vec<syn::Attribute>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UbxTypeFromFn {
    From,
    FromUnchecked,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UbxEnumRestHandling {
    Reserved,
    ErrorProne,
}
