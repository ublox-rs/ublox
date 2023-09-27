use crate::types::BitFlagsMacro;
use crate::types::{
    PackDesc, PackField, PacketFlag, PayloadLen, RecvPackets, UbxEnumRestHandling, UbxExtendEnum,
    UbxTypeFromFn, UbxTypeIntoFn,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{collections::HashSet, convert::TryFrom};
use syn::{parse_quote, Ident, Type};

fn generate_debug_impl(pack_name: &str, ref_name: &Ident, pack_descr: &PackDesc) -> TokenStream {
    let fields = pack_descr.fields.iter().map(|field| {
        let field_name = &field.name;
        let field_accessor = field.intermediate_field_name();
        quote! {
            .field(stringify!(#field_name), &self.#field_accessor())
        }
    });
    quote! {
        impl core::fmt::Debug for #ref_name<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(#pack_name)
                    #(#fields)*
                    .finish()
            }
        }
    }
}

fn generate_serialize_impl(
    _pack_name: &str,
    ref_name: &Ident,
    pack_descr: &PackDesc,
) -> TokenStream {
    let fields = pack_descr.fields.iter().map(|field| {
        let field_name = &field.name;
        let field_accessor = field.intermediate_field_name();
        if field.size_bytes.is_some() || field.is_optional() {
            quote! {
                state.serialize_entry(stringify!(#field_name), &self.#field_accessor())?;
            }
        } else {
            quote! {
                state.serialize_entry(
                    stringify!(#field_name),
                    &crate::ubx_packets::FieldIter(self.#field_accessor())
                )?;
            }
        }
    });
    quote! {
        #[cfg(feature = "serde")]
        impl SerializeUbxPacketFields for #ref_name<'_> {
            fn serialize_fields<S>(&self, state: &mut S) -> Result<(), S::Error>
            where
                S: serde::ser::SerializeMap,
            {
                #(#fields)*
                Ok(())
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for #ref_name<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut state = serializer.serialize_map(None)?;
                self.serialize_fields(&mut state)?;
                state.end()
            }
        }
    }
}

pub fn generate_recv_code_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let pack_name = &pack_descr.name;
    let ref_name = format_ident!("{}Ref", pack_descr.name);

    let mut getters = Vec::with_capacity(pack_descr.fields.len());
    let mut field_validators = Vec::new();
    let mut size_fns = Vec::new();

    let mut off = 0usize;
    for (field_index, f) in pack_descr.fields.iter().enumerate() {
        let ty = f.intermediate_type();
        let get_name = f.intermediate_field_name();
        let field_comment = &f.comment;

        if let Some(size_bytes) = f.size_bytes.map(|x| x.get()) {
            let get_raw = get_raw_field_code(f, off, quote! {self.0});
            let new_line = quote! { let val = #get_raw;  };
            let mut get_value_lines = vec![new_line];

            if let Some(ref out_ty) = f.map.map_type {
                let get_raw_name = format_ident!("{}_raw", get_name);
                let slicetype = syn::parse_str("&[u8]").unwrap();
                let raw_ty = if f.is_field_raw_ty_byte_array() {
                    &slicetype
                } else {
                    &f.ty
                };
                getters.push(quote! {
                    #[doc = #field_comment]
                    #[inline]
                    pub fn #get_raw_name(&self) -> #raw_ty {
                        #(#get_value_lines)*
                        val
                    }
                });

                if f.map.convert_may_fail {
                    let get_val = get_raw_field_code(f, off, quote! { payload });
                    let is_valid_fn = &out_ty.is_valid_fn;
                    field_validators.push(quote! {
                        let val = #get_val;
                        if !#is_valid_fn(val) {
                            return Err(ParserError::InvalidField{
                                packet: #pack_name,
                                field: stringify!(#get_name)
                            });
                        }
                    });
                }
                let from_fn = &out_ty.from_fn;
                get_value_lines.push(quote! {
                    let val = #from_fn(val);
                });
            }

            if let Some(ref scale) = f.map.scale {
                get_value_lines.push(quote! { let val = val * #scale; });
            }
            getters.push(quote! {
                #[doc = #field_comment]
                #[inline]
                pub fn #get_name(&self) -> #ty {
                    #(#get_value_lines)*
                    val
                }
            });
            off += size_bytes;
        } else {
            assert!(field_index == pack_descr.fields.len() - 1 || f.size_fn().is_some());

            let range = if let Some(size_fn) = f.size_fn() {
                let range = quote! {
                    {
                        let offset = #off #(+ self.#size_fns())*;
                        offset..offset+self.#size_fn()
                    }
                };
                size_fns.push(size_fn);
                range
            } else {
                quote! { #off.. }
            };

            let mut get_value_lines = vec![quote! { &self.0[#range] }];
            if let Some(ref out_ty) = f.map.map_type {
                let get_raw = &get_value_lines[0];
                let new_line = quote! { let val = #get_raw ;  };
                get_value_lines[0] = new_line;
                let from_fn = &out_ty.from_fn;
                get_value_lines.push(quote! {
                    #from_fn(val)
                });

                if f.map.convert_may_fail {
                    let is_valid_fn = &out_ty.is_valid_fn;
                    field_validators.push(quote! {
                        let val = &payload[#off..];
                        if !#is_valid_fn(val) {
                            return Err(ParserError::InvalidField{
                                packet: #pack_name,
                                field: stringify!(#get_name)
                            });
                        }
                    });
                }
            }
            let out_ty = if f.has_intermediate_type() {
                ty.clone()
            } else {
                parse_quote! { &[u8] }
            };
            getters.push(quote! {
                #[doc = #field_comment]
                #[inline]
                pub fn #get_name(&self) -> #out_ty {
                    #(#get_value_lines)*
                }
            });
        }
    }
    let struct_comment = &pack_descr.comment;
    let validator = if let Some(payload_len) = pack_descr.packet_payload_size() {
        quote! {
            fn validate(payload: &[u8]) -> Result<(), ParserError> {
                let expect = #payload_len;
                let got = payload.len();
                if got ==  expect {
                    #(#field_validators)*
                    Ok(())
                } else {
                    Err(ParserError::InvalidPacketLen{ packet: #pack_name, expect, got })
                }
            }
        }
    } else {
        let size_fns: Vec<_> = pack_descr
            .fields
            .iter()
            .filter_map(|f| f.size_fn())
            .collect();

        let min_size = if size_fns.is_empty() {
            let size = pack_descr
                .packet_payload_size_except_last_field()
                .expect("except last all fields should have fixed size");
            quote! {
                #size;
            }
        } else {
            let size = pack_descr
                .packet_payload_size_except_size_fn()
                .unwrap_or_default();
            quote! {
                {
                    if got < #size {
                        return Err(ParserError::InvalidPacketLen{ packet: #pack_name, expect: #size, got });
                    }
                    #size #(+ #ref_name(payload).#size_fns())*
                }
            }
        };

        quote! {
            fn validate(payload: &[u8]) -> Result<(), ParserError> {
                let got = payload.len();
                let min = #min_size;
                if got >= min {
                    #(#field_validators)*
                    Ok(())
                } else {
                    Err(ParserError::InvalidPacketLen{ packet: #pack_name, expect: min, got })
                }
            }
        }
    };

    let debug_impl = generate_debug_impl(pack_name, &ref_name, pack_descr);
    let serialize_impl = generate_serialize_impl(pack_name, &ref_name, pack_descr);

    quote! {
        #[doc = #struct_comment]
        #[doc = "Contains a reference to an underlying buffer, contains accessor methods to retrieve data."]
        pub struct #ref_name<'a>(&'a [u8]);
        impl<'a> #ref_name<'a> {
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.0
            }

            #(#getters)*

            #validator
        }

        #debug_impl
        #serialize_impl
    }
}

pub fn generate_types_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let name = Ident::new(&pack_descr.name, Span::call_site());
    let class = pack_descr.header.class;
    let id = pack_descr.header.id;
    let fixed_payload_len = match pack_descr.header.payload_len.fixed() {
        Some(x) => quote! { Some(#x) },
        None => quote! { None },
    };
    let struct_comment = &pack_descr.comment;
    let max_payload_len = match pack_descr.header.payload_len {
        PayloadLen::Fixed(x) => x,
        PayloadLen::Max(x) => x,
    };
    quote! {

        #[doc = #struct_comment]
        pub struct #name;
        impl UbxPacketMeta for #name {
            const CLASS: u8 = #class;
            const ID: u8 = #id;
            const FIXED_PAYLOAD_LEN: Option<u16> = #fixed_payload_len;
            const MAX_PAYLOAD_LEN: u16 = #max_payload_len;
        }
    }
}

pub fn generate_send_code_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let main_name = Ident::new(&pack_descr.name, Span::call_site());
    let payload_struct = format_ident!("{}Builder", pack_descr.name);

    let mut builder_needs_lifetime = false;

    let mut fields = Vec::with_capacity(pack_descr.fields.len());
    let mut pack_fields = Vec::with_capacity(pack_descr.fields.len());
    let mut write_fields = Vec::with_capacity(pack_descr.fields.len());
    let mut extend_fields = Vec::with_capacity(pack_descr.fields.len());
    let mut off = 6usize;
    for (fi, f) in pack_descr.fields.iter().enumerate() {
        let ty = f.intermediate_type();
        let name = f.intermediate_field_name();
        let field_comment = &f.comment;
        fields.push(quote! {
            #[doc = #field_comment]
            pub #name: #ty
        });

        let size_bytes = match f.size_bytes {
            Some(x) => x.get(),
            None => {
                // Iterator with `data` field.
                extend_fields.push(quote! {
                    for f in self.#name {
                      len_bytes += f.extend_to(out);
                    }
                });

                builder_needs_lifetime = true;

                assert_eq!(
                    fi,
                    pack_descr.fields.len() - 1,
                    "Iterator field must be the last field."
                );
                break;
            },
        };

        if let Some(into_fn) = f.map.map_type.as_ref().map(|x| &x.into_fn) {
            pack_fields.push(quote! {
                let bytes = #into_fn(self.#name).to_le_bytes()
            });
        } else if !f.is_field_raw_ty_byte_array() {
            pack_fields.push(quote! {
              let bytes = self.#name.to_le_bytes()
            });
        } else {
            pack_fields.push(quote! {
              let bytes: &[u8] = &self.#name;
            });
        }

        write_fields.push(pack_fields.last().unwrap().clone());
        write_fields.push(quote! {
            out.write(&bytes)?;
            checksum_calc.update(&bytes)
        });

        extend_fields.push(pack_fields.last().unwrap().clone());
        extend_fields.push(quote! {
            len_bytes += bytes.len();
            // TODO: Extend all at once when we bump MSRV
            //out.extend(bytes.into_iter().cloned());
            for b in bytes.iter() {
                out.extend(core::iter::once(*b));
            }
        });

        for i in 0..size_bytes {
            let byte_off = off.checked_add(i).unwrap();
            pack_fields.push(quote! {
                ret[#byte_off] = bytes[#i]
            });
        }

        off += size_bytes;
    }
    let builder_attr = if pack_descr
        .header
        .flags
        .iter()
        .any(|x| *x == PacketFlag::DefaultForBuilder)
    {
        quote! { #[derive(Default)] }
    } else {
        quote! {}
    };
    let struct_comment = &pack_descr.comment;

    let payload_struct_lifetime = if builder_needs_lifetime {
        quote! { <'a> }
    } else {
        quote! {}
    };

    let mut ret = quote! {
        #[doc = #struct_comment]
        #[doc = "Struct that is used to construct packets, see the crate-level documentation for more information"]
        #builder_attr
        pub struct #payload_struct #payload_struct_lifetime {
            #(#fields),*
        }
    };

    if let Some(packet_payload_size) = pack_descr.packet_payload_size() {
        let packet_size = packet_payload_size + 8;
        let packet_payload_size_u16 = u16::try_from(packet_payload_size).unwrap();
        ret.extend(quote! {
            impl #payload_struct_lifetime #payload_struct #payload_struct_lifetime {
                pub const PACKET_LEN: usize = #packet_size;

                #[inline]
                pub fn into_packet_bytes(self) -> [u8; Self::PACKET_LEN] {
                    let mut ret = [0u8; Self::PACKET_LEN];
                    ret[0] = SYNC_CHAR_1;
                    ret[1] = SYNC_CHAR_2;
                    ret[2] = #main_name::CLASS;
                    ret[3] = #main_name::ID;
                    let pack_len_bytes = #packet_payload_size_u16 .to_le_bytes();
                    ret[4] = pack_len_bytes[0];
                    ret[5] = pack_len_bytes[1];
                    #(#pack_fields);*;
                    let (ck_a, ck_b) = ubx_checksum(&ret[2..(Self::PACKET_LEN - 2)]);
                    ret[Self::PACKET_LEN - 2] = ck_a;
                    ret[Self::PACKET_LEN - 1] = ck_b;
                    ret
                }
            }
            impl From<#payload_struct> for [u8; #packet_size] {
                fn from(x: #payload_struct) -> Self {
                    x.into_packet_bytes()
                }
            }

            impl UbxPacketCreator for #payload_struct {
                #[inline]
                fn create_packet<T: MemWriter>(self, out: &mut T) -> Result<(), MemWriterError<T::Error>> {
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
        ret.extend(quote! {
          impl #payload_struct_lifetime #payload_struct #payload_struct_lifetime {
              #[cfg(feature = "alloc")]
              #[inline]
              pub fn into_packet_vec(self) -> Vec<u8> {
                let mut vec = Vec::new();
                self.extend_to(&mut vec);
                vec
              }

              #[inline]
              pub fn extend_to<T>(self, out: &mut T)
              where
                 T: core::iter::Extend<u8> +
                    core::ops::DerefMut<Target = [u8]>
              {
                  // TODO: Enable when `extend_one` feature is stable.
                  // out.extend_reserve(6);
                  let mut len_bytes = 0;
                  let header = [SYNC_CHAR_1, SYNC_CHAR_2, #main_name::CLASS, #main_name::ID, 0, 0];
                  // TODO: Extend all at once when we bump MSRV
                  //out.extend(header);
                  for b in header.iter() {
                      out.extend(core::iter::once(*b));
                  }
                  #(#extend_fields);*;

                  let len_bytes = len_bytes.to_le_bytes();
                  out[4] = len_bytes[0];
                  out[5] = len_bytes[1];

                  let (ck_a, ck_b) = ubx_checksum(&out[2..]);
                  out.extend(core::iter::once(ck_a));
                  out.extend(core::iter::once(ck_b));
              }
          }
        })
    }

    ret
}

pub fn generate_code_to_extend_enum(ubx_enum: &UbxExtendEnum) -> TokenStream {
    assert_eq!(ubx_enum.repr, {
        let ty: Type = parse_quote! { u8 };
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
        },
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
                    fn is_valid(x: #repr_ty) -> bool {
                        match x {
                            #(#values)* => true,
                            _ => false,
                        }
                    }
                }
            }
        },
        None => quote! {},
    };

    let to_code = match ubx_enum.into_fn {
        None => quote! {},
        Some(UbxTypeIntoFn::Raw) => quote! {
            impl #name {
                const fn into_raw(self) -> #repr_ty {
                    self as #repr_ty
                }
            }
        },
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
        #to_code

        #[cfg(feature = "serde")]
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_u8(*self as u8)
            }
        }
    };
    code
}

pub fn generate_code_to_extend_bitflags(bitflags: BitFlagsMacro) -> syn::Result<TokenStream> {
    match bitflags.rest_handling {
        Some(UbxEnumRestHandling::ErrorProne) | None => {
            return Err(syn::Error::new(
                bitflags.name.span(),
                "Only reserved supported",
            ))
        },
        Some(UbxEnumRestHandling::Reserved) => (),
    }

    let mut known_flags = HashSet::new();
    let mut items = Vec::with_capacity(usize::try_from(bitflags.nbits).unwrap());
    let repr_ty = &bitflags.repr_ty;

    for bit in 0..bitflags.nbits {
        let flag_bit = 1u64.checked_shl(bit).unwrap();
        if let Some(item) = bitflags.consts.iter().find(|x| x.value == flag_bit) {
            known_flags.insert(flag_bit);
            let name = &item.name;
            let attrs = &item.attrs;
            if bit != 0 {
                items.push(quote! {
                    #(#attrs)*
                    const #name  = ((1 as #repr_ty) << #bit)
                });
            } else {
                items.push(quote! {
                    #(#attrs)*
                    const #name  = (1 as #repr_ty)
                });
            }
        } else {
            let name = format_ident!("RESERVED{}", bit);
            if bit != 0 {
                items.push(quote! { const #name = ((1 as #repr_ty) << #bit)});
            } else {
                items.push(quote! { const #name = (1 as #repr_ty) });
            }
        }
    }

    if known_flags.len() != bitflags.consts.len() {
        let user_flags: HashSet<_> = bitflags.consts.iter().map(|x| x.value).collect();
        let set = user_flags.difference(&known_flags);
        return Err(syn::Error::new(
            bitflags.name.span(),
            format!("Strange flags, not power of 2?: {:?}", set),
        ));
    }

    let vis = &bitflags.vis;
    let attrs = &bitflags.attrs;
    let name = &bitflags.name;

    let from = match bitflags.from_fn {
        None => quote! {},
        Some(UbxTypeFromFn::From) => quote! {
            impl #name {
                const fn from(x: #repr_ty) -> Self {
                    Self::from_bits_truncate(x)
                }
            }
        },
        Some(UbxTypeFromFn::FromUnchecked) => unimplemented!(),
    };

    let into = match bitflags.into_fn {
        None => quote! {},
        Some(UbxTypeIntoFn::Raw) => quote! {
            impl #name {
                const fn into_raw(self) -> #repr_ty {
                    self.bits()
                }
            }
        },
    };

    let serialize_fn = format_ident!("serialize_{}", repr_ty.to_token_stream().to_string());
    let serde = quote! {
        #[cfg(feature = "serde")]
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.#serialize_fn(self.bits())
            }
        }
    };

    Ok(quote! {
        bitflags! {
            #(#attrs)*
            #vis struct #name : #repr_ty {
                #(#items);*;
            }
        }
        #from
        #into
        #serde
    })
}

pub fn generate_code_for_parse(recv_packs: &RecvPackets) -> TokenStream {
    let union_enum_name = &recv_packs.union_enum_name;

    let mut pack_enum_variants = Vec::with_capacity(recv_packs.all_packets.len());
    let mut matches = Vec::with_capacity(recv_packs.all_packets.len());
    let mut class_id_matches = Vec::with_capacity(recv_packs.all_packets.len());
    let mut serializers = Vec::with_capacity(recv_packs.all_packets.len());

    for name in &recv_packs.all_packets {
        let ref_name = format_ident!("{}Ref", name);
        pack_enum_variants.push(quote! {
            #name(#ref_name <'a>)
        });

        matches.push(quote! {
            (#name::CLASS, #name::ID) if <#ref_name>::validate(payload).is_ok()  => {
                Ok(#union_enum_name::#name(#ref_name(payload)))
            }
        });

        class_id_matches.push(quote! {
            #union_enum_name::#name(_) => (#name::CLASS, #name::ID)
        });

        serializers.push(quote! {
            #union_enum_name::#name(ref msg) => crate::ubx_packets::PacketSerializer {
                class: #name::CLASS,
                msg_id: #name::ID,
                msg,
            }
            .serialize(serializer)
        });
    }

    let unknown_var = &recv_packs.unknown_ty;

    let max_payload_len_calc = recv_packs
        .all_packets
        .iter()
        .fold(quote! { 0u16 }, |prev, name| {
            quote! { max_u16(#name::MAX_PAYLOAD_LEN, #prev) }
        });

    quote! {
        #[doc = "All possible packets enum"]
        #[derive(Debug)]
        pub enum #union_enum_name<'a> {
            #(#pack_enum_variants),*,
            Unknown(#unknown_var<'a>)
        }

        impl<'a> #union_enum_name<'a> {
            pub fn class_and_msg_id(&self) -> (u8, u8) {
                match *self {
                    #(#class_id_matches),*,
                    #union_enum_name::Unknown(ref pack) => (pack.class, pack.msg_id),
                }
            }
        }

        pub(crate) fn match_packet(class: u8, msg_id: u8, payload: &[u8]) -> Result<#union_enum_name, ParserError> {
            match (class, msg_id) {
                #(#matches)*
                _ => Ok(#union_enum_name::Unknown(#unknown_var {
                    payload,
                    class,
                    msg_id
                })),
            }
        }

        const fn max_u16(a: u16, b: u16) -> u16 {
            [a, b][(a < b) as usize]
        }
        pub(crate) const MAX_PAYLOAD_LEN: u16 = #max_payload_len_calc;
        #[cfg(feature = "serde")]
        pub struct PacketSerializer<'a, T> {
            class: u8,
            msg_id: u8,
            msg: &'a T,
        }

        #[cfg(feature = "serde")]
        impl<'a, T: SerializeUbxPacketFields> serde::Serialize for PacketSerializer<'a, T> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut state = serializer.serialize_map(None)?;
                state.serialize_entry("class", &self.class)?;
                state.serialize_entry("msg_id", &self.msg_id)?;
                self.msg.serialize_fields(&mut state)?;
                state.end()
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for #union_enum_name<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match *self {
                    #(#serializers),*,
                    #union_enum_name::Unknown(ref pack) => pack.serialize(serializer),
                }
            }
        }
    }
}

fn get_raw_field_code(field: &PackField, cur_off: usize, data: TokenStream) -> TokenStream {
    let size_bytes = match field.size_bytes {
        Some(x) => x,
        None => unimplemented!(),
    };

    let mut bytes = Vec::with_capacity(size_bytes.get());
    for i in 0..size_bytes.get() {
        let byte_off = cur_off.checked_add(i).unwrap();
        bytes.push(quote! { #data[#byte_off] });
    }
    let raw_ty = &field.ty;

    let signed_byte: Type = parse_quote! { i8 };

    if field.map.get_as_ref {
        let size_bytes: usize = size_bytes.into();
        quote! { &#data[#cur_off .. (#cur_off + #size_bytes)] }
    } else if field.is_field_raw_ty_byte_array() {
        quote! { [#(#bytes),*] }
    } else if size_bytes.get() != 1 || *raw_ty == signed_byte {
        quote! { <#raw_ty>::from_le_bytes([#(#bytes),*]) }
    } else {
        quote! { #data[#cur_off] }
    }
}
