extern crate proc_macro2;
extern crate quote;
extern crate syn;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Error, parse_macro_input};
use syn::{DeriveInput, Fields};

#[proc_macro_derive(Ast)]
pub fn ast_node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let out = match input.data {
        Data::Struct(s) => match s.fields {
            // Named fields of a struct or struct variant such as `Point { x: f64,
            // y: f64 }`.
            Fields::Named(sct) => match (sct.named.first(), sct.named.last()) {
                (Some(a), Some(b)) => {
                    let a_name = a.ident.clone().unwrap();
                    let b_name = b.ident.clone().unwrap();

                    Ok(quote! {
                        impl Node for #name {
                            fn span(&self) -> Span {
                                Span::merge(self.#a_name.span(), self.#b_name.span())
                            }
                        }
                    })
                }

                _ => Err(Error::new(
                    name.span(),
                    "AST structured node must have at least one field",
                )),
            },
            // Unnamed fields of a tuple struct or tuple variant such as `Some(T)`.
            Fields::Unnamed(_) => Err(Error::new(
                name.span(),
                "Unnamed structs cannot be AST nodes",
            )),
            Fields::Unit => Err(Error::new(name.span(), "Unit structs cannot be AST nodes")),
        },
        Data::Union(_) => Err(Error::new(name.span(), "Unions cannot be AST nodes")),
        Data::Enum(e) => {
            let fields = &e.variants;
            let name_rep = vec![&name; fields.len()];
            let field_names = fields.iter().map(|f| &f.ident);

            Ok(quote! {
                impl Node for #name {
                    fn span(&self) -> Span {
                        match self {
                            #(#name_rep::#field_names(x) => x.span()),*
                        }
                    }
                }
            })
        },
    };

    match out {
        Ok(ts) => ts.into(),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
