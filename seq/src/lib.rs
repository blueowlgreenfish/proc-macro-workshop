use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Token,
};

#[derive(Debug)]
struct SeqMacroInput {/* ... */}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ident = syn::Ident::parse(input)?;
        let _in = <Token![in]>::parse(input)?;
        let _from = syn::LitInt::parse(input)?;
        let _dots = <Token![..]>::parse(input)?;
        let _to = syn::LitInt::parse(input)?;
        let _body = syn::Block::parse(input)?;
        // println!("{:#?}", to);
        // eprintln!("{:?} {:?} {:?} {:?} {:?}", ident, _in, from, _dots, to);
        // eprintln!("{:?}", body);
        // eprintln!("{:#?}", ident);
        Ok(SeqMacroInput {})
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as SeqMacroInput);
    // println!("{:#?}", input);
    TokenStream::new()
}
