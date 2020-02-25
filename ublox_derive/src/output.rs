use crate::types::{PackDesc, PackFieldEnum, PackFieldFlags, PackFieldRepr};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

pub fn generate_recv_code_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let ref_name = format_ident!("{}Ref", pack_descr.name);
    let mut getters = Vec::with_capacity(pack_descr.fields.len());

    for f in &pack_descr.fields {
        let ty = f.intermidiate_type();
        let get_name = &f.name;
        getters.push(quote! {
            #[inline]
            pub fn #get_name(&self) -> #ty {
            unimplemented!()
            }
        });
    }
    quote! {
    pub struct #ref_name<'a>(&'a [u8]);
    impl<'a> #ref_name<'a> {
            #(#getters)*
    }
    }
}

pub fn generate_types_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let mut req_types = Vec::new();
    for f in &pack_descr.fields {
        let ty = f.intermidiate_type();
        match f.repr {
            PackFieldRepr::Plain | PackFieldRepr::Map(_) => {}
            PackFieldRepr::Enum(ref e) => {
                req_types.push(define_enum(e, ty, &f.ty));
            }
            PackFieldRepr::Flags(ref flags) => {
                req_types.push(define_bitflags(flags, ty, &f.ty));
            }
        }
    }
    let name = format_ident!("{}Raw", pack_descr.name);
    let class = pack_descr.header.class;
    let id = pack_descr.header.id;
    let fixed_len = match pack_descr.header.fixed_len {
        Some(x) => quote! { Some(#x) },
        None => quote! { None },
    };
    quote! {
        #(#req_types)*

    impl UbxPacket for #name {
        const class: u8 = #class;
        const id: u8 = #id;
    const fixed_length: Option<u16> = #fixed_len;
    }
    }
}

pub fn generate_send_code_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let payload_struct = format_ident!("{}Payload", pack_descr.name);

    let mut fields = Vec::with_capacity(pack_descr.fields.len());
    for f in &pack_descr.fields {
        let ty = f.intermidiate_type();
        let name = &f.name;
        fields.push(quote! {
            #name: #ty
        });
    }

    match pack_descr.packet_size() {
        Some(packet_size) => {
            quote! {
            pub struct #payload_struct {
                #(#fields),*
            }
            impl #payload_struct {
                #[inline]
                fn to_packet(s: #payload_struct) -> [u8; #packet_size] {
                unimplemented!();
                }
            }
            }
        }
        None => {
            quote! {
            pub struct #payload_struct {
                #(#fields)*,
            }
            impl #payload_struct {
                #[inline]
                fn to_packet(s: payload_struct, out: &mut Vec<u8>) {
                unimplemented!();
                }
            }
            }
        }
    }
}

fn define_enum(enum_: &PackFieldEnum, enum_name: &Type, repr_ty: &Type) -> TokenStream {
    let mut members = Vec::with_capacity(enum_.values.len());
    for mem in &enum_.values {
        let name = &mem.name;
        let value = &mem.value;
        members.push(quote! {
            #name = #value
        });
    }
    quote! {
    #[repr(#repr_ty)]
        pub enum #enum_name {
        #(#members),*
        }
    }
}

fn define_bitflags(flags: &PackFieldFlags, flags_name: &Type, repr_ty: &Type) -> TokenStream {
    let mut consts = Vec::with_capacity(flags.values.len());
    for mem in &flags.values {
        let name = &mem.name;
        let num: u64 = match mem.value {
            crate::types::PackFieldFlagValue::Bit(x) => 1u64 << x,
            crate::types::PackFieldFlagValue::Mask(x) => x,
        };
        consts.push(quote! {
            pub const #name: #flags_name = #flags_name(#num as #repr_ty);
        });
    }
    quote! {

    #[repr(transparent)]
        pub struct #flags_name(#repr_ty);

    impl #flags_name {
        #(#consts)*
    }
    }
}
