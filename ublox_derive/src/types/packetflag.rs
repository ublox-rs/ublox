use syn::parse::Parse;

use crate::input::keyword;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PacketFlag {
    DefaultForBuilder,
}

impl Parse for PacketFlag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keyword::default_for_builder) {
            input.parse::<keyword::default_for_builder>()?;
            Ok(PacketFlag::DefaultForBuilder)
        } else {
            Err(lookahead.error())
        }
    }
}
