use std::collections::HashSet;

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

pub fn generics_add_debug<'a>(
    ty: &mut syn::TypeParam,
    field_ty: impl Iterator<Item = &'a syn::Type>,
    associated: &mut HashSet<&'a syn::Type>,
    bound: &HashSet<syn::Ident>,
) {
    let syn::TypeParam { ident, bounds, .. } = ty;
    let phantom_data: syn::Type = syn::parse_quote!(PhantomData<#ident>);
    // Do not add Debug trait constraint when the gnerics T contains associated types or T is PhantomData<T>.
    if !field_ty.fold(bound.contains(ident), |acc, t| {
        generics_search(t, ident, associated) || t == &phantom_data || acc
    }) {
        bounds.push(syn::parse_quote!(::std::fmt::Debug));
    }
}
