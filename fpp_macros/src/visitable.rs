use proc_macro2::Ident;
use quote::quote;

fn camel_to_snake_case(ident: &Ident, prefix: &str) -> Ident {
    let name = ident.to_string();
    let mut snake_case_name = String::new();
    let mut prev_char_is_upper = false; // To handle consecutive uppercase letters

    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            // Insert underscore if it's not the first character and not a consecutive uppercase
            if i > 0 && !prev_char_is_upper {
                snake_case_name.push('_');
            }
            snake_case_name.push(c.to_ascii_lowercase());
            prev_char_is_upper = true;
        } else {
            snake_case_name.push(c);
            prev_char_is_upper = false;
        }
    }
    Ident::new(&(prefix.to_string() + &snake_case_name), ident.span())
}

pub(super) fn visitable_derive(
    mut s: synstructure::Structure<'_>,
    no_self: bool,
) -> proc_macro2::TokenStream {
    if let syn::Data::Union(_) = s.ast().data {
        panic!("cannot derive on union")
    }

    let visit_function_name = camel_to_snake_case(&s.ast().ident, "visit_");

    let has_attr = |attrs: &[syn::Attribute], name| {
        let mut found = false;
        attrs.iter().for_each(|attr| {
            if !attr.path().is_ident("visitable") {
                return;
            }
            let _ = attr.parse_nested_meta(|nested| {
                if nested.path.is_ident(name) {
                    found = true;
                }
                Ok(())
            });
        });
        found
    };

    let get_attr = |attrs: &[syn::Attribute], name: &str| {
        let mut content = None;
        attrs.iter().for_each(|attr| {
            if !attr.path().is_ident("visitable") {
                return;
            }
            let _ = attr.parse_nested_meta(|nested| {
                if nested.path.is_ident(name) {
                    let value = nested.value()?;
                    let value = value.parse()?;
                    content = Some(value);
                }
                Ok(())
            });
        });
        content
    };

    s.add_bounds(synstructure::AddBounds::Generics);

    // Ignore all fields that are manually specified to ignore or that
    // are internal to the AST Node
    s.filter_variants(|v| !has_attr(&v.ast().attrs, "ignore"));
    s.filter(|f| {
        let field_name = match &f.ast().ident {
            None => "".to_string(),
            Some(ident) => ident.to_string(),
        };

        !(has_attr(&f.ast().attrs, "ignore") || field_name == "node_id")
    });

    let ref_visit = s.each(|bind| {
        let extra = get_attr(&bind.ast().attrs, "extra").unwrap_or(quote! {});
        quote! { crate::visit::Walkable::walk_ref(#bind, __visitor, #extra)? }
    });

    s.bind_with(|_| synstructure::BindStyle::RefMut);
    let mut_visit = s.each(|bind| {
        let extra = get_attr(&bind.ast().attrs, "extra").unwrap_or(quote! {});
        quote! { crate::visit::MutWalkable::walk_mut(#bind, __visitor, #extra)? }
    });

    let visit_self = match no_self {
        true => quote! {},
        false => quote! { __visitor.#visit_function_name(self)?; }
    };

    s.gen_impl(quote! {
        gen impl<'__ast, __V> crate::visit::Walkable<'__ast, __V> for @Self
            where __V: crate::visit::Visitor<'__ast>,
        {
            fn walk_ref(&'__ast self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                #visit_self
                match *self { #ref_visit }

                std::ops::ControlFlow::Continue(())
            }
        }

        gen impl<'__ast, __V> crate::visit::MutWalkable<'__ast, __V> for @Self
            where __V: crate::visit::MutVisitor<'__ast>,
        {
            fn walk_mut(&'__ast mut self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                #visit_self
                match *self { #mut_visit }

                std::ops::ControlFlow::Continue(())
            }
        }
    })
}
