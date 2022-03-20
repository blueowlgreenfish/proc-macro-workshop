use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
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
    let smaller_expand = data.into_iter().map(|f| {
        let mut lit = String::new();
        if !f.attrs.is_empty() {
            for attr in f.attrs {
                if let Ok(syn::Meta::NameValue(hi)) = attr.parse_meta() {
                    match hi.lit {
                        syn::Lit::Str(yes) => {
                            lit = yes.value();
                        },
                        _ => {
                            unimplemented!();
                        },
                    }
                }
            }
        };
        let field_ident = f.ident.as_ref().unwrap();
        let field_ident_string = field_ident.to_string();
        if !lit.is_empty() {
            quote! {
                // & needed in front of format_args! to retain binary form
                .field(#field_ident_string, &format_args!(#lit, &self.#field_ident))
            }
        } else {
            quote! {
                .field(#field_ident_string, &self.#field_ident)
            }
        }
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
