use crate::{output::match_packet, types::recvpackets::RecvPackets};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_code_for_parse(recv_packs: &RecvPackets) -> TokenStream {
    let union_enum_name = format_ident!("{}", &recv_packs.union_enum_name);

    let mut pack_enum_variants = Vec::with_capacity(recv_packs.all_packets.len());

    let mut matches = Vec::with_capacity(recv_packs.all_packets.len());

    let mut class_id_matches = Vec::with_capacity(recv_packs.all_packets.len());

    let mut serializers = Vec::with_capacity(recv_packs.all_packets.len());

    let mut len_matches = Vec::with_capacity(recv_packs.all_packets.len());

    let mut into_owned_matches = Vec::with_capacity(recv_packs.all_packets.len());

    for name in &recv_packs.all_packets {
        let pack_name = format_ident!("{}", name);
        pack_enum_variants.push(quote! {
            #name(#pack_name <'a>)
        });

        matches.push(quote! {
            (#name::CLASS, #name::ID) if <#pack_name>::validate(payload).is_ok()  => {
                Ok(#union_enum_name::#name(#pack_name::new_borrowed(payload)))
            }
        });

        class_id_matches.push(quote! {
            #union_enum_name::#name(_) => (#name::CLASS, #name::ID)
        });

        serializers.push(quote! {
            #union_enum_name::#name(ref msg) => PacketSerializer {
                class: #name::CLASS,
                msg_id: #name::ID,
                msg,
            }
            .serialize(serializer)
        });

        len_matches.push(quote! {
            #union_enum_name::#name(ref packet) => packet.payload_len(),
        });

        into_owned_matches.push(quote! {
            #union_enum_name::#name(packet) => {
                let owned_data = packet.into_owned();
                #union_enum_name::#name(#pack_name {
                    buffer: PacketBuffer::Owned(owned_data),
                })
            }
        });
    }

    let unknown_var = format_ident!("{}", &recv_packs.unknown_ty);

    let max_payload_len_calc = recv_packs
        .all_packets
        .iter()
        .fold(quote! { 0u16 }, |prev, name| {
            quote! { max_u16(#name::MAX_PAYLOAD_LEN, #prev) }
        });

    let fn_match_packet =
        match_packet::generate_fn_match_packet(&union_enum_name, &matches, &unknown_var);

    quote! {
        #[doc = "All possible packets enum"]
        #[derive(Debug)]
        #[non_exhaustive]
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

            pub fn into_owned(self) -> Self {
                match self {
                    #(#into_owned_matches),*,
                    #union_enum_name::Unknown(pack) => {
                        #union_enum_name::Unknown(pack)
                    }
                }
            }

            #[inline]
            pub fn payload_len(&self) -> usize {
                match *self {
                    #(#len_matches)*
                    #union_enum_name::Unknown(ref pack) => pack.payload_len,
                }
            }
        }

        #fn_match_packet

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
