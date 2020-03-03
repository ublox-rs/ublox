use crate::types::{HowCodeForPackage, PackDesc, UbxEnum, UbxEnumRestHandling, UbxTypeFromFn};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use syn::Ident;

pub fn generate_code_for_packet(
    pack_descr: &PackDesc,
    ubx_types: &HashMap<String, UbxEnum>,
    mode: HowCodeForPackage,
) -> String {
    let mut ret = String::with_capacity(10 * 1024);
    let code = generate_types_for_packet(pack_descr);
    ret.push_str(&code.to_string());

    if mode == HowCodeForPackage::RecvOnly || mode == HowCodeForPackage::SendRecv {
        let code = generate_recv_code_for_packet(pack_descr, ubx_types);
        ret.push_str(&code.to_string());
    }

    if mode == HowCodeForPackage::SendOnly || mode == HowCodeForPackage::SendRecv {
        let code_frags = generate_send_code_for_packet(pack_descr);
        for code in code_frags {
            ret.push_str(&code.to_string());
        }
    }

    ret
}

pub fn generate_code_for_ubx_enum(ubx_enum: &UbxEnum) -> String {
    assert_eq!(ubx_enum.repr, {
        let ty: syn::Type = syn::parse_quote! { u8 };
        ty
    });
    let name = &ubx_enum.name;
    let mut variants = ubx_enum.variants.clone();
    let attrs = &ubx_enum.attrs;
    if let Some(UbxEnumRestHandling::Reserved) = ubx_enum.rest_handling {
        let defined: HashSet<u8> = ubx_enum.variants.iter().map(|x| x.1).collect();
        for i in 0..=u8::max_value() {
            if !defined.contains(&i) {
                let name = format_ident!("Reserved{}", i);
                variants.push((name, i));
            }
        }
    }
    let repr_ty = &ubx_enum.repr;
    let from_code = match ubx_enum.from_fn {
        Some(UbxTypeFromFn::From) => {
            assert_ne!(
                Some(UbxEnumRestHandling::ErrorProne),
                ubx_enum.rest_handling
            );
            let mut match_branches = Vec::with_capacity(variants.len());
            for (id, val) in &variants {
                match_branches.push(quote! { #val => #name :: #id });
            }

            quote! {
                impl #name {
                    fn from(x: #repr_ty) -> Self {
                        match x {
                            #(#match_branches),*
                        }
                    }
                }
            }
        }
        Some(UbxTypeFromFn::FromUnchecked) => {
            assert_ne!(Some(UbxEnumRestHandling::Reserved), ubx_enum.rest_handling);
            let mut match_branches = Vec::with_capacity(variants.len());
            for (id, val) in &variants {
                match_branches.push(quote! { #val => #name :: #id });
            }

            let mut values = Vec::with_capacity(variants.len());
            for (i, (_, val)) in variants.iter().enumerate() {
                if i != 0 {
                    values.push(quote! { | #val });
                } else {
                    values.push(quote! { #val });
                }
            }

            quote! {
                impl #name {
                    fn from_unchecked(x: #repr_ty) -> Self {
                        match x {
                            #(#match_branches),*,
                            _ => unreachable!(),
                        }
                    }
                    fn is_valid_to_convert(x: #repr_ty) -> bool {
                        match x {
                            #(#values)* => true,
                            _ => false,
                        }
                    }
                }
            }
        }
        None => quote! {},
    };

    let mut enum_variants = Vec::with_capacity(variants.len());
    for (id, val) in &variants {
        enum_variants.push(quote! { #id = #val });
    }

    let code = quote! {
        #(#attrs)*
        pub enum #name {
            #(#enum_variants),*
        }

        #from_code
    };
    code.to_string()
}

fn generate_recv_code_for_packet(
    pack_descr: &PackDesc,
    ubx_types: &HashMap<String, UbxEnum>,
) -> TokenStream {
    let ref_name = format_ident!("{}Ref", pack_descr.name);
    let mut getters = Vec::with_capacity(pack_descr.fields.len());

    let mut off = 0usize;
    for f in &pack_descr.fields {
        let ty = f.intermidiate_type();
        let get_name = f.intermidiate_field_name();

        let size_bytes = match f.size_bytes {
            Some(x) => x,
            None => unimplemented!(),
        };
        let mut bytes = Vec::with_capacity(size_bytes.get());
        for i in 0..size_bytes.get() {
            let byte_off = off.checked_add(i).unwrap();
            bytes.push(quote! { self.0[#byte_off] });
        }
        let raw_ty = &f.ty;

        let mut get_value_lines = if size_bytes.get() != 1 {
            vec![quote! { <#raw_ty>::from_le_bytes([#(#bytes),*]) }]
        } else {
            vec![quote! { self.0[#off] }]
        };

        if let Some(ref out_ty) = f.map.map_type {
            let use_from_unchecked =
                if let Some(ubx_type) = ubx_types.get(&out_ty.into_token_stream().to_string()) {
                    ubx_type.from_fn == Some(UbxTypeFromFn::FromUnchecked)
                } else {
                    false
                };

            let get_raw = &get_value_lines[0];
            let new_line = quote! { let val = #get_raw ;  };
            get_value_lines[0] = new_line;
            if use_from_unchecked {
                get_value_lines.push(quote! {
                    <#out_ty>::from_unchecked(val)
                });
            } else {
                get_value_lines.push(quote! {
                    <#out_ty>::from(val)
                });
            }
        }

        if let Some(ref scale) = f.map.scale {
            let last_i = get_value_lines.len() - 1;
            let last_line = &get_value_lines[last_i];
            let new_last_line = quote! { let val = #last_line ; };
            get_value_lines[last_i] = new_last_line;
            get_value_lines.push(quote! {val * #scale });
        }
        let field_comment = &f.comment;
        getters.push(quote! {
            #[doc = #field_comment]
            #[inline]
            pub fn #get_name(&self) -> #ty {
                #(#get_value_lines)*
            }
        });
        off += size_bytes.get();
    }
    let struct_comment = &pack_descr.comment;

    quote! {
        #[doc = #struct_comment]
        #[doc = "It is just reference to internal parser's buffer"]
        pub struct #ref_name<'a>(&'a [u8]);
        impl<'a> #ref_name<'a> {
            #(#getters)*
        }
    }
}

fn generate_types_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let name = Ident::new(&pack_descr.name, Span::call_site());
    let class = pack_descr.header.class;
    let id = pack_descr.header.id;
    let fixed_payload_len = match pack_descr.header.fixed_payload_len {
        Some(x) => quote! { Some(#x) },
        None => quote! { None },
    };
    let struct_comment = &pack_descr.comment;
    quote! {

        #[doc = #struct_comment]
        pub struct #name;
        impl UbxPacket for #name {
            const CLASS: u8 = #class;
            const ID: u8 = #id;
            const FIXED_PAYLOAD_LENGTH: Option<u16> = #fixed_payload_len;
        }
    }
}

fn generate_send_code_for_packet(pack_descr: &PackDesc) -> Vec<TokenStream> {
    let main_name = Ident::new(&pack_descr.name, Span::call_site());
    let payload_struct = format_ident!("{}Builder", pack_descr.name);

    let mut fields = Vec::with_capacity(pack_descr.fields.len());
    let mut pack_fields = Vec::with_capacity(pack_descr.fields.len());
    let mut write_fields = Vec::with_capacity(pack_descr.fields.len());
    let mut off = 6usize;
    for f in &pack_descr.fields {
        let ty = f.intermidiate_type();
        let name = f.intermidiate_field_name();
        let field_comment = &f.comment;
        fields.push(quote! {
            #[doc = #field_comment]
            pub #name: #ty
        });
        let size_bytes = match f.size_bytes {
            Some(x) => x,
            None => unimplemented!(),
        };
        if f.has_intermidiate_type() {
            pack_fields.push(quote! {
                let bytes = self.#name.as_raw_value().to_le_bytes()
            });
        } else {
            pack_fields.push(quote! {
                let bytes = self.#name.to_le_bytes()
            });
        }
        write_fields.push(pack_fields.last().unwrap().clone());
        write_fields.push(quote! {
            out.write(&bytes)?;
            checksum_calc.update(&bytes)
        });
        for i in 0..size_bytes.get() {
            let byte_off = off.checked_add(i).unwrap();
            pack_fields.push(quote! {
                ret[#byte_off] = bytes[#i]
            });
        }

        off += size_bytes.get();
    }

    let mut ret = Vec::with_capacity(4);
    let struct_comment = &pack_descr.comment;
    ret.push(quote! {
        #[doc = #struct_comment]
        #[doc = "Struct that used as \"builder\" for packet"]
        pub struct #payload_struct {
            #(#fields),*
        }
    });

    if let Some(packet_payload_size) = pack_descr.packet_payload_size() {
        let packet_size = packet_payload_size + 8;
        let packet_payload_size_u16 = u16::try_from(packet_payload_size).unwrap();
        ret.push(quote! {
            impl #payload_struct {
                #[inline]
                pub fn to_packet_bytes(self) -> [u8; #packet_size] {
                    let mut ret = [0u8; #packet_size];
                    ret[0] = SYNC_CHAR_1;
                    ret[1] = SYNC_CHAR_2;
                    ret[2] = #main_name::CLASS;
                    ret[3] = #main_name::ID;
                    let pack_len_bytes = #packet_payload_size_u16 .to_le_bytes();
                    ret[4] = pack_len_bytes[0];
                    ret[5] = pack_len_bytes[1];
                    #(#pack_fields);*;
                    let (ck_a, ck_b) = ubx_checksum(&ret[2..#packet_size-2]);
                    ret[#packet_size-2] = ck_a;
                    ret[#packet_size-1] = ck_b;
                    ret
                }
            }
            impl From<#payload_struct> for [u8; #packet_size] {
                fn from(x: #payload_struct) -> Self {
                    x.to_packet_bytes()
                }
            }
        });

        ret.push(quote! {
            impl UbxPacketCreator for #payload_struct {
                #[inline]
                fn create_packet(self, out: &mut dyn MemWriter) -> Result<(), NotEnoughMem> {
                    out.reserve_allocate(#packet_size)?;
                    let len_bytes = #packet_payload_size_u16 .to_le_bytes();
                    let header = [SYNC_CHAR_1, SYNC_CHAR_2, #main_name::CLASS, #main_name::ID, len_bytes[0], len_bytes[1]];
                    out.write(&header)?;
                    let mut checksum_calc = UbxChecksumCalc::default();
                    checksum_calc.update(&header[2..]);
                    #(#write_fields);*;
                    let (ck_a, ck_b) = checksum_calc.result();
                    out.write(&[ck_a, ck_b])?;
                    Ok(())
                }
            }
        });
    } else {
        unimplemented!();
    }

    ret
}
