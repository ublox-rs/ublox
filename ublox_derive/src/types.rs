use syn::{punctuated::Punctuated, Ident, Token, Type};

pub struct PackDesc {
    pub name: String,
    pub header: PackHeader,
    pub comment: String,
    pub fields: Vec<PackField>,
}

impl PackDesc {
    /// if packet has variable size, then `None`
    pub fn packet_size(&self) -> Option<usize> {
        let mut ret: usize = 0;
        for f in &self.fields {
            let size = f.size_bytes?;
            ret = ret
                .checked_add(size)
                .expect("overflow during packet size calculation");
        }
        Some(ret)
    }
}

pub struct PackHeader {
    pub class: u8,
    pub id: u8,
    pub fixed_len: Option<u16>,
}

pub struct PackField {
    pub name: Ident,
    pub ty: Type,
    pub repr: PackFieldRepr,
    pub comment: String,
    pub field_name_as_type: Type,
    pub size_bytes: Option<usize>,
}

impl PackField {
    pub fn intermidiate_type(&self) -> &Type {
        match self.repr {
            PackFieldRepr::Plain => &self.ty,
            PackFieldRepr::Map(ref map) => map.out_type.as_ref().unwrap_or(&self.ty),
            PackFieldRepr::Enum(ref en) => en
                .explicit_name
                .as_ref()
                .unwrap_or(&self.field_name_as_type),
            PackFieldRepr::Flags(ref f) => {
                f.explicit_name.as_ref().unwrap_or(&self.field_name_as_type)
            }
        }
    }
}

pub enum PackFieldRepr {
    Plain,
    Map(PackFieldMap),
    Enum(PackFieldEnum),
    Flags(PackFieldFlags),
}

pub struct PackFieldMap {
    pub out_type: Option<Type>,
    pub scale: Option<syn::LitFloat>,
    pub alias: Option<Ident>,
}

impl PackFieldMap {
    pub fn is_none(&self) -> bool {
        self.out_type.is_none() && self.scale.is_none() && self.alias.is_none()
    }
    pub fn none() -> Self {
        Self {
            out_type: None,
            scale: None,
            alias: None,
        }
    }
}

pub struct PackFieldEnum {
    pub explicit_name: Option<Type>,
    pub values: Punctuated<PackFieldEnumItemValue, Token![,]>,
}

pub struct PackFieldEnumItemValue {
    pub comment: String,
    pub name: Ident,
    pub _eq_token: Token![=],
    pub value: syn::LitInt,
}

pub struct PackFieldFlags {
    pub explicit_name: Option<Type>,
    pub values: Punctuated<PackFieldBitflagItemValue, Token![,]>,
}

pub struct PackFieldBitflagItemValue {
    pub comment: String,
    pub name: Ident,
    pub _eq_token: Token![=],
    pub value: PackFieldFlagValue,
}

#[derive(Clone, Copy)]
pub enum PackFieldFlagValue {
    Bit(u8),
    Mask(u64),
}
