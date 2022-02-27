use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> std::option::Option<&'a syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments[0].ident != wrapper {
            return std::option::Option::None;
        }
        if let syn::PathArguments::AngleBracketed(inner_ty) = &p.path.segments[0].arguments {
            let inner_ty = inner_ty.args.pairs().next().unwrap();
            if let syn::GenericArgument::Type(t) = inner_ty.value() {
                return std::option::Option::Some(t);
            }
        }
    }
    std::option::Option::None
}

fn builder_of(f: &syn::Field) -> std::option::Option<&syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "builder" {
            return std::option::Option::Some(attr);
        }
    }
    std::option::Option::None
}

fn mk_err<T: quote::ToTokens>(t: T) -> std::option::Option<(bool, proc_macro2::TokenStream)> {
    std::option::Option::Some((
        false,
        syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error(),
    ))
}

fn extend_method(f: &syn::Field) -> std::option::Option<(bool, proc_macro2::TokenStream)> {
    let name = f.ident.as_ref().unwrap();
    let g = builder_of(f)?;

    let meta = match g.parse_meta() {
        std::result::Result::Ok(syn::Meta::List(nvs)) => {
            // println!("{:#?}", nvs);
            assert_eq!(nvs.path.get_ident().unwrap(), "builder");
            if nvs.nested.len() != 1 {
                return mk_err(nvs);
            }

            let inner_ty = nvs.nested.pairs().next().unwrap();
            if let syn::NestedMeta::Meta(hi) = inner_ty.value() {
                match hi {
                    syn::Meta::NameValue(wow) => {
                        if wow.path.get_ident().unwrap() != "each" {
                            return mk_err(nvs);
                        }
                        wow.clone()
                    },
                    something => return mk_err(something),
                }
            } else {
                return mk_err(inner_ty.value());
            }
        },
        std::result::Result::Ok(ha) => {
            return mk_err(ha);
        },
        std::result::Result::Err(e) => {
            return std::option::Option::Some((false, e.to_compile_error()));
        },
    };

    match meta.lit {
        syn::Lit::Str(s) => {
            let arg = syn::Ident::new(&s.value(), s.span());
            let inner_ty = ty_inner_type("Vec", &f.ty).unwrap();
            let method = quote! {
                pub fn #arg(&mut self, #arg: #inner_ty) -> &mut Self {
                    self.#name.push(#arg);
                    self
                }
            };
            std::option::Option::Some((&arg == name, method))
        },
        lit => panic!("expected string, found {:?}", lit),
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", ast);
    let name = ast.ident;
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
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if ty_inner_type("Option", ty).is_some() || builder_of(f).is_some() {
            quote! { #name: #ty }
            /*
             * args: Vec<String>
             * env: Vec<String>
             * current_dir: Option<String>
             */
        } else {
            quote! { #name: std::option::Option<#ty> }
            /*
             * executable: Option<String>
             */
        }
    });
    let methods = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let set_method = if let std::option::Option::Some(inner_ty) = ty_inner_type("Option", ty) {
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = std::option::Option::Some(#name);
                    self
                }
                /*
                 * pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                 *     self.current_dir = Some(current_dir)
                 *     self
                 * }
                 */
            }
        } else if builder_of(f).is_some() {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
                    self
                }
                /*
                 * pub fn args(&mut self, args: Vec<String>) &mut Self {
                 *     self.args = args;
                 *     self
                 * }
                 * pub fn env(&mut self, env: Vec<String>) &mut Self {
                 *     self.env = env;
                 *     self
                 * }
                 */
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = std::option::Option::Some(#name);
                    self
                }
                /*
                 * pub fn executable(&mut self, executable: String) -> &mut Self {
                 *     self.executable = Some(executable);
                 *     self
                 * }
                 */
            }
        };
        match extend_method(f) {
            std::option::Option::None => set_method,
            std::option::Option::Some((true, extend_method)) => extend_method,
            std::option::Option::Some((false, extend_method)) => {
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
                #name: std::option::Option::None
            }
        }
    });
    let expanded = quote! {
        pub struct #bident {
            #(#optionized,)*
        }
        impl #bident {
            #(#methods)*

            pub fn build(&self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
        impl #name {
            fn builder() -> #bident {
                #bident {
                    #(#build_empty,)*
                }
            }
        }
    };

    expanded.into()
}
