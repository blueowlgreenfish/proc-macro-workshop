use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);
    let ident = input.ident;
    let ident_string = ident.to_string();
    let data = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!();
    };
    let mut field_idents = Vec::new();
    for value in data.into_iter() {
        field_idents.push(value.ident.unwrap());
    }
    let smaller_expand = field_idents.iter().map(|f| {
        let field_idents_string = f.to_string();
        quote! { .field(#field_idents_string, &self.#f) }
    });
    let expand = quote! {
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#ident_string)
                    #(
                        #smaller_expand
                    )*
                    .finish()
            }
        }
    };

    expand.into()
}
