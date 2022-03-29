use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", input);

    TokenStream::from(match custom_debug(input) {
        Ok(token) => token,
        Err(err) => err.to_compile_error(),
    })
}

fn attr_debug(
    attrs: &[syn::Attribute],
    ident: &syn::Ident,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    fn debug(attr: &syn::Attribute) -> Option<syn::Result<syn::LitStr>> {
        match attr.parse_meta() {
            Ok(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit: syn::Lit::Str(s),
                ..
            })) if path.is_ident("debug") => Some(Ok(s)),
            _ => Some(Err(syn::Error::new(
                attr.span(),
                "failed to parse attr meta",
            ))),
        }
    }
    match attrs.iter().find_map(debug) {
        // If attrs is an empty slice, it returns None.
        None => Ok(None),
        Some(Ok(fmt)) => Ok(Some(quote! { &::std::format_args!(#fmt, self.#ident) })),
        Some(Err(err)) => Err(err),
    }
}

fn generics_search<'a>(
    ty: &'a syn::Type,
    ident: &syn::Ident,
    associated: &mut HashSet<&'a syn::Type>,
) -> bool {
    fn check_associated<'a>(
        ty: &'a syn::Type,
        ident: &syn::Ident,
        associated: &mut HashSet<&'a syn::Type>,
    ) -> bool {
        if let syn::Type::Path(syn::TypePath {
            path:
                syn::Path {
                    segments,
                    leading_colon: None,
                },
            ..
        }) = ty
        {
            if segments.len() > 1
                && segments
                    .first()
                    .map(|seg| &seg.ident == ident)
                    .unwrap_or(false)
            {
                associated.insert(ty);
                return true;
            }
        }
        false
    }
    fn check_angle_bracket_associated<'a>(
        ty: &'a syn::Type,
        ident: &syn::Ident,
        associated: &mut HashSet<&'a syn::Type>,
    ) -> bool {
        fn check<'a>(
            arg: &'a syn::PathArguments,
            ident: &syn::Ident,
            associated: &mut HashSet<&'a syn::Type>,
        ) -> bool {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
            }) = arg
            {
                args.iter().fold(false, |acc, arg| {
                    if let syn::GenericArgument::Type(t) = arg {
                        check_associated(t, ident, associated) || acc
                    } else {
                        acc
                    }
                })
            } else {
                false
            }
        }
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = ty
        {
            return segments
                .last()
                .map(|seg| check(&seg.arguments, ident, associated))
                .unwrap_or(false);
        }
        false
    }

    check_associated(ty, ident, associated) || check_angle_bracket_associated(ty, ident, associated)
}

fn generics_add_debug<'a>(
    ty: &mut syn::TypeParam,
    field_ty: impl Iterator<Item = &'a syn::Type>,
    associated: &mut std::collections::HashSet<&'a syn::Type>,
) {
    let syn::TypeParam { ident, bounds, .. } = ty;
    let phantom_data: &syn::Type = &syn::parse_quote!(PhantomData<#ident>);
    // Do not add Debug trait constraint when the gnerics T contains associated types or T is PhantomData<T>.
    if !field_ty.fold(false, |acc, t| {
        generics_search(t, ident, associated) || t == phantom_data || acc
    }) {
        bounds.push(syn::parse_quote!(::std::fmt::Debug));
    }
}

fn custom_debug(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let (ident, mut generics) = (input.ident, input.generics);
        let ident_str = ident.to_string();
        let field_idents = named.iter().map(|f| f.ident.as_ref().unwrap());
        let field_idents_str = field_idents.clone().map(|i| i.to_string());
        let field_rhs = field_idents
            .zip(named.iter().map(|f| f.attrs.as_slice()))
            .map(|(i, a)| attr_debug(a, i).map(|t| t.unwrap_or(quote! { &self.#i })))
            .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

        let mut generics_associated = HashSet::with_capacity(8);
        generics
            .type_params_mut()
            // Use for_each here instead of map.
            .for_each(|tp| {
                generics_add_debug(tp, named.iter().map(|f| &f.ty), &mut generics_associated)
            });

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        Ok(quote! {
            impl #impl_generics ::std::fmt::Debug for #ident #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                    f.debug_struct(#ident_str)
                        #(
                            .field(#field_idents_str, #field_rhs)
                        )*
                        .finish()
                }
            }
        })
    } else {
        Err(syn::Error::new(input.span(), "Named Struct Only"))
    }
}
