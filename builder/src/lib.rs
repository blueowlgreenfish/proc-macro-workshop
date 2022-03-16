use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments[0].ident != wrapper {
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

fn builder_of(f: &syn::Field) -> Option<syn::MetaList> {
    for attr in &f.attrs {
        if let Ok(syn::Meta::List(ms)) = attr.parse_meta() {
            // println!("{:#?}", ms);
            if ms.path.get_ident().unwrap() == "builder" {
                return Some(ms);
            } else {
                continue;
            }
        }
    }
    None
    // for attr in &f.attrs {
    //     if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "builder" {
    //         if let TokenTree::Group(g) = attr.tokens.clone().into_iter().next().unwrap() {
    //             return Some(g);
    //         }
    //     }
    // }
    // None
}

fn extend_method(f: &syn::Field) -> Option<(bool, proc_macro2::TokenStream)> {
    let name = f.ident.as_ref().unwrap();
    let ms = builder_of(f)?;
    let inner_ty = ms.nested.pairs().next().unwrap();
    let meta_value = if let syn::NestedMeta::Meta(meta) = inner_ty.value() {
        if let syn::Meta::NameValue(mnv) = meta {
            mnv
        } else {
            panic!();
        }
    } else {
        panic!();
    };

    // let mut tokens = g.stream().into_iter();
    // if let TokenTree::Ident(i) = tokens.next().unwrap() {
    //     assert_eq!(i, "each");
    // }
    // if let TokenTree::Punct(p) = tokens.next().unwrap() {
    //     assert_eq!(p.as_char(), '=');
    // }
    // let arg = if let TokenTree::Literal(l) = tokens.next().unwrap() {
    //     l
    // } else {
    //     unimplemented!();
    // };
    match &meta_value.lit {
        syn::Lit::Str(s) => {
            let arg = proc_macro2::Ident::new(&s.value(), Span::call_site());
            let inner_ty = ty_inner_type("Vec", &f.ty).unwrap();
            let method = quote! {
                pub fn #arg(&mut self, #arg: #inner_ty) -> &mut Self {
                    self.#name.push(#arg);
                    self
                }
            };
            Some((&arg == name, method))
        },
        lit => panic!("expected string, found {:?}", lit),
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);
    let name = input.ident;
    let bname = format!("{}Builder", name);
    let bident = proc_macro2::Ident::new(&bname, Span::call_site());
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
        if ty_inner_type("Option", ty).is_some() || builder_of(f).is_some() {
            quote! { #name: #ty }
        } else {
            quote! { #name: Option<#ty> }
        }
    });
    let methods = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let set_method = if let Some(inner_ty) = ty_inner_type("Option", ty) {
            quote! {
                fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else if builder_of(f).is_some() {
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
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
        };

        match extend_method(f) {
            None => set_method,
            Some((true, extend_method)) => extend_method,
            Some((false, extend_method)) => {
                let expr = quote! {
                    #set_method
                    #extend_method
                };
                expr
            },
        }
    });
    let build_fields = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() || builder_of(f).is_some() {
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
        if builder_of(f).is_some() {
            quote! { #name: Vec::new() }
        } else {
            quote! {
                #name: None
            }
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
