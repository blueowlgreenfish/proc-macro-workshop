type PunctPreds = syn::punctuated::Punctuated<syn::WherePredicate, syn::token::Comma>;
type PredsIdent = (PunctPreds, std::collections::HashSet<syn::Ident>);
pub type OptPredsIdent = Option<PredsIdent>;

pub fn struct_attr(attrs: &[syn::Attribute]) -> OptPredsIdent {
    attrs
        .iter()
        .find_map(|attr| attr.parse_meta().ok().and_then(search::debug))
}

pub fn field_attr(
    meta: syn::Meta,
    opt_preds_ident: &mut OptPredsIdent,
) -> Option<syn::Result<syn::LitStr>> {
    fn transform(
        preds_ident: PredsIdent,
        opt_preds_ident: &mut OptPredsIdent,
    ) -> Option<syn::Result<syn::LitStr>> {
        if let Some((p, s)) = opt_preds_ident.as_mut() {
            p.extend(preds_ident.0);
            s.extend(preds_ident.1);
        } else {
            opt_preds_ident.replace(preds_ident);
        }
        None
    }
    search::debug(meta).and_then(|preds_ident| transform(preds_ident, opt_preds_ident))
}

mod search {
    use super::{OptPredsIdent, PunctPreds};

    pub fn debug(meta: syn::Meta) -> OptPredsIdent {
        let debug: syn::Path = syn::parse_quote!(debug);
        if meta.path() == &debug {
            search_bound(meta)
        } else {
            None
        }
    }

    fn search_bound(meta: syn::Meta) -> OptPredsIdent {
        if let syn::Meta::List(syn::MetaList { nested, .. }) = meta {
            nested.iter().find_map(predicate)
        } else {
            None
        }
    }

    fn predicate(m: &syn::NestedMeta) -> OptPredsIdent {
        let bound: syn::Path = syn::parse_quote!(bound);
        match m {
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path, lit, ..
            })) if path == &bound => {
                if let syn::Lit::Str(s) = lit {
                    let wp: PunctPreds = s
                        .parse_with(syn::punctuated::Punctuated::parse_terminated)
                        .ok()?;
                    let set = wp.iter().filter_map(search_generics_ident).collect();
                    Some((wp, set))
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    fn search_generics_ident(w: &syn::WherePredicate) -> Option<syn::Ident> {
        if let syn::WherePredicate::Type(syn::PredicateType {
            bounded_ty: syn::Type::Path(syn::TypePath { path, .. }),
            ..
        }) = w
        {
            path.segments.first().map(|seg| seg.ident.clone())
        } else {
            None
        }
    }
}
