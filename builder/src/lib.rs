use proc_macro::TokenStream;
//use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
//use syn::{parse_macro_input, DeriveInput, Ident};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    //    println!("{:#?}", ast);
    let name = ast.ident;
    //let bname = format!("{}Builder", name);
    //let bident = Ident::new(&bname, name.span());
    //let bident = Ident::new(&bname, Span::call_site());
    let bident = format_ident!("{}Builder", name);
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };
    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        //quote! { #name: std::option::Option<#ty> }
        quote! { #name: Option<#ty> }
    });
    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });
    let build_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });
    let expanded = quote! {
        pub struct #bident {
            #(#optionized,)*
        }
        impl #name {
            pub fn builder() -> #bident {
                #bident {
                    #(#build_empty,)*
                //    executable: None,
                //    args: None,
                //    env: None,
                //    current_dir: None,
                }
            }
        }
        impl #bident {
            #(#methods)*

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    };

    expanded.into()
}
