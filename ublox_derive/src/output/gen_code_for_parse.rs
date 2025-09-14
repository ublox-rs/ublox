use crate::{output::match_packet, types::recvpackets::RecvPackets};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_code_for_parse(recv_packs: &RecvPackets) -> TokenStream {
    let union_enum_name_ref = format_ident!("{}Ref", &recv_packs.union_enum_name);
    let union_enum_name_owned = format_ident!("{}Owned", &recv_packs.union_enum_name);

    let mut pack_enum_variants_ref = Vec::with_capacity(recv_packs.all_packets.len());
    let mut pack_enum_variants_owned = Vec::with_capacity(recv_packs.all_packets.len());

    let mut matches_ref = Vec::with_capacity(recv_packs.all_packets.len());
    let mut matches_owned = Vec::with_capacity(recv_packs.all_packets.len());
    let mut matches_ref_to_owned = Vec::with_capacity(recv_packs.all_packets.len());

    let mut class_id_matches_ref = Vec::with_capacity(recv_packs.all_packets.len());
    let mut class_id_matches_owned = Vec::with_capacity(recv_packs.all_packets.len());

    let mut serializers = Vec::with_capacity(recv_packs.all_packets.len());

    for name in &recv_packs.all_packets {
        let ref_name = format_ident!("{}Ref", name);
        let owned_name = format_ident!("{}Owned", name);
        pack_enum_variants_ref.push(quote! {
            #name(#ref_name <'a>)
        });
        pack_enum_variants_owned.push(quote! {
            #name(#owned_name)
        });

        matches_ref.push(quote! {
            (#name::CLASS, #name::ID) if <#ref_name>::validate(payload).is_ok()  => {
                Ok(#union_enum_name_ref::#name(#ref_name(payload)))
            }
        });

        matches_owned.push(quote! {
            (#name::CLASS, #name::ID) if <#owned_name>::validate(payload).is_ok()  => {
                let mut bytes = [0u8; #owned_name::PACKET_SIZE];
                bytes.clone_from_slice(payload);
                Ok(#union_enum_name_owned::#name(#owned_name(bytes)))
            }
        });
        matches_ref_to_owned.push(quote! {
            #union_enum_name_ref::#name(packet) => #union_enum_name_owned::#name(packet.into()),
        });

        class_id_matches_ref.push(quote! {
            #union_enum_name_ref::#name(_) => (#name::CLASS, #name::ID)
        });
        class_id_matches_owned.push(quote! {
            #union_enum_name_owned::#name(_) => (#name::CLASS, #name::ID)
        });

        serializers.push(quote! {
            #union_enum_name_ref::#name(ref msg) => PacketSerializer {
                class: #name::CLASS,
                msg_id: #name::ID,
                msg,
            }
            .serialize(serializer)
        });
    }

    let unknown_var_ref = format_ident!("{}Ref", &recv_packs.unknown_ty);
    let unknown_var_owned = format_ident!("{}Owned", &recv_packs.unknown_ty);

    let max_payload_len_calc = recv_packs
        .all_packets
        .iter()
        .fold(quote! { 0u16 }, |prev, name| {
            quote! { max_u16(#name::MAX_PAYLOAD_LEN, #prev) }
        });

    let unknown_conversion = quote! {
        #union_enum_name_ref::Unknown(#unknown_var_ref {payload, class, msg_id}) => {
            let mut payload_copy = [0u8; MAX_PAYLOAD_LEN as usize];
            let payload_len = core::cmp::min(payload.len(), MAX_PAYLOAD_LEN as usize);
            payload_copy[..payload_len].copy_from_slice(&payload[..payload_len]);

            #union_enum_name_owned::Unknown(#unknown_var_owned {
                payload: payload_copy,
                payload_len,
                class: *class,
                msg_id: *msg_id
            })
        }
    };

    let fn_match_packet = match_packet::generate_fn_match_packet(
        &union_enum_name_ref,
        &matches_ref,
        &unknown_var_ref,
    );

    let fn_match_packet_owned = match_packet::generate_fn_match_packet_owned(
        &union_enum_name_owned,
        &matches_owned,
        &unknown_var_owned,
    );

    quote! {
        #[doc = "All possible packets enum"]
        #[derive(Debug)]
        pub enum #union_enum_name_ref<'a> {
            #(#pack_enum_variants_ref),*,
            Unknown(#unknown_var_ref<'a>)
        }
        #[doc = "All possible packets enum, owning the underlying data"]
        #[derive(Debug)]
        pub enum #union_enum_name_owned {
            #(#pack_enum_variants_owned),*,
            Unknown(#unknown_var_owned<{MAX_PAYLOAD_LEN as usize}>)
        }

        impl<'a> #union_enum_name_ref<'a> {
            pub fn class_and_msg_id(&self) -> (u8, u8) {
                match *self {
                    #(#class_id_matches_ref),*,
                    #union_enum_name_ref::Unknown(ref pack) => (pack.class, pack.msg_id),
                }
            }

            pub fn to_owned(&self) -> #union_enum_name_owned {
                self.into()
            }
        }
        impl #union_enum_name_owned {
            pub fn class_and_msg_id(&self) -> (u8, u8) {
                match *self {
                    #(#class_id_matches_owned),*,
                    #union_enum_name_owned::Unknown(ref pack) => (pack.class, pack.msg_id),
                }
            }
        }

        #fn_match_packet
        #fn_match_packet_owned


        impl<'a> From<&#union_enum_name_ref<'a>> for #union_enum_name_owned {
            fn from(packet: &#union_enum_name_ref<'a>) -> Self {
                match packet {
                    #(#matches_ref_to_owned)*
                    #unknown_conversion
                }
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
        impl serde::Serialize for #union_enum_name_ref<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match *self {
                    #(#serializers),*,
                    #union_enum_name_ref::Unknown(ref pack) => pack.serialize(serializer),
                }
            }
        }
    }
}
