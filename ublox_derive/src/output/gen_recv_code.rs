use crate::debug::DebugContext;
use crate::output::util;
use crate::types::packfield::PackField;
use crate::types::{PackDesc, PayloadLen};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;

pub fn generate_recv_code_for_packet(dbg_ctx: DebugContext, pack_descr: &PackDesc) -> TokenStream {
    let pack_name: &String = &pack_descr.name;
    let ref_name: syn::Ident = format_ident!("{}Ref", pack_descr.name);
    let owned_name: syn::Ident = format_ident!("{}Owned", pack_descr.name);
    let packet_size: usize = match pack_descr.header.payload_len {
        PayloadLen::Fixed(value) => value,
        PayloadLen::Max(value) => value,
    }
    .into();

    let mut getters: Vec<TokenStream> = Vec::with_capacity(pack_descr.fields.len());
    let mut field_validators: Vec<TokenStream> = Vec::new();
    let mut size_fns: Vec<&TokenStream> = Vec::new();

    let mut off = 0usize;
    process_fields(
        dbg_ctx,
        pack_descr,
        pack_name,
        &mut off,
        &mut getters,
        &mut field_validators,
        &mut size_fns,
    );

    let struct_comment = &pack_descr.comment;
    let validator = generate_validator(pack_descr, pack_name, &ref_name, field_validators);
    let debug_impl = util::generate_debug_impl(pack_name, &ref_name, &owned_name, pack_descr);
    let serialize_impl = util::generate_serialize_impl(pack_name, &ref_name, pack_descr);
    let from_ref_impl = generate_from_ref_impl(&ref_name, &owned_name, packet_size);

    quote! {
        #[doc = #struct_comment]
        #[doc = "Contains a reference to an underlying buffer, contains accessor methods to retrieve data."]
        pub struct #ref_name<'a>(pub(crate) &'a [u8]);
        impl<'a> #ref_name<'a> {
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.0
            }

            #[inline]
            pub fn payload_len(&self) -> usize {
                self.0.len()
            }

            pub fn to_owned(&self) -> #owned_name {
                self.into()
            }

            #(#getters)*

            #validator
        }

        #[doc = #struct_comment]
        #[doc = "Owns the underlying buffer of data, contains accessor methods to retrieve data."]
        pub struct #owned_name(pub(crate) [u8; #packet_size]);

        impl #owned_name {
            pub(crate) const PACKET_SIZE: usize = #packet_size;

            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            #(#getters)*

            #validator
        }

        #from_ref_impl
        #debug_impl
        #serialize_impl
    }
}

fn generate_validator(
    pack_descr: &PackDesc,
    pack_name: &String,
    ref_name: &syn::Ident,
    field_validators: Vec<TokenStream>,
) -> TokenStream {
    let validator = if let Some(payload_len) = pack_descr.packet_payload_size() {
        quote! {
            pub(crate) fn validate(payload: &[u8]) -> Result<(), ParserError> {
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
            pub(crate) fn validate(payload: &[u8]) -> Result<(), ParserError> {
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
    validator
}

fn generate_from_ref_impl(
    ref_name: &syn::Ident,
    owned_name: &syn::Ident,
    packet_size: usize,
) -> TokenStream {
    quote! {
        impl<'a> From<&#ref_name<'a>> for #owned_name {
            fn from(packet: &#ref_name<'a>) -> Self {
                let src = packet.as_bytes();
                let mut dst = [0u8; #packet_size];
                dst[..src.len()].clone_from_slice(src);
                Self(dst)
            }
        }

        impl<'a> From<#ref_name<'a>> for #owned_name {
            fn from(packet: #ref_name<'a>) -> Self {
                (&packet).into()
            }
        }
    }
}

fn process_fields<'a>(
    dbg_ctx: DebugContext,
    pack_descr: &'a PackDesc,
    pack_name: &String,
    off: &mut usize,
    getters: &mut Vec<TokenStream>,
    field_validators: &mut Vec<TokenStream>,
    size_fns: &mut Vec<&'a TokenStream>,
) {
    for (field_index, f) in pack_descr.fields.iter().enumerate() {
        let ty: &syn::Type = f.intermediate_type();

        let get_name = f.intermediate_field_name();
        let field_comment = &f.comment;

        if let Some(size_bytes) = f.size_bytes.map(|x| x.get()) {
            process_fixed_size_field(
                f,
                pack_name,
                field_comment,
                get_name,
                ty,
                *off,
                getters,
                field_validators,
            );
            *off += size_bytes;
        } else {
            assert!(field_index == pack_descr.fields.len() - 1 || f.size_fn().is_some());
            process_variable_size_field(
                dbg_ctx,
                f,
                pack_name,
                field_comment,
                get_name,
                ty,
                *off,
                getters,
                field_validators,
                size_fns,
            );
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "Yes we need to refactor...")]
fn process_fixed_size_field(
    f: &PackField,
    pack_name: &String,
    field_comment: &str,
    get_name: &syn::Ident,
    ty: &syn::Type,
    off: usize,
    getters: &mut Vec<TokenStream>,
    field_validators: &mut Vec<TokenStream>,
) {
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
}

#[allow(clippy::too_many_arguments, reason = "Yes we need to refactor...")]
fn process_variable_size_field<'a>(
    dbg_ctx: DebugContext,
    f: &'a PackField,
    pack_name: &String,
    field_comment: &str,
    get_name: &syn::Ident,
    ty: &syn::Type,
    off: usize,
    getters: &mut Vec<TokenStream>,
    field_validators: &mut Vec<TokenStream>,
    size_fns: &mut Vec<&'a TokenStream>,
) {
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
    let getter_out_ty = remove_lifetimes(out_ty.clone());

    let getter_def = quote! {
        #[doc = #field_comment]
        #[inline]
        pub fn #get_name(&self) -> #getter_out_ty {
            #(#get_value_lines)*
        }
    };
    dbg_ctx.print("getter def:");
    dbg_ctx.print_code(&getter_def);
    getters.push(getter_def);
}

fn remove_lifetimes(mut ty: syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = &mut ty {
        for segment in &mut type_path.path.segments {
            // Only process angle-bracketed args
            if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                // Filter out lifetimes
                args.args = args
                    .args
                    .clone()
                    .into_iter()
                    .filter(|arg| !matches!(arg, syn::GenericArgument::Lifetime(_)))
                    .collect();

                // If no args remain, clear the arguments entirely
                if args.args.is_empty() {
                    segment.arguments = syn::PathArguments::None;
                }
            }
        }
    }
    ty
}
