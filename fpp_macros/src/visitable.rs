use crate::util::{camel_to_snake_case, ident_with_prefix};
use quote::quote;

pub(super) fn walkable_visit_derive(
    mut s: synstructure::Structure<'_>,
) -> proc_macro2::TokenStream {
    if let syn::Data::Union(_) = s.ast().data {
        panic!("cannot derive on union")
    }

    let visit_function_name = ident_with_prefix("visit_", &camel_to_snake_case(&s.ast().ident));

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
        quote! { crate::visit::Visitable::visit(#bind, __visitor)? }
    });

    s.bind_with(|_| synstructure::BindStyle::RefMut);
    let mut_visit = s.each(|bind| {
        quote! { crate::visit::MutVisitable::visit(#bind, __visitor)? }
    });

    s.gen_impl(quote! {
        gen impl<'__ast, __V> crate::visit::Walkable<'__ast, __V> for @Self
            where __V: crate::visit::Visitor<'__ast>,
        {
            fn walk_ref(&'__ast self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                match *self { #ref_visit }

                std::ops::ControlFlow::Continue(())
            }
        }

        gen impl<__V> crate::visit::MutWalkable<__V> for @Self
            where __V: crate::visit::MutVisitor,
        {
            fn walk_mut(&mut self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                match *self { #mut_visit }

                std::ops::ControlFlow::Continue(())
            }
        }

        gen impl<'__ast, __V> crate::Visitable<'__ast, __V> for @Self
            where __V: crate::visit::Visitor<'__ast>,
        {
            fn visit(&'__ast self, visitor: &mut __V) -> ::std::ops::ControlFlow<__V::Break> {
                visitor.#visit_function_name(self)
            }
        }

        gen impl<__V> crate::MutVisitable<__V> for @Self
            where __V: crate::visit::MutVisitor,
        {
            fn visit(&mut self, visitor: &mut __V) -> ::std::ops::ControlFlow<__V::Break> {
                visitor.#visit_function_name(self)
            }
        }
    })
}

pub(super) fn walkable_direct_derive(
    mut s: synstructure::Structure<'_>,
) -> proc_macro2::TokenStream {
    if let syn::Data::Union(_) = s.ast().data {
        panic!("cannot derive on union")
    }

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
        quote! { crate::visit::Visitable::visit(#bind, __visitor)? }
    });

    s.bind_with(|_| synstructure::BindStyle::RefMut);
    let mut_visit = s.each(|bind| {
        quote! { crate::visit::MutVisitable::visit(#bind, __visitor)? }
    });

    s.gen_impl(quote! {
        gen impl<'__ast, __V> crate::visit::Walkable<'__ast, __V> for @Self
            where __V: crate::visit::Visitor<'__ast>,
        {
            fn walk_ref(&'__ast self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                match *self { #ref_visit }

                std::ops::ControlFlow::Continue(())
            }
        }

        gen impl<__V> crate::visit::MutWalkable<__V> for @Self
            where __V: crate::visit::MutVisitor,
        {
            fn walk_mut(&mut self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
                match *self { #mut_visit }

                std::ops::ControlFlow::Continue(())
            }
        }

        gen impl<'__ast, __V> crate::Visitable<'__ast, __V> for @Self
            where __V: crate::visit::Visitor<'__ast>,
        {
            fn visit(&'__ast self, visitor: &mut __V) -> ::std::ops::ControlFlow<__V::Break> {
                self.walk_ref(visitor)
            }
        }

        gen impl<__V> crate::MutVisitable<__V> for @Self
            where __V: crate::visit::MutVisitor,
        {
            fn visit(&mut self, visitor: &mut __V) -> ::std::ops::ControlFlow<__V::Break> {
                self.walk_mut(visitor)
            }
        }
    })
}
