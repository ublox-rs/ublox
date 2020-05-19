use crate::types::BitFlagsMacro;
use crate::types::{
    PackDesc, PackField, PacketFlag, PayloadLen, RecvPackets, UbxEnumRestHandling, UbxExtendEnum,
    UbxTypeFromFn, UbxTypeIntoFn,
};
use crate::input::UbxBitfield;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::{collections::HashSet, convert::TryFrom};
use syn::{parse_quote, Ident, Type};
use syn::{Attribute, Fields};

pub fn generate_ubx_register(
    struct_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> TokenStream {
    println!("Generating ubx_register!");
    println!("{:?}", struct_name);
    let mut getters = vec!();
    for f in fields.iter() {
        println!("{:?}", f);
        let mut bits = UbxBitfield{ hi: 0, lo: 0 };
        for attr in f.attrs.iter() {
            let flags: UbxBitfield = attr.parse_args().unwrap();
            //println!("{:?}", flags);
            //break flags;
            bits = flags;
        };
        println!("{:?}", bits);
        println!("{:?}", f.ident);
        println!("{:?}", f.ty);
        let getter_name = format_ident!("get_{}", f.ident.as_ref().expect("Struct field must have a name"));
        let field_type = &f.ty;
        let lo = bits.lo;
        let mask: usize = (1 << (bits.hi - bits.lo + 1)) - 1;
        let is_bool = {
            if let syn::Type::Path(path) = field_type {
                format!("{}", path.path.get_ident().unwrap()) == "bool"
            } else {
                false
            }
        };
        // TODO: We shouldn't cast to usize
        if is_bool {
            // TODO: Check that hi == lo
            getters.push(quote!{
                pub fn #getter_name(&self) -> #field_type {
                    ((self.0 as usize >> #lo) & #mask) != 0
                }
            });
        } else {
            getters.push(quote!{
                pub fn #getter_name(&self) -> #field_type {
                    ((self.0 as usize >> #lo) & #mask).try_into().unwrap()
                }
            });
        }
    }
    quote!{
        use std::convert::TryInto;

        //struct #struct_name<'a>(&'a [u8; 1]);
        pub struct #struct_name(u8);

        impl #struct_name {
            /*pub fn get_field1(&self) -> bool {
                (self.0 & 0x1) != 0
            }

            pub fn get_field2(&self) -> u8 {
                (self.0 >> 1) & 0x7F
            }*/

            #(#getters)*

            const fn from(x: u8) -> Self {
                #struct_name(x)
            }
        }
    }
}
