use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    // let ty = dbg!(parse_macro_input!(input as syn::ItemEnum));
    let ty = parse_macro_input!(input as syn::ItemEnum);
    println!("{:#?}", ty);
    assert!(args.is_empty());
    let ts = quote! { #ty };
    ts.into()
}
