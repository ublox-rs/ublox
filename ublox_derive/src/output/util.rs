use crate::types::{packfield::PackField, PackDesc};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Ident, Type};

pub(super) fn generate_debug_impl(
    pack_name: &str,
    ref_name: &Ident,
    owned_name: &Ident,
    pack_descr: &PackDesc,
) -> TokenStream {
    let fields: Vec<TokenStream> = pack_descr
        .fields
        .iter()
        .map(|field| {
            let field_name = &field.name;
            let field_accessor = field.intermediate_field_name();
            quote! {
                .field(stringify!(#field_name), &self.#field_accessor())
            }
        })
        .collect();

    quote! {
        impl core::fmt::Debug for #ref_name<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(#pack_name)
                    #(#fields)*
                    .finish()
            }
        }
        impl core::fmt::Debug for #owned_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.debug_struct(#pack_name)
                    #(#fields)*
                    .finish()
            }
        }
    }
}

pub(super) fn generate_serialize_impl(
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
                    &FieldIter(self.#field_accessor())
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

pub(super) fn get_raw_field_code(
    field: &PackField,
    cur_off: usize,
    data: TokenStream,
) -> TokenStream {
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
