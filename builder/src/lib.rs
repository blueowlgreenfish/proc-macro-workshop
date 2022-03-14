use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn ty_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments[0].ident != "Option" {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(inner_ty) = &p.path.segments[0].arguments {
            let inner_ty = inner_ty.args.pairs().next().unwrap();
            if let syn::GenericArgument::Type(t) = inner_ty.value() {
                return Some(t);
            }
        }
    }
    None
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);
    let name = input.ident;
    let bname = format!("{}Builder", name);
    let bident = Ident::new(&bname, Span::call_site());
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!();
    };
    let optionized = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if ty_inner_type(ty).is_some() {
            quote! { #name: #ty }
        } else {
            quote! { #name: Option<#ty> }
        }
    });
    let methods = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if let Some(inner_ty) = ty_inner_type(ty) {
            quote! {
                fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });
    let build_fields = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if ty_inner_type(ty).is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }
    });
    let build_empty = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        quote! {
            #name: None
        }
    });
    let expanded = quote! {
    pub struct #bident {
        #(#optionized,)*
    }
    impl #name {
        fn builder() -> #bident {
            #bident {
                #(#build_empty,)*
            }
        }
    }
    impl #bident {
            #(#methods)*

            fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    };

    expanded.into()
}
