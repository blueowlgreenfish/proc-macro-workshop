use proc_macro::TokenStream;
//use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
//use syn::{parse_macro_input, DeriveInput, Ident};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    //eprintln!("{:#?}", ast);
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
        //        let original_type = f.ty.clone();
        //        let mut segments = syn::punctuated::Punctuated::new();
        //        segments.push_value(syn::syn::PathSegment {
        //            ident: syn::Ident::new("Option", )
        //        });
        //            let ty = syn::Type::Path(syn::TypePath {
        //                qself: None,
        //                path: syn::Path {
        //                    leading_colon: None,
        //                    segments,
        //                }
        //            });
        //        )
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: std::option::Option<#ty> }
        //    attrs: Vec::new(),
        //    vis: syn::Visibility::Inherited,
        //    ident: f.ident.clone(),
        //    colon_token: f.colon_token,
        //    ty: f.ty.clone(),
    });
    let expanded = quote! {
        pub struct #bident {
            #(#optionized,)*

            //#fields

            //executable: Option<String>,
            //args: Option<Vec<String>>,
            //env: Option<Vec<String>>,
            //current_dir: Option<String>,
        }
        impl #name {
            pub fn builder() -> #bident {
                #bident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
        impl #bident {
            fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }
            fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }
            fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    executable: self.executable.clone().ok_or("oh nyo executable")?,
                    args: self.args.clone().ok_or("oh nyo args")?,
                    env: self.env.clone().ok_or("oh nyo env")?,
                    current_dir: self.current_dir.clone().ok_or("oh nyo current_dir")?,
                })
            }
        }
    };

    expanded.into()
}
