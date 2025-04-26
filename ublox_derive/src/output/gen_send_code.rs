use crate::types::packetflag::PacketFlag;
use crate::types::PackDesc;
use crate::util::DebugContext;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

pub fn generate_send_code_for_packet(dbg_ctx: DebugContext, pack_descr: &PackDesc) -> TokenStream {
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
            out.extend(bytes);
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
        pack_descr
            .lifetime_tokens()
            .expect("builder needs lifetime but no lifetime was specified")
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

    dbg_ctx.print_at(
        file!(),
        line!(),
        format_args!("initial payload struct: {}", &ret),
    );
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
                  // Tracking issue: https://github.com/rust-lang/rust/issues/72631
                  // out.extend_reserve(6);
                  let mut len_bytes = 0;
                  let header = [SYNC_CHAR_1, SYNC_CHAR_2, #main_name::CLASS, #main_name::ID, 0, 0];
                  out.extend(header);

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

    dbg_ctx.print_at(
        file!(),
        line!(),
        format_args!("final payload struct: {}", &ret),
    );
    ret
}
