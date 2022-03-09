use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, visit_mut::VisitMut};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut out = input.clone();

    let ty = parse_macro_input!(input as syn::Item);
    assert!(args.is_empty());

    if let Err(e) = sorted_impl(ty) {
        out.extend(TokenStream::from(e.to_compile_error()));
    }
    out
}

fn sorted_impl(input: syn::Item) -> Result<(), syn::Error> {
    if let syn::Item::Enum(e) = input {
        let mut names = Vec::new();
        for variant in e.variants.iter() {
            let name = variant.ident.to_string();
            if names.last().map(|last| &name < last).unwrap_or(false) {
                let next_lex_i = names.binary_search(&name).unwrap_err();
                return Err(syn::Error::new(
                    // syn::spanned::Spanned::span(&variant),
                    variant.span(),
                    format!("{} should sort before {}", name, names[next_lex_i]),
                ));
            }
            names.push(name);
        }
        Ok(())
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

#[derive(Default)]
struct LexiographicMatching {
    errors: Vec<syn::Error>,
}

fn path_as_string(path: &syn::Path) -> String {
    format!("{}", quote! { #path })
}

fn get_arm_name(arm: &syn::Pat) -> Option<String> {
    match *arm {
        syn::Pat::Ident(syn::PatIdent {
            subpat: Some((_, ref sp)),
            ..
        }) => get_arm_name(sp),
        syn::Pat::Struct(ref s) => Some(path_as_string(&s.path)),
        syn::Pat::TupleStruct(ref s) => Some(path_as_string(&s.path)),
        _ => None,
    }
}

impl syn::visit_mut::VisitMut for LexiographicMatching {
    fn visit_expr_match_mut(&mut self, m: &mut syn::ExprMatch) {
        if m.attrs.iter().any(|a| a.path.is_ident("sorted")) {
            m.attrs.retain(|a| !a.path.is_ident("sorted"));
            let mut names = Vec::new();
            for arm in m.arms.iter() {
                let name = get_arm_name(&arm.pat).unwrap();
                if names.last().map(|last| &name < last).unwrap_or(false) {
                    let next_lex_i = names.binary_search(&name).unwrap_err();
                    self.errors.push(syn::Error::new(
                        arm.span(),
                        format!("{} should sort before {}", name, names[next_lex_i]),
                    ));
                }
                names.push(name);
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, m)
    }
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as syn::ItemFn);
    assert!(args.is_empty());

    let mut lm = LexiographicMatching::default();
    lm.visit_item_fn_mut(&mut f);
    let mut ts = quote! { #f };
    ts.extend(lm.errors.into_iter().map(|e| e.to_compile_error()));
    ts.into()
}
