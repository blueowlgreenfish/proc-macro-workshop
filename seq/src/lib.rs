use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Token,
};

#[derive(Debug)]
struct SeqMacroInput {/* ... */}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let var = syn::Ident::parse(input)?;
        let _in = <Token![in]>::parse(input)?;
        let from = syn::LitInt::parse(input)?;
        let dots = <Token![..]>::parse(input)?;
        let _to = syn::LitInt::parse(input)?;
        // let _body = syn::Block::parse(input)?;
        let content;
        let braces = syn::braced!(content in input);
        // eprintln!("{:#?} {:#?} {:#?} {:#?} {:#?}", var, in, from, dots, braces);
        println!("{:#?}", _in);
        println!("{:#?} {:#?} {:#?} {:#?}", var, from, dots, braces);
        println!("{:#?}", content);
        let tt = proc_macro2::TokenStream::parse(&content)?;
        println!("{:#?}", tt);

        Ok(SeqMacroInput {})
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SeqMacroInput);
    println!("{:#?}", input);
    TokenStream::new()
}
