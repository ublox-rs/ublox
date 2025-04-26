use syn::{braced, parse::Parse, punctuated::Punctuated, Ident, Token};

pub struct RecvPackets {
    pub union_enum_name: Ident,
    pub unknown_ty: Ident,
    pub all_packets: Vec<Ident>,
}

impl Parse for RecvPackets {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![enum]>()?;
        let union_enum_name: Ident = input.parse()?;
        let content;
        let _brace_token: syn::token::Brace = braced!(content in input);
        content.parse::<Token![_]>()?;
        content.parse::<Token![=]>()?;
        let unknown_ty: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let packs: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse)?;
        Ok(Self {
            union_enum_name,
            unknown_ty,
            all_packets: packs.into_iter().collect(),
        })
    }
}
