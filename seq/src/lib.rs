use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Token,
};

#[derive(Debug)]
struct SeqMacroInput {
    ident: syn::Ident,
    from: syn::LitInt,
    to: syn::LitInt,
    tt: proc_macro2::TokenStream,
}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = syn::Ident::parse(input)?;
        let _in = <Token![in]>::parse(input)?;
        let from = syn::LitInt::parse(input)?;
        let _dots = <Token![..]>::parse(input)?;
        let to = syn::LitInt::parse(input)?;
        let content;
        let _braces = syn::braced!(content in input);
        let tt = proc_macro2::TokenStream::parse(&content)?;
        // println!("{:#?}", tt);

        Ok(SeqMacroInput {
            ident,
            from,
            to,
            tt,
        })
    }
}

impl From<SeqMacroInput> for proc_macro2::TokenStream {
    fn from(seq_macro: SeqMacroInput) -> Self {
        (seq_macro.from.base10_parse::<u64>().unwrap()..seq_macro.to.base10_parse::<u64>().unwrap())
            .map(|i| seq_macro.expand(seq_macro.tt.clone(), i))
            .collect()
    }
}

// impl Into<proc_macro2::TokenStream> for SeqMacroInput {
//     fn into(self) -> proc_macro2::TokenStream {
//         // (self.from.value()..self.to.value())
//         (self.from.base10_parse::<u64>().unwrap()..self.to.base10_parse::<u64>().unwrap())
//             .map(|i| self.expand(self.tt.clone(), i))
//             .collect()
//     }
// }

impl SeqMacroInput {
    fn expand(&self, stream: proc_macro2::TokenStream, i: u64) -> proc_macro2::TokenStream {
        stream.into_iter().map(|tt| self.expand2(tt, i)).collect()
    }

    fn expand2(&self, tt: proc_macro2::TokenTree, i: u64) -> proc_macro2::TokenTree {
        match tt {
            proc_macro2::TokenTree::Group(g) => {
                let mut expanded =
                    proc_macro2::Group::new(g.delimiter(), self.expand(g.stream(), i));
                expanded.set_span(g.span());
                proc_macro2::TokenTree::Group(expanded)
            },
            proc_macro2::TokenTree::Ident(ref ident) if ident == &self.ident => {
                let mut lit = proc_macro2::Literal::u64_unsuffixed(i);
                lit.set_span(ident.span());
                proc_macro2::TokenTree::Literal(lit)
            },
            tt => tt,
        }
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SeqMacroInput);
    // println!("{:#?}", input);
    let output: proc_macro2::TokenStream = input.into();
    output.into()
    // TokenStream::new()
}
