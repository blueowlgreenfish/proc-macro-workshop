use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

mod bound;
mod generics;

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
    opt_preds_ident: &mut bound::OptPredsIdent,
) -> syn::Result<proc_macro2::TokenStream> {
    fn debug(
        attr: &syn::Attribute,
        opt_preds_ident: &mut bound::OptPredsIdent,
    ) -> Option<syn::Result<syn::LitStr>> {
        match attr.parse_meta() {
            Ok(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit: syn::Lit::Str(s),
                ..
            })) if path.is_ident("debug") => Some(Ok(s)),
            Ok(meta) => bound::field_attr(meta, opt_preds_ident),
            _ => Some(Err(syn::Error::new(
                attr.span(),
                "failed to parse attr meta",
            ))),
        }
    }
    match attrs.iter().find_map(|attr| debug(attr, opt_preds_ident)) {
        // If attrs is an empty slice, it returns None.
        None => Ok(quote! { &self.#ident }),
        Some(Ok(fmt)) => Ok(quote! { &::std::format_args!(#fmt, self.#ident) }),
        Some(Err(err)) => Err(err),
    }
}

fn custom_debug(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let (ident, mut generics) = (input.ident, input.generics);
        let mut opt = bound::struct_attr(&input.attrs);
        let ident_str = ident.to_string();
        let field_idents = named.iter().map(|f| f.ident.as_ref().unwrap());
        let field_idents_str = field_idents.clone().map(|i| i.to_string());
        let field_rhs = field_idents
            .zip(named.iter().map(|f| f.attrs.as_slice()))
            .map(|(i, a)| attr_debug(a, i, &mut opt))
            .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

        let mut generics_associated = HashSet::with_capacity(8);
        let (mut bound_where_clause, bound_generics) = opt.unwrap_or_default();
        let closure = |g: &mut syn::TypeParam| {
            generics::generics_add_debug(
                g,
                named.iter().map(|f| &f.ty),
                &mut generics_associated,
                &bound_generics,
            )
        };
        generics
            .type_params_mut()
            // Use for_each here instead of map.
            .for_each(closure);

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut where_clause = where_clause
            .cloned()
            .unwrap_or_else(|| syn::parse_quote! { where });
        let convert =
            |ty: &syn::Type| -> syn::WherePredicate { syn::parse_quote!(#ty: ::std::fmt::Debug) };
        bound_where_clause.extend(generics_associated.into_iter().map(convert));
        where_clause.predicates.extend(bound_where_clause);

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
