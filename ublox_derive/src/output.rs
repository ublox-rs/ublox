use crate::types::recvpackets::RecvPackets;
use crate::types::BitFlagsMacro;
use crate::types::{PackDesc, PayloadLen, UbxEnumRestHandling, UbxTypeFromFn, UbxTypeIntoFn};
use crate::util::DebugContext;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{collections::HashSet, convert::TryFrom};
use syn::{parse_quote, Ident};

pub(crate) mod extend_enum;
pub(crate) mod gen_code_for_parse;
pub(crate) mod gen_send_code;
mod match_packet;
mod util;

pub fn generate_recv_code_for_packet(dbg_ctx: DebugContext, pack_descr: &PackDesc) -> TokenStream {
    let pack_name = &pack_descr.name;
    let ref_name = format_ident!("{}Ref", pack_descr.name);
    let owned_name = format_ident!("{}Owned", pack_descr.name);
    let packet_size = match pack_descr.header.payload_len {
        PayloadLen::Fixed(value) => value,
        PayloadLen::Max(value) => value,
    } as usize;

    let mut getters = Vec::with_capacity(pack_descr.fields.len());
    let mut field_validators = Vec::new();
    let mut size_fns = Vec::new();

    let mut off = 0usize;
    for (field_index, f) in pack_descr.fields.iter().enumerate() {
        let ty = f.intermediate_type();
        let get_name = f.intermediate_field_name();
        let field_comment = &f.comment;

        if let Some(size_bytes) = f.size_bytes.map(|x| x.get()) {
            let get_raw = util::get_raw_field_code(f, off, quote! {self.0});
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
                    let get_val = util::get_raw_field_code(f, off, quote! { payload });
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

    let debug_impl = util::generate_debug_impl(pack_name, &ref_name, &owned_name, pack_descr);
    let serialize_impl = util::generate_serialize_impl(pack_name, &ref_name, pack_descr);

    quote! {
        #[doc = #struct_comment]
        #[doc = "Contains a reference to an underlying buffer, contains accessor methods to retrieve data."]
        pub struct #ref_name<'a>(&'a [u8]);
        impl<'a> #ref_name<'a> {
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.0
            }

            pub fn to_owned(&self) -> #owned_name {
                self.into()
            }

            #(#getters)*

            #validator
        }

        #[doc = #struct_comment]
        #[doc = "Owns the underlying buffer of data, contains accessor methods to retrieve data."]
        pub struct #owned_name([u8; #packet_size]);

        impl #owned_name {
            const PACKET_SIZE: usize = #packet_size;

            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            #(#getters)*

            #validator
        }

        impl<'a> From<&#ref_name<'a>> for #owned_name {
            fn from(packet: &#ref_name<'a>) -> Self {
                let mut bytes = [0u8; #packet_size];
                bytes.clone_from_slice(packet.as_bytes());
                Self(bytes)
            }
        }

        impl<'a> From<#ref_name<'a>> for #owned_name {
            fn from(packet: #ref_name<'a>) -> Self {
                (&packet).into()
            }
        }

        #debug_impl
        #serialize_impl
    }
}

pub fn generate_types_for_packet(dbg_ctx: DebugContext, pack_descr: &PackDesc) -> TokenStream {
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
