use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);
    let ident = input.ident;
    let ident_string = ident.to_string();
    let generics = input.generics;
    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
    let generics_ident: Option<&syn::Ident> =
        if let Some(syn::GenericParam::Type(g)) = generics.params.first() {
            Some(&g.ident)
        } else {
            None
        };
    let data = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!();
    };
    let smaller_expand = data.clone().into_iter().map(|f| {
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
    let type_ident = data.into_iter().map(|f| {
        let type_path = if let syn::Type::Path(syn::TypePath { path, .. }) = f.ty {
            path
        } else {
            unimplemented!();
        };
        type_path.get_ident().map(|hi| hi.to_string())
    });
    //     impl<T> Debug for Field<T>
    //     where
    //         PhantomData<T>: Debug,
    //     {...}
    let expand = if let Some(gi) = generics_ident {
        let hello = type_ident.into_iter().next().unwrap();
        if hello.is_some() {
            if hello.clone().unwrap() == "PhantomData" {
                let phantom = Ident::new(&hello.unwrap(), Span::call_site());
                quote! {
                    impl #impl_generics std::fmt::Debug for #ident #ty_generics where #gi: std::fmt::Debug, #phantom<#gi>: std::fmt::Debug {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.debug_struct(#ident_string)
                                #(
                                    #smaller_expand
                                )*
                                .finish()
                        }
                    }
                }
            } else {
                quote! {
                    impl #impl_generics std::fmt::Debug for #ident #ty_generics where #gi: std::fmt::Debug {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.debug_struct(#ident_string)
                                #(
                                    #smaller_expand
                                )*
                                .finish()
                        }
                    }
                }
            }
        } else {
            quote! {}
        }
    } else {
        quote! {
            impl std::fmt::Debug for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#ident_string)
                        #(
                            #smaller_expand
                        )*
                        .finish()
                }
            }
        }
    };

    expand.into()
}
