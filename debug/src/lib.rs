use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);
    let ident = input.ident;
    let ident_string = ident.to_string();
    let expand = quote! {
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#ident_string)
                    .field("name", &self.name)
                    .field("bitmask", &self.bitmask)
                    .finish()
            }
        }
    };

    expand.into()
}
